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
//!   - The `I` prefix is held to classes the meta actually flags as interfaces.
//!     This is the only check here that is evidence rather than arithmetic -
//!     the flag comes out of the dump, not out of the hash - and it refutes
//!     about half of all I-prefixed candidates at no cost in real ones.
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
//!     # exhaustive, and explosive: cost is len(wordlist)^depth
//!     cargo run --release -p hash-guesser -- bintypes --words words.txt \
//!         --mode force --depth 2
//!
//! Every line of output is a *candidate*. A hash colliding with a plausible
//! name is not evidence the name is right - 32 bits is not many - so nothing
//! here should reach an override table until it has been checked against
//! shipped data.

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
    /// Every arrangement of up to --depth words. Exhaustive, and exponential.
    Force,
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

    /// Files or directories of names already tried and rejected. Defaults to
    /// hashes/bad/ when it exists.
    #[arg(long)]
    bad: Vec<PathBuf>,

    /// Leading string to try on every candidate; repeatable. Defaults to "" and,
    /// for bintypes, "I".
    #[arg(long)]
    prefix: Vec<String>,

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
    let targets: HashSet<u32> = all.difference(&known).copied().collect();
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

    let mut prefixes: Vec<Prefix> = if args.prefix.is_empty() {
        match args.table {
            // `I` is the interface convention, and common enough that leaving
            // it to the wordlist would waste a word slot on every candidate.
            Table::Bintypes => vec![Prefix::new(""), Prefix::new("I")],
            Table::Binfields => vec![Prefix::new("")],
        }
    } else {
        args.prefix.iter().map(|p| Prefix::new(p)).collect()
    };

    // Hold the `I` prefix to classes the meta actually flags as interfaces.
    // Half of all I-prefixed candidates are refuted this way, and unlike
    // anything else here it is evidence rather than arithmetic - the flag comes
    // out of the dump, not out of the hash. The converse is deliberately not
    // applied: 191 of the 316 interfaces we have names for are *not* I-named,
    // so "class is an interface" says nothing about a non-I candidate.
    if !args.no_interface_filter {
        if let Some(ifaces) = &interfaces {
            for p in &mut prefixes {
                if p.text == "I" {
                    let only: HashSet<u32> = ifaces.intersection(&all).copied().collect();
                    eprintln!(
                        "[..] prefix \"I\" restricted to {} class(es) the meta \
                         marks as interfaces",
                        only.len()
                    );
                    *p = Prefix::new("I").restricted_to(only);
                }
            }
        } else if prefixes.iter().any(|p| p.text == "I") {
            eprintln!(
                "[warn] --all was given, so there is no interface flag to \
                 check the \"I\" prefix against; expect roughly half of the \
                 I-prefixed candidates to be wrong"
            );
        }
    }
    for p in &prefixes {
        anyhow::ensure!(
            p.text.chars().all(|c| c.is_ascii_alphanumeric()),
            "prefix {:?} would produce a name that is not PascalCase",
            p.text
        );
    }

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
        (None, Mode::Identity) => Vec::new(),
        (None, _) => anyhow::bail!(
            "--words is required for --mode {}; build one with \
             `python3 scripts/split_words.py hashes/hashes.*.txt \
             hashes/overrides/*.txt > words.txt`",
            match args.mode {
                Mode::Mutate => "mutate",
                Mode::Force => "force",
                Mode::Identity => "identity",
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
        "[..] {} sentence(s), {} word(s), {} prefix(es), {} name(s) ruled out",
        sentences.len(),
        wordlist.len(),
        prefixes.len(),
        bad.len()
    );

    let guesser = Guesser {
        targets: Targets::new(targets),
        bad,
        prefixes,
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
        Mode::Force => {
            anyhow::ensure!(args.depth >= 1, "--depth must be at least 1");
            let combos = (wordlist.len() as f64).powi(args.depth as i32);
            eprintln!(
                "[..] force depth {}: ~{:.3e} combination(s)",
                args.depth, combos
            );
            found.extend(guesser.force(&wordlist, args.depth));
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
    eprintln!(
        "[ok] {} probe(s) -> {} candidate(s) for {} distinct hash(es)",
        guesser.probes.load(Ordering::Relaxed),
        unique.len(),
        hits.len()
    );
    if !unique.is_empty() {
        eprintln!(
            "[note] these are candidates, not cracks. A 32-bit hash collides \
             with plausible names by chance - confirm each against shipped data \
             before `hashtool add`"
        );
    }
    Ok(())
}
