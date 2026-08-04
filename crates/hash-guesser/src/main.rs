//! Guess names for bin hashes nothing resolves yet.
//!
//! A rewrite of LeagueHashes' `xguesser.cpp`. The idea is unchanged, and it is
//! a good one: names in this game are not arbitrary strings but arrangements of
//! a small vocabulary, so an unknown name is very likely to be words we have
//! already seen, in an order we have already seen them in. Take the vocabulary
//! out of the names we know, recombine it, and keep whatever hashes to
//! something we are missing.
//!
//! What changed in the rewrite:
//!
//!   - The target set comes from `dumps/`, not a checked-in `all.*.txt`. Every
//!     hash the game has used across 245 builds, current the moment a dump
//!     lands, with no second file to maintain.
//!   - Output is PascalCase, always, and is checked before it is printed. That
//!     is the repo rule (see `scripts/names.py`), and it means results can be
//!     piped straight into `hashtool add`. The Hungarian prefixes the original
//!     carried (`m`, `b`, `ar`) are gone with it - each produced a camelCase
//!     name we would now reject.
//!   - The mutation pass is a plain "substitute or insert at each position",
//!     rather than four interleaved stacks with mutation flags whose reporting
//!     conditions did not line up. Same intent, minus the double-counting.
//!   - An identity pass runs first, trying every known name against the other
//!     table. Class and field vocabularies overlap heavily and these guesses
//!     cost one probe each.
//!   - Every I-named candidate is held to a class the meta actually flags as an
//!     interface. This is the only check here that is evidence rather than
//!     arithmetic - the flag comes out of the dump, not out of the hash - and it
//!     refutes about half of all I-named candidates at no cost in real ones.
//!   - `--suffix` anchors the tail of a name for free, by folding the suffix
//!     backwards out of each target instead of hashing it on every probe. Most
//!     of the big families here are suffix families (Data 529, Controller 201,
//!     Driver 138), and reaching them by searching a word deeper costs hundreds
//!     of times the noise.
//!   - Two modes the original did not have. `delete` drops one word from each
//!     known name: ~10^5 probes for the whole pass, and a shape the corpus is
//!     full of - 5521 of 13799 known names are another known name minus one
//!     interior word. `chain` is `force` constrained to word pairs seen in
//!     known names, which cuts depth 3 from 64M probes to ~400k and brings
//!     depth 4 in cheaper than uniform depth 3 was. Priors on generation,
//!     unlike priors on filtering, lower the noise instead of failing to
//!     discriminate.
//!   - Parallel, and the hot path allocates nothing: a candidate's string is
//!     built only once its hash has already matched.
//!
//! Usage:
//!
//!     # the wordlist, from every name the repo knows
//!     python3 scripts/split_words.py hashes/hashes.*.txt hashes/overrides/*.txt > words.txt
//!
//!     # substitute and insert around known names - the mode to run first
//!     cargo run --release -p hash-guesser -- binfields --words words.txt
//!
//!     # one word deleted from each known name - the cheapest recall there is
//!     cargo run --release -p hash-guesser -- binfields --mode delete
//!
//!     # exhaustive, and explosive: cost is len(wordlist)^depth
//!     cargo run --release -p hash-guesser -- bintypes --words words.txt \
//!         --mode force --depth 2
//!
//!     # depth 4, but only along word pairs attested in known names
//!     cargo run --release -p hash-guesser -- bintypes --words words.txt \
//!         --mode chain --depth 4
//!
//!     # two words plus a known tail, for the price of two words
//!     cargo run --release -p hash-guesser -- bintypes --words words.txt \
//!         --mode force --depth 2 --suffix Data --suffix Controller
//!
//! Every line of output is a *candidate*. A hash colliding with a plausible
//! name is not evidence the name is right - 32 bits is not many - so nothing
//! here should reach an override table until it has been checked against
//! shipped data. The run prints its own expected false-positive count for
//! exactly that reason: when it approaches the hit count, the output is chance.

mod fold;
mod guess;
mod input;
mod words;

use std::collections::{BTreeSet, HashSet};
use std::io::Write;
use std::path::PathBuf;
use std::sync::atomic::Ordering;
use std::sync::atomic::AtomicU64;

