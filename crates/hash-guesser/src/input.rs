//! Loading the four inputs: what to crack, what is already known, what has
//! been ruled out, and the words to build with.

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use rayon::prelude::*;
use serde::Deserialize;

use crate::words;

pub const FNV_OFFSET: u32 = 0x811c_9dc5;
const FNV_PRIME: u32 = 0x0100_0193;

/// FNV-1a over the lowercased bytes, continued from `h`.
///
/// Continuable because that is the whole performance story: a name is built one
/// word at a time, and every prefix of it has already been hashed. Rehashing
/// the full string at each step would make the force mode quadratic in name
/// length for no reason.
#[inline]
pub fn fnv1a_cont(data: &[u8], mut h: u32) -> u32 {
    for &c in data {
        let c = if c.is_ascii_uppercase() { c + 32 } else { c };
        h ^= c as u32;
        h = h.wrapping_mul(FNV_PRIME);
    }
    h
}

#[inline]
pub fn fnv1a(name: &str) -> u32 {
    fnv1a_cont(name.as_bytes(), FNV_OFFSET)
}

fn parse_hash(text: &str) -> Option<u32> {
    let text = text.strip_prefix("0x").unwrap_or(text);
    u32::from_str_radix(text, 16).ok()
}

/// A `{hash} {name}` table -> hash -> name. Malformed lines are skipped rather
/// than fatal: these files are also read by the Python tooling, which validates
/// them properly, and a guesser that refuses to start over one bad line in a
/// 9000-line vendored mirror is a guesser nobody runs.
pub fn read_table(path: &Path) -> Result<HashMap<u32, String>> {
    let text = fs::read_to_string(path)
        .with_context(|| format!("reading table {}", path.display()))?;
    let mut out = HashMap::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((h, name)) = line.split_once(' ') {
            if let Some(h) = parse_hash(h) {
                out.insert(h, name.trim().to_string());
            }
        }
    }
    Ok(out)
}

/// A bare list of hashes, one per line. A trailing name is ignored, so the
/// output of a previous run - or a slice of an override table - can be fed
/// straight back in as a target list.
pub fn read_hash_list(path: &Path) -> Result<HashSet<u32>> {
    let text = fs::read_to_string(path)
        .with_context(|| format!("reading hash list {}", path.display()))?;
    let mut out = HashSet::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let first = line.split_whitespace().next().unwrap_or("");
        match parse_hash(first) {
            Some(h) => {
                out.insert(h);
            }
            None => eprintln!("[warn] {}: not a hash: {line:?}", path.display()),
        }
    }
    anyhow::ensure!(!out.is_empty(), "hash list {} is empty", path.display());
    Ok(out)
}

/// Names already ruled out, from `{hash} {name}` files or directories of them.
/// Held lowercased: a name rejected in one casing is rejected in every casing,
/// since they are all the same hash.
pub fn read_bad(paths: &[PathBuf]) -> Result<HashSet<String>> {
    let mut files = Vec::new();
    for path in paths {
        if path.is_dir() {
            for entry in fs::read_dir(path)
                .with_context(|| format!("reading bad-list dir {}", path.display()))?
            {
                let entry = entry?;
                if entry.file_type()?.is_file() {
                    files.push(entry.path());
                }
            }
        } else if path.exists() {
            files.push(path.clone());
        } else {
            eprintln!("[warn] no bad list at {}", path.display());
        }
    }
    let mut out = HashSet::new();
    for f in &files {
        for name in read_table(f)?.into_values() {
            out.insert(name.to_lowercase());
        }
    }
    Ok(out)
}

/// One dump, read for hashes only. Everything but the keys is `IgnoredAny`, so
/// this does not have to track the dump schema as it changes - only that
/// classes are keyed by hash and carry properties keyed by hash.
#[derive(Deserialize)]
struct Dump {
    classes: HashMap<String, DumpClass>,
}

/// Both maps are `Option`, not `#[serde(default)]` alone: the dumper writes an
/// explicit `null` for a class with no defaults rather than omitting the key
/// (187 of 2222 classes in 13.19.5329749), and `default` only covers a *missing*
/// field. Treating null as "no hashes here" is right either way.
#[derive(Deserialize)]
struct DumpClass {
    #[serde(default)]
    properties: Option<HashMap<String, serde::de::IgnoredAny>>,
    #[serde(default)]
    defaults: Option<HashMap<String, serde::de::IgnoredAny>>,
    #[serde(default)]
    is: Option<DumpFlags>,
}

