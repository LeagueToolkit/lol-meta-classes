//! The search itself.
//!
//! Every mode is the same inner loop: extend a running FNV hash by one word,
//! and check it. What differs is which sequences of words get built.
//!
//! Two things make it fast enough to be worth running. FNV-1a is a left-to-right
//! fold, so a candidate's hash is its prefix's hash extended by the new word -
//! the work per candidate is the length of the word added, not of the whole
//! name. And a candidate's *string* is never built unless its hash hits, which
//! it almost never does; the hot path touches no allocator at all.

use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicU64, Ordering};

use rayon::prelude::*;

use crate::input::{fnv1a, fnv1a_cont, FNV_OFFSET};
use crate::words;

/// A leading string every candidate is tried with. `""` is always present;
/// `"I"` catches the interface classes (`IGameCalculationPart`), which are
/// otherwise a word the wordlist would have to carry as a false word.
///
/// The Hungarian prefixes the original carried here (`m`, `b`, `ar`, `m_`) are
/// gone: each of them produces a camelCase name, which this repo does not
/// accept. See `names.py`.
pub struct Prefix {
    pub text: String,
    pub hash: u32,
}

impl Prefix {
    pub fn new(text: &str) -> Self {
        Self {
            text: text.to_string(),
            hash: fnv1a_cont(text.as_bytes(), FNV_OFFSET),
        }
    }
}

pub struct Found {
    pub hash: u32,
    pub name: String,
}

/// The states the search is hunting for, behind a bitmap prefilter.
///
/// A *state* is what the running hash has to equal for a candidate to land. With
/// no suffix that is just the target hash. With a suffix it is the target folded
/// backwards through the suffix - so the search looks for the state the name
/// would be in *before* the suffix was appended, and the suffix costs the hot
/// loop nothing at all. One target with k suffixes is k states, each remembering
/// which target and which suffix it came from.
///
/// The map is consulted billions of times and hit essentially never, so the cost
/// that matters is the cost of *missing*. A 26-bit bitmap (8 MiB, stays in L2/L3
/// under the working set) answers that in one load and a test; the map itself is
/// touched only for the ~0.05% of probes the bitmap lets through.
pub struct Targets {
    bits: Vec<u64>,
    map: HashMap<u32, Vec<(u32, u16)>>,
}

const FILTER_BITS: u32 = 26;
const FILTER_MASK: u32 = (1 << FILTER_BITS) - 1;

impl Targets {
    /// From `(state, target hash, suffix index)` triples.
    pub fn new(entries: impl IntoIterator<Item = (u32, u32, u16)>) -> Self {
        let mut bits = vec![0u64; (1usize << FILTER_BITS) / 64];
        let mut map: HashMap<u32, Vec<(u32, u16)>> = HashMap::new();
        for (state, hash, sidx) in entries {
            let i = (state & FILTER_MASK) as usize;
            bits[i >> 6] |= 1u64 << (i & 63);
            map.entry(state).or_default().push((hash, sidx));
        }
        Self { bits, map }
    }

    #[inline(always)]
    fn hit(&self, state: u32) -> Option<&[(u32, u16)]> {
        let i = (state & FILTER_MASK) as usize;
        if self.bits[i >> 6] & (1u64 << (i & 63)) == 0 {
            return None;
        }
        self.map.get(&state).map(Vec::as_slice)
    }

    /// Distinct states, which is what the collision arithmetic actually runs on:
    /// expected false positives are `probes x states / 2^32`, so k suffixes cost
    /// k times the noise of none. Suffix anchoring buys coverage, not accuracy.
    pub fn states(&self) -> usize {
        self.map.len()
    }
}

pub struct Guesser {
    pub targets: Targets,
    /// Indexed by the suffix index carried in `Targets`. Always non-empty; the
    /// unanchored search is the single empty suffix.
    pub suffixes: Vec<String>,
    /// Names already tried and rejected, lowercased.
    pub bad: HashSet<String>,
    pub prefixes: Vec<Prefix>,
    /// Class hashes the meta flags as an interface. When set, a candidate whose
    /// finished name matches `I[A-Z]` is dropped unless its hash is in here.
    ///
    /// The only check in this tool that is evidence rather than arithmetic: the
    /// flag comes out of the dump, not out of the hash. Measured against names
    /// already cracked, `I[A-Z]` implies the flag in 125 of 127 cases.
    pub interfaces: Option<HashSet<u32>>,
    pub probes: AtomicU64,
}