use anyhow::{Context, Result};
use clap::{Parser, ValueEnum};

use guess::{Found, Guesser, Prefix, Targets};

#[derive(Copy, Clone, PartialEq, Eq, ValueEnum)]
enum Mode {
    /// Try every known name verbatim. Nearly free; catches names shared
    /// between the class and field tables.
    Identity,
    /// Substitute and insert each wordlist word at each position of each known
    /// name. The default, and the one that finds things.
    Mutate,
    /// Delete one word from each position of each known name. The third edit
    /// beside mutate's two, split out because it needs no wordlist and costs
    /// ~10^5 probes total - run it with identity, which covers the last-word
    /// deletions (they are prefixes).
    Delete,
    /// Every arrangement of up to --depth words. Exhaustive, and exponential.
    Force,
    /// Force, but each word must follow a word it has followed in some known
    /// name. Two orders of magnitude fewer probes per depth, so depth 4 is
    /// cheaper than uniform depth 3 over the top 400 words.
    Chain,
}

#[derive(Copy, Clone, PartialEq, Eq, ValueEnum)]
enum Table {
    Bintypes,
    Binfields,
}

impl Table {
    fn as_str(self) -> &'static str {
        match self {
            Table::Bintypes => "bintypes",
            Table::Binfields => "binfields",
        }
    }
}

#[derive(Parser)]
#[command(about = "Guess names for unresolved bin hashes", long_about = None)]
struct Args {
    /// Which table to crack.
    #[arg(value_enum)]
    table: Table,

    /// Wordlist, as written by scripts/split_words.py. Required unless the
    /// mode is `identity`.
    #[arg(short, long)]
    words: Option<PathBuf>,

    #[arg(long, value_enum, default_value = "mutate")]
    mode: Mode,

    /// Word count for --mode force. 2 is seconds, 3 wants --top.
    #[arg(long, default_value_t = 2)]
    depth: usize,

    /// Use only the first N words. The wordlist is frequency-ordered, so this
    /// keeps the productive end and drops the tail.
    #[arg(long)]
    top: Option<usize>,

    #[arg(long, default_value = "dumps")]
    dumps: PathBuf,

    #[arg(long, default_value = "hashes")]
    hashes: PathBuf,

    /// Target hashes from this `{hash} {name}` file instead of from dumps/.
    #[arg(long)]
    all: Option<PathBuf>,

    /// Hunt only these hashes (a file of hex hashes, one per line; a trailing
    /// name is ignored).
    ///
    /// This is the lever that makes an intensive search worth running. False
    /// positives scale with `probes x targets / 2^32`, so the target count is
    /// half the noise budget: chasing one family of 70 hashes instead of all
    /// 2554 buys a 36x deeper search at the same noise. Pair it with --prefix
    /// when the family's first word is known.
    #[arg(long)]
    only: Option<PathBuf>,

    /// Files or directories of names already tried and rejected. Defaults to
    /// hashes/bad/ when it exists.
    #[arg(long)]
    bad: Vec<PathBuf>,

    /// Leading string to try on every candidate; repeatable. Defaults to "" and,
    /// for bintypes, "I".
    #[arg(long)]
    prefix: Vec<String>,

    /// Trailing string to try on every candidate; repeatable. Replaces the
    /// default of "" (no suffix), exactly as --prefix does.
    ///
    /// This is the cheap way to search one more word deep. `--suffix Data
    /// --suffix Controller` covers every "<two words>Data" and
    /// "<two words>Controller" at the cost of a two-word search, because the
    /// suffix is folded backwards into the target set once rather than hashed on
    /// every probe. It multiplies the noise by the number of suffixes, not by
    /// the size of the wordlist: 12 suffixes at depth 2 is 260x quieter than
    /// depth 3, for names in those 12 families.
    ///
    /// The big families, by count over named classes: Data 529, Controller 201,
    /// Driver 138, Def 61, Component 48, Definition 47, Updater 43, Base 43,
    /// Block 41, List 41, Get 37, Config 34.
    #[arg(long)]
    suffix: Vec<String>,

    /// Read suffixes from a file, one per line. Merged with --suffix.
    #[arg(long)]
    suffix_list: Option<PathBuf>,

    /// Keep the leading Hungarian prefix when splitting known names into words.
    #[arg(long)]
    keep_prefix: bool,

