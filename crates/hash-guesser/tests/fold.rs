//! The forward and backward folds, checked against the committed hashtables.
//!
//! `fnv1a_back` is what makes `--suffix` free, and it is the kind of code that
//! is either exactly right or silently produces a search that finds nothing. So
//! it is pinned against real data rather than literals: every name in the repo,
//! split at every position, has to fold forward to its hash and backward from
//! it.

use std::fs;
use std::path::PathBuf;

// Its own copy of the module, as in splitter.rs.
#[path = "../src/fold.rs"]
mod fold;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
}

/// `(hash, name)` from every table the repo carries.
fn corpus() -> Vec<(u32, String)> {
    let mut out = Vec::new();
    for rel in [
        "hashes/hashes.bintypes.txt",
        "hashes/hashes.binfields.txt",
        "hashes/overrides/bintypes.txt",
        "hashes/overrides/binfields.txt",
    ] {
        let Ok(text) = fs::read_to_string(repo_root().join(rel)) else {
            continue;
        };
        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some((h, name)) = line.split_once(' ') {
                if let Ok(h) = u32::from_str_radix(h.strip_prefix("0x").unwrap_or(h), 16) {
                    out.push((h, name.trim().to_string()));
                }
            }
        }
    }
    assert!(out.len() > 10_000, "corpus looks empty: {}", out.len());
    out
}

#[test]
fn forward_fold_reproduces_every_table_entry() {
    for (h, name) in corpus() {
        assert_eq!(fold::fnv1a(&name), h, "{name} does not hash to {h:08x}");
    }
}

#[test]
fn backward_fold_inverts_forward_fold_at_every_split() {
    // Only ASCII names split safely on byte indices, and every name in these
    // tables is ASCII - assert that rather than assume it.
    for (h, name) in corpus() {
        assert!(name.is_ascii(), "non-ascii name {name}");
        for i in 0..=name.len() {
            let (pre, suf) = name.split_at(i);
            assert_eq!(
                fold::fnv1a_back(suf.as_bytes(), h),
                fold::fnv1a(pre),
                "{name}: back from {h:08x} through {suf:?} != forward of {pre:?}"
            );
        }
    }
}

#[test]
fn backward_fold_is_case_insensitive_like_the_forward_one() {
    // The suffix a caller passes is presentation; the state it folds to must not
    // depend on how they cased it, or `--suffix data` and `--suffix Data` would
    // search different things.
    for suffix in ["Data", "Controller", "Definition", "Vfx", "UI"] {
        let h = fold::fnv1a("SomeClassName");
        assert_eq!(
            fold::fnv1a_back(suffix.as_bytes(), h),
            fold::fnv1a_back(suffix.to_lowercase().as_bytes(), h)
        );
        assert_eq!(
            fold::fnv1a_back(suffix.as_bytes(), h),
            fold::fnv1a_back(suffix.to_uppercase().as_bytes(), h)
        );
    }
}

#[test]
fn empty_suffix_is_the_identity() {
    // What makes the unanchored search a special case of the anchored one rather
    // than a separate code path.
    for (h, _) in corpus().into_iter().take(500) {
        assert_eq!(fold::fnv1a_back(b"", h), h);
    }
}