impl Guesser {
    /// Check one candidate, building its name only if the state lands.
    ///
    /// Everything past the bitmap costs nothing in practice: by the time we get
    /// here the state has already matched, which is rare.
    #[inline(always)]
    fn check(&self, prefix: &Prefix, stack: &[&str], state: u32, out: &mut Vec<Found>) {
        let Some(hits) = self.targets.hit(state) else {
            return;
        };
        for &(hash, sidx) in hits {
            let suffix = self.suffixes[sidx as usize].as_str();
            let mut name =
                String::with_capacity(prefix.text.len() + stack.len() * 8 + suffix.len());
            name.push_str(&prefix.text);
            for w in stack {
                name.push_str(w);
            }
            name.push_str(suffix);
            // Never emit a candidate the repo would refuse. Words are
            // capitalized and suffixes are letters only, so the only way this
            // fires is a `--prefix` that opens lowercase: one letter is the
            // notation exception and passes, more is a name `hashtool add`
            // would reject and is dropped here rather than printed.
            if !words::is_valid_name(&name) {
                continue;
            }
            if let Some(ifaces) = &self.interfaces {
                if words::is_interface_named(&name) && !ifaces.contains(&hash) {
                    continue;
                }
            }
            if self.bad.contains(&name.to_lowercase()) {
                continue;
            }
            out.push(Found { hash, name });
        }
    }

    /// Walk one word sequence, reporting from `report_from` onward.
    ///
    /// The cutoff is what stops a mutation pass rediscovering the names it
    /// started from: the prefix of a mutated sentence that ends before the
    /// mutated position is just the original sentence, which is already known.
    fn walk(&self, prefix: &Prefix, seq: &[&str], report_from: usize, out: &mut Vec<Found>) {
        let mut h = prefix.hash;
        let mut probes = 0u64;
        for i in 0..seq.len() {
            h = fnv1a_cont(seq[i].as_bytes(), h);
            if i >= report_from {
                probes += 1;
                self.check(prefix, &seq[..=i], h, out);
            }
        }
        self.probes.fetch_add(probes, Ordering::Relaxed);
    }

    /// Every known name, tried verbatim against the targets.
    ///
    /// Cheap and worth doing first, because the tables are cross-checked here:
    /// a class name is often also a field name, and vice versa. Those cost one
    /// probe each and are the highest-yield guesses available.
    pub fn identity(&self, sentences: &[Vec<String>]) -> Vec<Found> {
        sentences
            .par_iter()
            .flat_map_iter(|s| {
                let seq: Vec<&str> = s.iter().map(String::as_str).collect();
                let mut out = Vec::new();
                for prefix in &self.prefixes {
                    self.walk(prefix, &seq, 0, &mut out);
                }
                out
            })
            .collect()
    }

    /// For every word and every known name: substitute the word at each
    /// position, and insert it at each position.
    ///
    /// This is the productive mode. Real names are near-misses of other real
    /// names - `AttackableUnitDefinition` next to `AttackableUnit`, an
    /// `...Override` beside every `...` - so the neighbourhood of a known name
    /// is where an unknown one is most likely to be.
    pub fn mutate(&self, sentences: &[Vec<String>], wordlist: &[String]) -> Vec<Found> {
        wordlist
            .par_iter()
            .flat_map_iter(|word| {
                let mut out = Vec::new();
                let mut seq: Vec<&str> = Vec::with_capacity(16);
                for sentence in sentences {
                    for i in 0..=sentence.len() {
                        // Insert `word` at i, keeping everything else.
                        seq.clear();
                        seq.extend(sentence[..i].iter().map(String::as_str));
                        seq.push(word);
                        seq.extend(sentence[i..].iter().map(String::as_str));
                        for prefix in &self.prefixes {
                            self.walk(prefix, &seq, i, &mut out);
                        }

                        // Substitute `word` for the one at i.
                        if i < sentence.len() {
                            seq.clear();
                            seq.extend(sentence[..i].iter().map(String::as_str));
                            seq.push(word);
                            seq.extend(sentence[i + 1..].iter().map(String::as_str));
                            for prefix in &self.prefixes {
                                self.walk(prefix, &seq, i, &mut out);
                            }
                        }
                    }
                }
                out
            })
            .collect()
    }

