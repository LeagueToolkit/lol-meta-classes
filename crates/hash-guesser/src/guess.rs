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

use std::collections::HashSet;
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
        Self { text: text.to_string(), hash: fnv1a_cont(text.as_bytes(), FNV_OFFSET) }
    }
}

pub struct Found {
    pub hash: u32,
    pub name: String,
}

/// The hashes we want a name for, behind a bitmap prefilter.
///
/// The set is checked billions of times and hit essentially never, so the cost
/// that matters is the cost of *missing*. A 26-bit bitmap (8 MiB, stays in L2/L3
/// under the working set) answers that in one load and a test; the real hash
/// set is consulted only for the ~0.05% of probes the bitmap lets through.
pub struct Targets {
    bits: Vec<u64>,
    set: HashSet<u32>,
}

const FILTER_BITS: u32 = 26;
const FILTER_MASK: u32 = (1 << FILTER_BITS) - 1;

impl Targets {
    pub fn new(set: HashSet<u32>) -> Self {
        let mut bits = vec![0u64; (1usize << FILTER_BITS) / 64];
        for &h in &set {
            let i = (h & FILTER_MASK) as usize;
            bits[i >> 6] |= 1u64 << (i & 63);
        }
        Self { bits, set }
    }

    #[inline(always)]
    fn hit(&self, h: u32) -> bool {
        let i = (h & FILTER_MASK) as usize;
        (self.bits[i >> 6] & (1u64 << (i & 63))) != 0 && self.set.contains(&h)
    }
}

pub struct Guesser {
    pub targets: Targets,
    /// Names already tried and rejected, lowercased.
    pub bad: HashSet<String>,
    pub prefixes: Vec<Prefix>,
    pub probes: AtomicU64,
}

impl Guesser {
    /// Check one candidate, building its name only if the hash lands.
    #[inline(always)]
    fn check(&self, prefix: &Prefix, stack: &[&str], h: u32, out: &mut Vec<Found>) {
        if !self.targets.hit(h) {
            return;
        }
        let mut name = String::with_capacity(prefix.text.len() + stack.len() * 8);
        name.push_str(&prefix.text);
        for w in stack {
            name.push_str(w);
        }
        if self.bad.contains(&name.to_lowercase()) {
            return;
        }
        // Never emit a candidate the repo would refuse. With capitalized words
        // and a letter-only prefix this cannot fail, which is exactly why it is
        // worth asserting: it is the invariant that keeps the output feedable
        // straight into `hashtool add`.
        if !words::is_pascal(&name) {
            return;
        }
        out.push(Found { hash: h, name });
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
