//! Compatibility tests over the committed dumps.
//!
//! Dumps are never regenerated, so every one of them has to stay readable by
//! this crate indefinitely. These tests guard against a schema change that
//! silently orphans historical files.

use lol_meta_schema::{MetaDump, FORMAT_VERSION_UNVERSIONED};
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};

fn dumps_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("dumps")
}

/// Every committed dump, oldest first by build number.
fn all_dumps() -> Vec<PathBuf> {
    let mut paths: Vec<PathBuf> = std::fs::read_dir(dumps_dir())
        .expect("dumps/ should exist")
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|e| e == "json"))
        .collect();

    // Sorted on the build number, the trailing component, so "oldest" and
    // "newest" mean what they say: patch strings do not sort lexicographically.
    paths.sort_by_key(|p| {
        p.file_stem()
            .and_then(|s| s.to_str())
            .and_then(|s| s.rsplit('.').next())
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(0)
    });
    paths
}

/// Panics rather than skipping when there is nothing to read - a compatibility
/// test that passes for want of input is worse than no test.
fn require_dumps() -> Vec<PathBuf> {
    let all = all_dumps();
    assert!(
        !all.is_empty(),
        "no dumps found in {}",
        dumps_dir().display()
    );
    all
}

fn load(path: &Path) -> MetaDump {
    let file = File::open(path).unwrap_or_else(|e| panic!("opening {}: {e}", path.display()));
    serde_json::from_reader(BufReader::new(file))
        .unwrap_or_else(|e| panic!("deserializing {}: {e}", path.display()))
}

/// The ends of the range plus a spread in between. Reading all 237 would be
/// hundreds of megabytes, and the ends are where a schema change bites.
fn sample() -> Vec<PathBuf> {
    let all = require_dumps();
    let mut picked = vec![all.first().unwrap().clone(), all.last().unwrap().clone()];
    let step = (all.len() / 8).max(1);
    picked.extend(all.iter().step_by(step).cloned());
    picked.sort();
    picked.dedup();
    picked
}

#[test]
fn every_sampled_dump_deserializes() {
    let sample = sample();
    for path in &sample {
        let dump = load(path);
        assert!(!dump.version.is_empty(), "{}: empty version", path.display());
        assert!(!dump.classes.is_empty(), "{}: no classes", path.display());
    }
    eprintln!("checked {} of {} dumps", sample.len(), all_dumps().len());
}

#[test]
fn the_oldest_and_newest_dumps_both_deserialize() {
    // Separate from the sample so a regression at either end names the file
    // instead of hiding inside a loop.
    let all = require_dumps();
    for path in [all.first().unwrap(), all.last().unwrap()] {
        let dump = load(path);
        assert!(!dump.classes.is_empty(), "{}: no classes", path.display());
    }
}

#[test]
fn dumps_written_before_the_version_field_read_as_unversioned() {
    // An absent `formatVersion` has to default rather than fail to parse, or
    // every dump committed to date becomes unreadable.
    let oldest = load(require_dumps().first().unwrap());
    assert_eq!(oldest.format_version, FORMAT_VERSION_UNVERSIONED);
}

#[test]
fn nested_structures_are_reachable() {
    let all = require_dumps();
    let dump = load(all.last().unwrap());

    // Walks the parts most likely to break: properties carry the container and
    // map sub-objects, and defaults is the only nullable branch.
    let mut saw_property = false;
    let mut saw_defaults = false;
    for class in dump.classes.values() {
        for property in class.properties.values() {
            let _ = property.value_type;
            saw_property = true;
        }
        if class.defaults.is_some() {
            saw_defaults = true;
        }
    }
    assert!(saw_property, "no properties in the newest dump");
    assert!(saw_defaults, "no class carried defaults");
}