    /// Every known name with one word deleted.
    ///
    /// The third edit alongside `mutate`'s insert and substitute, split out as
    /// its own mode because its noise profile is nothing like theirs: no
    /// wordlist multiplies the sentence count, so the whole pass is ~10^5
    /// probes - identity-cheap, mutate-shaped. The shape is common in the
    /// corpus itself: 5521 of 13799 known names are another known name with one
    /// interior word removed.
    ///
    /// Deleting the *last* word is deliberately not reported: the result is a
    /// strict prefix of the sentence, and the identity pass - which checks
    /// every prefix of every sentence it walks - has already probed it.
    /// `report_from = i` makes that fall out on its own: the shortened
    /// sequence has no position at or past `i` left to report.
    pub fn delete(&self, sentences: &[Vec<String>]) -> Vec<Found> {
        sentences
            .par_iter()
            .flat_map_iter(|sentence| {
                let mut out = Vec::new();
                let mut seq: Vec<&str> = Vec::with_capacity(16);
                for i in 0..sentence.len() {
                    seq.clear();
                    seq.extend(sentence[..i].iter().map(String::as_str));
                    seq.extend(sentence[i + 1..].iter().map(String::as_str));
                    for prefix in &self.prefixes {
                        self.walk(prefix, &seq, i, &mut out);
                    }
                }
                out
            })
            .collect()
    }

    /// Every arrangement of up to `depth` words from the wordlist.
    ///
    /// Exhaustive and therefore explosive: cost is `len(wordlist)^depth`. Depth
    /// 2 over a few thousand words is seconds; depth 3 is hours and wants
    /// `--top` to cut the wordlist to the words that actually earn their place.
    /// Shorter names are covered on the way, since every prefix is checked.
    pub fn force(&self, wordlist: &[String], depth: usize) -> Vec<Found> {
        if depth == 0 {
            return Vec::new();
        }
        wordlist
            .par_iter()
            .flat_map_iter(|first| {
                let mut out = Vec::new();
                for prefix in &self.prefixes {
                    let mut stack: Vec<&str> = Vec::with_capacity(depth);
                    let h = fnv1a_cont(first.as_bytes(), prefix.hash);
                    stack.push(first);
                    self.probes.fetch_add(1, Ordering::Relaxed);
                    self.check(prefix, &stack, h, &mut out);
                    self.force_rec(prefix, &mut stack, h, wordlist, depth - 1, &mut out);
                    stack.pop();
                }
                out
            })
            .collect()
    }

    fn force_rec<'a>(
        &self,
        prefix: &Prefix,
        stack: &mut Vec<&'a str>,
        h: u32,
        wordlist: &'a [String],
        depth: usize,
        out: &mut Vec<Found>,
    ) {
        if depth == 0 {
            return;
        }
        // `h` is a u32 passed by value, so descending and backtracking costs
        // nothing to save or restore - the reason this is one search per prefix
        // rather than one carrying a vector of running hashes.
        let mut probes = 0u64;
        for word in wordlist {
            let h = fnv1a_cont(word.as_bytes(), h);
            stack.push(word);
            probes += 1;
            self.check(prefix, stack, h, out);
            self.force_rec(prefix, stack, h, wordlist, depth - 1, out);
            stack.pop();
        }
        self.probes.fetch_add(probes, Ordering::Relaxed);
    }

    /// `force`, constrained so a word may only follow a word it has been seen
    /// to follow in a known name.
    ///
    /// The wordlist is unigram - order and co-occurrence are not in it - so
    /// this is a prior `force` cannot express, and it is a prior on
    /// *generation*, not a filter on hits: fewer sequences get probed at all,
    /// and expected noise is `probes x states / 2^32`, so it falls with them.
    /// Attested pairs are 0.16% of all pairs over this corpus' 3113 words, so
    /// the collapse is large: depth 3 falls from 64M arrangements of the top 400
    /// words to ~400k chains, and depth 4 costs less than uniform depth 3 did,
    /// which is what buys the extra word.
    pub fn chain(&self, bigrams: &Bigrams, depth: usize) -> Vec<Found> {
        bigrams
            .roots
            .par_iter()
            .flat_map_iter(|&root| {
                let mut out = Vec::new();
                for prefix in &self.prefixes {
                    let word = bigrams.words[root as usize].as_str();
                    let mut stack: Vec<&str> = Vec::with_capacity(depth);
                    let h = fnv1a_cont(word.as_bytes(), prefix.hash);
                    stack.push(word);
                    self.probes.fetch_add(1, Ordering::Relaxed);
                    self.check(prefix, &stack, h, &mut out);
                    self.chain_rec(prefix, &mut stack, root, h, bigrams, depth - 1, &mut out);
                }
                out
            })
            .collect()
    }

    fn chain_rec<'a>(
        &self,
        prefix: &Prefix,
        stack: &mut Vec<&'a str>,
        last: u32,
        h: u32,
        bigrams: &'a Bigrams,
        depth: usize,
        out: &mut Vec<Found>,
    ) {
        if depth == 0 {
            return;
        }
        let mut probes = 0u64;
        for &next in &bigrams.succ[last as usize] {
            let word = bigrams.words[next as usize].as_str();
            let h = fnv1a_cont(word.as_bytes(), h);
            stack.push(word);
            probes += 1;
            self.check(prefix, stack, h, out);
            self.chain_rec(prefix, stack, next, h, bigrams, depth - 1, out);
            stack.pop();
        }
        self.probes.fetch_add(probes, Ordering::Relaxed);
    }
}