#[derive(Deserialize)]
struct DumpFlags {
    #[serde(default)]
    interface: bool,
}

/// What one sweep of `dumps/` yields.
pub struct DumpHashes {
    pub types: HashSet<u32>,
    pub fields: HashSet<u32>,
    /// Class hashes the meta marks as an interface, in any build. This is the
    /// one piece of evidence about a name that does not come from the hash, so
    /// it is the only thing here that can genuinely refute a candidate: a
    /// guess of `IFoo` for a class the meta says is not an interface is wrong
    /// almost every time. Measured against names already cracked, `I[A-Z]`
    /// implies the interface flag in 125 of 127 cases.
    pub interfaces: HashSet<u32>,
}

/// Every class and property hash the game has actually used, across every
/// committed dump.
///
/// This is the target set, and taking it from `dumps/` rather than a checked-in
/// `all.*.txt` is the point: it is current by construction, it covers the whole
/// history rather than one build, and a hash that no longer appears in any dump
/// is still worth a name. `defaults` is read alongside `properties` because a
/// property can appear there and nowhere else.
pub fn read_dump_hashes(dumps: &Path) -> Result<DumpHashes> {
    let mut files: Vec<PathBuf> = fs::read_dir(dumps)
        .with_context(|| format!("reading dumps dir {}", dumps.display()))?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|e| e == "json"))
        .collect();
    files.sort();
    anyhow::ensure!(!files.is_empty(), "no dumps in {}", dumps.display());

    let per_file: Vec<DumpHashes> = files
        .par_iter()
        .map(|path| -> Result<DumpHashes> {
            let text = fs::read_to_string(path)
                .with_context(|| format!("reading dump {}", path.display()))?;
            let dump: Dump = serde_json::from_str(&text)
                .with_context(|| format!("parsing dump {}", path.display()))?;
            let mut out = DumpHashes {
                types: HashSet::new(),
                fields: HashSet::new(),
                interfaces: HashSet::new(),
            };
            for (key, class) in &dump.classes {
                if let Some(h) = parse_hash(key) {
                    out.types.insert(h);
                    if class.is.as_ref().is_some_and(|f| f.interface) {
                        out.interfaces.insert(h);
                    }
                }
                let keys = class
                    .properties
                    .iter()
                    .flat_map(|m| m.keys())
                    .chain(class.defaults.iter().flat_map(|m| m.keys()));
                for key in keys {
                    if let Some(h) = parse_hash(key) {
                        out.fields.insert(h);
                    }
                }
            }
            Ok(out)
        })
        .collect::<Result<Vec<_>>>()?;

    // Union across builds, and `interfaces` unions too: a class flagged an
    // interface in any build is one we should accept an I-name for, since the
    // name has to be right for the build it was seen in.
    let mut all = DumpHashes {
        types: HashSet::new(),
        fields: HashSet::new(),
        interfaces: HashSet::new(),
    };
    for d in per_file {
        all.types.extend(d.types);
        all.fields.extend(d.fields);
        all.interfaces.extend(d.interfaces);
    }
    Ok(all)
}

/// A wordlist, as `split_words.py` writes it. `--count` output is accepted too,
/// so a counted list can be fed straight back in without stripping the counts.
///
/// Entries that could not head a PascalCase name are dropped with a warning -
/// they would only produce candidates `hashtool add` refuses.
pub fn read_wordlist(path: &Path) -> Result<Vec<String>> {
    let text = fs::read_to_string(path)
        .with_context(|| format!("reading wordlist {}", path.display()))?;
    let mut out = Vec::new();
    let mut seen = HashSet::new();
    let mut dropped = 0usize;
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let word = line.rsplit(char::is_whitespace).next().unwrap_or(line);
        if !words::is_usable_word(word) {
            dropped += 1;
            continue;
        }
        let word = words::capitalize(word);
        // Case-folded: two spellings are the same guess, because the hash
        // lowercases. Keeping both would just repeat the work.
        if seen.insert(word.to_lowercase()) {
            out.push(word);
        }
    }
    if dropped > 0 {
        eprintln!(
            "[warn] {}: dropped {dropped} entr(ies) that cannot start a \
             PascalCase name",
            path.display()
        );
    }
    anyhow::ensure!(!out.is_empty(), "wordlist {} is empty", path.display());
    Ok(out)
}