    /// Don't hold the "I" prefix to classes the meta flags as interfaces.
    /// Only useful if you think the flag is unreliable for a given build.
    #[arg(long)]
    no_interface_filter: bool,

    /// Write results here instead of stdout.
    #[arg(short, long)]
    out: Option<PathBuf>,
}

const PREFIX_MAX: usize = 2;

fn main() -> Result<()> {
    let args = Args::parse();
    let table = args.table.as_str();

    // Known: the vendored upstream mirror plus our overrides. Both are names we
    // already have, so both are excluded from the targets and both contribute
    // vocabulary.
    let mirror = input::read_table(&args.hashes.join(format!("hashes.{table}.txt")))?;
    let overrides = input::read_table(&args.hashes.join("overrides").join(format!("{table}.txt")))
        .unwrap_or_default();

    // The target set: every hash the game uses, minus the ones already named.
    // `interfaces` comes back only from a dump sweep - an explicit --all file
    // carries hashes and nothing else, so the interface filter goes with it.
    let mut interfaces: Option<HashSet<u32>> = None;
    let all: HashSet<u32> = match &args.all {
        Some(path) => input::read_table(path)?.into_keys().collect(),
        None => {
            let dumped = input::read_dump_hashes(&args.dumps)?;
            interfaces = Some(dumped.interfaces);
            match args.table {
                Table::Bintypes => dumped.types,
                Table::Binfields => dumped.fields,
            }
        }
    };
    let known: HashSet<u32> = mirror.keys().chain(overrides.keys()).copied().collect();
    let mut targets: HashSet<u32> = all.difference(&known).copied().collect();
    if let Some(path) = &args.only {
        let wanted: HashSet<u32> = input::read_hash_list(path)?;
        let before = targets.len();
        targets.retain(|h| wanted.contains(h));
        let already: Vec<u32> = wanted.iter().copied().filter(|h| known.contains(h)).collect();
        eprintln!(
            "[..] --only: {} of {} listed hash(es) are unresolved targets \
             (was {before}); {} already have names",
            targets.len(),
            wanted.len(),
            already.len()
        );
    }
    anyhow::ensure!(
        !targets.is_empty(),
        "nothing to crack: every {table} hash in the corpus already resolves"
    );

    let bad_paths = if args.bad.is_empty() {
        let default = args.hashes.join("bad");
        if default.is_dir() {
            vec![default]
        } else {
            Vec::new()
        }
    } else {
        args.bad.clone()
    };
    let bad = input::read_bad(&bad_paths)?;

    let prefixes: Vec<Prefix> = if args.prefix.is_empty() {
        match args.table {
            // `I` is the interface convention, and common enough that leaving
            // it to the wordlist would waste a word slot on every candidate.
            Table::Bintypes => vec![Prefix::new(""), Prefix::new("I")],
            Table::Binfields => vec![Prefix::new("")],
        }
    } else {
        args.prefix.iter().map(|p| Prefix::new(p)).collect()
    };

    // Hold every I-named candidate to a class the meta actually flags as an
    // interface. Unlike anything else here this is evidence rather than
    // arithmetic - the flag comes out of the dump, not out of the hash - and it
    // refutes about half of all I-named candidates.
    //
    // Checked on the finished name, not on the search prefix. `I` is a word in
    // the wordlist (rank 26, out of splitting every `IFoo` name), so `force` and
    // `mutate` build `I`+`Model`+`Joint`+`Content` under the empty prefix and
    // sail straight past a prefix-scoped check. That hole is how
    // `f9cfefd4 IModelJointContent` reached a candidate file for a class whose
    // meta says `interface: false`.
    //
    // The converse is deliberately not applied: 191 of the 316 interfaces we
    // have names for are *not* I-named, so "class is an interface" says nothing
    // about a non-I candidate.
    let interface_filter: Option<HashSet<u32>> = match (&interfaces, args.no_interface_filter) {
        (Some(ifaces), false) => {
            let only: HashSet<u32> = ifaces.intersection(&all).copied().collect();
            eprintln!(
                "[..] I-named candidates held to the {} class(es) the meta \
                 marks as interfaces",
                only.len()
            );
            Some(only)
        }
        (None, false) => {
            eprintln!(
                "[warn] --all was given, so there is no interface flag to check \
                 I-named candidates against; expect roughly half of them to be \
                 wrong"
            );
            None
        }
        (_, true) => None,
    };
    for p in &prefixes {
        anyhow::ensure!(
            p.text.chars().all(|c| c.is_ascii_alphanumeric()),
            "prefix {:?} would produce a name that is not PascalCase",
            p.text
        );
    }

    // Suffixes, folded backwards into the target set below. Capitalized because
    // the hash lowercases anyway, so casing here is only about the name we
    // print - and a lowercase suffix would print a name `is_pascal` rejects.
    let mut suffixes: Vec<String> = Vec::new();
    if let Some(path) = &args.suffix_list {
        suffixes.extend(input::read_suffix_list(path)?);
    }
    suffixes.extend(args.suffix.iter().cloned());
    if suffixes.is_empty() {
        suffixes.push(String::new());
    }
    for s in &mut *suffixes {
        anyhow::ensure!(
            s.chars().all(|c| c.is_ascii_alphanumeric()),
            "suffix {s:?} would produce a name that is not PascalCase"
        );
        *s = words::capitalize(s);
    }
    suffixes.sort();
    suffixes.dedup();
    anyhow::ensure!(
        suffixes.len() <= u16::MAX as usize,
        "at most {} suffixes",
        u16::MAX
    );

    // Vocabulary comes from both tables regardless of which one is being
    // cracked: a field name is built from the same words as a class name, and
    // halving the corpus halves the sentences the mutation pass has to work
    // from for no gain.
    let prefix_max = if args.keep_prefix { 0 } else { PREFIX_MAX };
    let mut all_names: Vec<String> = Vec::new();
    for t in ["bintypes", "binfields"] {
        all_names.extend(
            input::read_table(&args.hashes.join(format!("hashes.{t}.txt")))
                .unwrap_or_default()
                .into_values(),
        );
        all_names.extend(
            input::read_table(&args.hashes.join("overrides").join(format!("{t}.txt")))
                .unwrap_or_default()
                .into_values(),
        );
    }
    let sentences = guess::sentences(all_names, prefix_max);

    let wordlist = match (&args.words, args.mode) {
        (Some(path), _) => {
            let mut w = input::read_wordlist(path)?;
            if let Some(n) = args.top {
                w.truncate(n);
            }
            w
        }
        (None, Mode::Identity | Mode::Delete) => Vec::new(),
        (None, _) => anyhow::bail!(
            "--words is required for --mode {}; build one with \
             `python3 scripts/split_words.py hashes/hashes.*.txt \
             hashes/overrides/*.txt > words.txt`",
            match args.mode {
                Mode::Mutate => "mutate",
                Mode::Force => "force",
                Mode::Chain => "chain",
                Mode::Identity | Mode::Delete => unreachable!(),
            }
        ),
    };

    eprintln!(
        "[..] {table}: {} unresolved of {} hashes in the corpus ({} already named)",
        targets.len(),
        all.len(),
        known.len()
    );
    eprintln!(
        "[..] {} sentence(s), {} word(s), {} prefix(es), {} suffix(es), \
         {} name(s) ruled out",
        sentences.len(),
        wordlist.len(),
        prefixes.len(),
        suffixes.len(),
        bad.len()
    );

    // Fold each suffix backwards through each target. `fnv1a_back` on the empty
    // suffix is the identity, so the unanchored search builds the same state set
    // it always did.
    let entries: Vec<(u32, u32, u16)> = targets
        .iter()
        .flat_map(|&t| {
            suffixes.iter().enumerate().map(move |(i, s)| {
                (input::fnv1a_back(s.as_bytes(), t), t, i as u16)
            })
        })
        .collect();

    let guesser = Guesser {
        targets: Targets::new(entries),
        suffixes,
        bad,
        prefixes,
        interfaces: interface_filter,
        probes: AtomicU64::new(0),
    };

    let mut found: Vec<Found> = Vec::new();
    match args.mode {
        Mode::Identity => found.extend(guesser.identity(&sentences)),
        Mode::Mutate => {
            // The identity pass is a rounding error next to the mutation pass
            // and hits a different kind of name, so it always runs first.
            let ident = guesser.identity(&sentences);
            // Counted after dedup, because the raw count is badly inflated: a
            // name is probed once per sentence it prefixes, so `AbilityResource`
            // is rediscovered from every `AbilityResource*` there is. Reporting
            // the raw length made this pass look four times more productive
            // than it is.
            let distinct: BTreeSet<(u32, &str)> =
                ident.iter().map(|f| (f.hash, f.name.as_str())).collect();
            eprintln!("[..] identity pass: {} candidate(s)", distinct.len());
            found.extend(ident);
            found.extend(guesser.mutate(&sentences, &wordlist));
        }
        Mode::Delete => {
            // Pure deletions only. The identity pass is not folded in the way
            // mutate folds it in, because here it is not a rounding error -
            // the two passes are the same order of magnitude - and keeping the
            // modes separate keeps the sweep's per-run noise attribution
            // honest. Identity covers the last-word deletions (prefixes), so
            // run both.
            found.extend(guesser.delete(&sentences));
        }
        Mode::Force => {
            anyhow::ensure!(args.depth >= 1, "--depth must be at least 1");
            let combos = (wordlist.len() as f64).powi(args.depth as i32);
            eprintln!(
                "[..] force depth {}: ~{:.3e} combination(s)",
                args.depth, combos
            );
            found.extend(guesser.force(&wordlist, args.depth));
        }
        Mode::Chain => {
            anyhow::ensure!(args.depth >= 1, "--depth must be at least 1");
            let bigrams = guess::Bigrams::new(&sentences, &wordlist);
            // Exact, not an estimate: the DP counts the walk the run is about
            // to make, so the noise is known before it is spent.
            eprintln!(
                "[..] chain depth {}: {} attested bigram(s), exactly {} probe(s)",
                args.depth,
                bigrams.bigram_count(),
                bigrams.probes(args.depth) * guesser.prefixes.len() as u64,
            );
            found.extend(guesser.chain(&bigrams, args.depth));
        }
    }

    // Dedup: the same name is reachable by many routes (insert-at-end of one
    // sentence is substitute-at-end of another). Ordered by hash so a rerun
    // produces a diffable file.
    let mut unique: BTreeSet<(u32, String)> = BTreeSet::new();
    for f in &found {
        debug_assert!(guess::verify(f), "{} does not hash to {:08x}", f.name, f.hash);
        if guess::verify(f) {
            unique.insert((f.hash, f.name.clone()));
        }
    }

    let mut sink: Box<dyn Write> = match &args.out {
        Some(path) => Box::new(
            std::fs::File::create(path)
                .with_context(|| format!("creating {}", path.display()))?,
        ),
        None => Box::new(std::io::stdout()),
    };
    for (h, name) in &unique {
        writeln!(sink, "{h:08x} {name}")?;
    }
    sink.flush()?;

    let hits: HashSet<u32> = unique.iter().map(|(h, _)| *h).collect();
    let probes = guesser.probes.load(Ordering::Relaxed);
    eprintln!(
        "[ok] {probes} probe(s) -> {} candidate(s) for {} distinct hash(es)",
        unique.len(),
        hits.len()
    );

    // The number that decides whether any of this is worth reading. Each probe
    // is one draw against `states` targets out of 2^32, so the run is expected
    // to turn up this many names that hash correctly and mean nothing. When it
    // is of the same order as the hit count, the output is noise: cut the
    // wordlist, cut the targets with --only, or anchor with --prefix/--suffix.
    let states = guesser.targets.states() as f64;
    let noise = probes as f64 * states / 4_294_967_296.0;
    eprintln!(
        "[..] {states:.0} target state(s) x {probes} probe(s) / 2^32 = \
         {noise:.3} expected false positive(s)"
    );
    if !unique.is_empty() {
        if noise >= unique.len() as f64 * 0.5 {
            eprintln!(
                "[warn] expected noise ({noise:.1}) is the same order as the \
                 hit count ({}) - this output is not distinguishable from \
                 chance",
                unique.len()
            );
        }
        eprintln!(
            "[note] these are candidates, not cracks. A 32-bit hash collides \
             with plausible names by chance - confirm each against shipped data \
             before `hashtool add`"
        );
    }
    Ok(())
}