/// The word-adjacency graph of the known names, for `chain`.
///
/// Words are interned case-folded - the hash lowercases, so two spellings are
/// the same guess - and `succ[w]` lists every word seen immediately after `w`
/// in some known name. Roots are the wordlist, so `--top` truncates where the
/// search *starts*; successors are whatever the corpus attests, kept even when
/// `--top` dropped them, because the bigram is the evidence, not the word's
/// frequency rank.
pub struct Bigrams {
    words: Vec<String>,
    succ: Vec<Vec<u32>>,
    roots: Vec<u32>,
}

impl Bigrams {
    pub fn new(sentences: &[Vec<String>], wordlist: &[String]) -> Self {
        let mut index: HashMap<String, u32> = HashMap::new();
        let mut words: Vec<String> = Vec::new();
        let mut intern = |w: &str, words: &mut Vec<String>| -> u32 {
            let key = w.to_lowercase();
            if let Some(&i) = index.get(&key) {
                return i;
            }
            let i = words.len() as u32;
            index.insert(key, i);
            words.push(w.to_string());
            i
        };
        let roots: Vec<u32> = wordlist.iter().map(|w| intern(w, &mut words)).collect();
        let mut pairs: HashSet<(u32, u32)> = HashSet::new();
        for s in sentences {
            for w in s.windows(2) {
                let a = intern(&w[0], &mut words);
                let b = intern(&w[1], &mut words);
                pairs.insert((a, b));
            }
        }
        let mut succ = vec![Vec::new(); words.len()];
        // Sorted so the walk order - and with it the output and the probe
        // counter - is deterministic run to run.
        let mut pairs: Vec<_> = pairs.into_iter().collect();
        pairs.sort_unstable();
        for (a, b) in pairs {
            succ[a as usize].push(b);
        }
        Self { words, succ, roots }
    }

    /// Exactly how many probes one prefix's `chain` walk at this depth will
    /// make, by DP over the graph. Printed before the run starts, because the
    /// noise budget should be known before it is spent.
    pub fn probes(&self, depth: usize) -> u64 {
        // prev[w]: probes in the subtree rooted at w with d levels remaining.
        let mut prev = vec![1u64; self.words.len()];
        for _ in 2..=depth {
            prev = (0..self.words.len())
                .map(|w| 1 + self.succ[w].iter().map(|&v| prev[v as usize]).sum::<u64>())
                .collect();
        }
        self.roots.iter().map(|&r| prev[r as usize]).sum()
    }

    pub fn bigram_count(&self) -> usize {
        self.succ.iter().map(Vec::len).sum()
    }
}

