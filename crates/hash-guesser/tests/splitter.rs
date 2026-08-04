//! The splitter, checked against the committed hashtables rather than a
//! handful of literals.
//!
//! `scripts/split_words.py` and `crates/hash-guesser/src/words.rs` implement
//! the same splitting rule in two languages, and a boundary they disagree about
//! is a word the guesser can never build. The unit tests in `words.rs` pin the
//! interesting shapes; this pins the property that has to hold for all 14000
//! names in the repo, which is the part a hand-written case list would miss.

use std::fs;
use std::path::PathBuf;

// Compiled into the test binary as its own copy of the module, so anything the
// tests below don't call reads as dead here while being live in the binary.
#[path = "../src/words.rs"]
#[allow(dead_code)]
mod words;

fn repo_root() -> PathBuf {
    // CARGO_MANIFEST_DIR is crates/hash-guesser.
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
}

fn names_from(rel: &str) -> Vec<String> {
    let path = repo_root().join(rel);
    let Ok(text) = fs::read_to_string(&path) else {
        return Vec::new();
    };
    text.lines()
        .filter_map(|l| l.trim().split_once(' ').map(|(_, n)| n.trim().to_string()))
        .filter(|n| !n.is_empty())
        .collect()
}

fn corpus() -> Vec<String> {
    let mut out = Vec::new();
    for rel in [
        "hashes/hashes.bintypes.txt",
        "hashes/hashes.binfields.txt",
        "hashes/overrides/bintypes.txt",
        "hashes/overrides/binfields.txt",
    ] {
        out.extend(names_from(rel));
    }
    assert!(!out.is_empty(), "no hashtables found under {:?}", repo_root());
    out
}

/// Nothing is invented and nothing is lost: concatenating the words gives the
/// name back, up to case and its separators.
///
/// This is what actually matters. A splitter that dropped a letter would still
/// produce plausible-looking words, and would silently make some names
/// unreachable; this catches that for every name in the repo.
#[test]
fn words_reconstruct_the_name() {
    for name in corpus() {
        // prefix_max = 0: the Hungarian-prefix rule deliberately drops a word,
        // so reconstruction is only total when it is switched off.
        let joined: String = words::split_words(&name, 0).join("");
        let expect: String = name
            .chars()
            .filter(|c| c.is_ascii_alphanumeric())
            .collect::<String>()
            .to_lowercase();
        assert_eq!(joined.to_lowercase(), expect, "splitting {name:?}");
    }
}

/// Every word is usable: non-empty, and alphanumeric so that a name built from
/// it can satisfy the PascalCase rule.
#[test]
fn words_are_well_formed() {
    for name in corpus() {
        for word in words::split_words(&name, 2) {
            assert!(!word.is_empty(), "empty word from {name:?}");
            assert!(
                word.chars().all(|c| c.is_ascii_alphanumeric()),
                "word {word:?} from {name:?} is not alphanumeric"
            );
        }
    }
}

/// A name that already satisfies the repo rule is rebuilt by the guesser
/// exactly as written - same bytes, not just the same letters.
///
/// This is the invariant the guesser's output depends on: it emits
/// prefix + concat(words), so if that round-trip were lossy for a PascalCase
/// name, the guesser could hit the right hash and print the wrong spelling.
#[test]
fn pascal_names_round_trip_exactly() {
    for name in corpus() {
        if !words::is_pascal(&name) {
            continue;
        }
        let joined: String = words::split_words(&name, 0).join("");
        assert_eq!(joined, name, "round-tripping {name:?}");
    }
}

/// A one-letter lowercase prefix costs the wordlist that letter and nothing else.
///
/// This is the claim `names.py` permits `mCoefficient` on, and it is worth
/// pinning against the corpus rather than a literal because the corpus is where
/// it has to hold: upstream spells 2000-odd of its fields this way, and if the
/// splitter lost anything past the prefix then every one of those names would be
/// contributing a truncated word and the exception would be costing us exactly
/// what it claims to save.
#[test]
fn a_one_letter_prefix_is_all_that_the_splitter_drops() {
    let mut checked = 0usize;
    for name in corpus() {
        let b = name.as_bytes();
        if !(b.len() >= 2 && b[0].is_ascii_lowercase() && b[1].is_ascii_uppercase()) {
            continue;
        }
        // The default prefix_max, so the heuristic is live and the prefix goes.
        let joined: String = words::split_words(&name, 2).join("");
        assert_eq!(joined, name[1..], "notation split of {name:?}");
        checked += 1;
    }
    // Guard against the filter quietly matching nothing, which would leave this
    // passing on an empty set and asserting nothing at all.
    assert!(checked > 1000, "only {checked} notation-prefixed names in the corpus");
}