/// Known names -> the word sequences the mutation pass works from.
///
/// Deduplicated on the joined form, because upstream and our overrides overlap
/// heavily and a sentence tried twice is work done twice.
pub fn sentences(names: impl IntoIterator<Item = String>, prefix_max: usize) -> Vec<Vec<String>> {
    let mut seen = HashSet::new();
    let mut out = Vec::new();
    for name in names {
        let split = words::split_words(&name, prefix_max);
        if split.is_empty() {
            continue;
        }
        if seen.insert(split.join("").to_lowercase()) {
            out.push(split);
        }
    }
    out
}

/// Sanity check on a found name, for the caller's own output: does it actually
/// hash to what we say it does?
pub fn verify(found: &Found) -> bool {
    fnv1a(&found.name) == found.hash
}

#[cfg(test)]
mod tests {
    use super::*;

    fn guesser(target_names: &[&str]) -> Guesser {
        let entries = target_names.iter().map(|n| (fnv1a(n), fnv1a(n), 0u16));
        Guesser {
            targets: Targets::new(entries),
            suffixes: vec![String::new()],
            bad: HashSet::new(),
            prefixes: vec![Prefix::new("")],
            interfaces: None,
            probes: AtomicU64::new(0),
        }
    }

    fn sents(names: &[&str]) -> Vec<Vec<String>> {
        sentences(names.iter().map(|s| s.to_string()), 2)
    }

    fn names(found: &[Found]) -> Vec<&str> {
        let mut v: Vec<&str> = found.iter().map(|f| f.name.as_str()).collect();
        v.sort_unstable();
        v.dedup();
        v
    }

    #[test]
    fn delete_removes_one_word_at_each_position() {
        let g = guesser(&["ColorOverDriver", "OverLifeDriver"]);
        let found = g.delete(&sents(&["ColorOverLifeDriver"]));
        // Delete "Life" -> ColorOverDriver; delete "Color" -> OverLifeDriver.
        assert_eq!(names(&found), ["ColorOverDriver", "OverLifeDriver"]);
        for f in &found {
            assert!(verify(f));
        }
    }

    #[test]
    fn delete_leaves_last_word_deletions_to_identity() {
        // ColorOverLife is the sentence minus its last word - a prefix, which
        // the identity pass already probes. Delete must not report it.
        let g = guesser(&["ColorOverLife"]);
        assert!(g.delete(&sents(&["ColorOverLifeDriver"])).is_empty());
        assert_eq!(
            names(&g.identity(&sents(&["ColorOverLifeDriver"]))),
            ["ColorOverLife"]
        );
    }

    #[test]
    fn chain_follows_only_attested_bigrams() {
        // Bigrams: Vfx->Color, Color->Driver (from the corpus). VfxColorDriver
        // chains; VfxDriverColor would need Vfx->Driver, never attested.
        let corpus = sents(&["VfxColor", "ColorDriver"]);
        let words = vec!["Vfx".to_string(), "Color".to_string(), "Driver".to_string()];
        let bigrams = Bigrams::new(&corpus, &words);
        let g = guesser(&["VfxColorDriver", "VfxDriverColor"]);
        let found = g.chain(&bigrams, 3);
        assert_eq!(names(&found), ["VfxColorDriver"]);
    }

    #[test]
    fn chain_probe_dp_matches_the_walk() {
        let corpus = sents(&["VfxColorOverLife", "ColorDriver", "OverLifeDriver"]);
        let words = vec!["Vfx".to_string(), "Color".to_string(), "Over".to_string()];
        let bigrams = Bigrams::new(&corpus, &words);
        for depth in 1..=5 {
            let g = guesser(&["NoSuchName"]);
            g.chain(&bigrams, depth);
            assert_eq!(
                g.probes.load(Ordering::Relaxed),
                bigrams.probes(depth),
                "depth {depth}"
            );
        }
    }

    #[test]
    fn chain_successors_survive_top_truncation() {
        // "Driver" is not a root (truncated wordlist), but the Color->Driver
        // bigram still extends a chain through it.
        let corpus = sents(&["VfxColor", "ColorDriver"]);
        let words = vec!["Vfx".to_string()];
        let bigrams = Bigrams::new(&corpus, &words);
        let g = guesser(&["VfxColorDriver"]);
        assert_eq!(names(&g.chain(&bigrams, 3)), ["VfxColorDriver"]);
    }
}
