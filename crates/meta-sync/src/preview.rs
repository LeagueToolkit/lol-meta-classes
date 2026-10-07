//! PBE pass: keeps the newest PBE build as the only dump in `dumps/pbe/`.
//!
//! `scripts/db_build.py` folds that dump on top of the live history and writes
//! the difference to `db/meta.pbe.json`. A PBE dump is kept only while its
//! patch is greater than the latest live patch.
//!
//! The newest build is resolved through sieve. The manifest archive lags sieve
//! by up to a day and drops a build that was superseded inside one snapshot
//! window.
//!
//! The dump in `dumps/pbe/` is reduced: see [`reduce_dump`]. The full dump stays
//! in the temporary directory, and [`Report::full_dump`] holds its path.

use crate::config::Config;
use crate::dumper;
use crate::error::{Result, SyncError};
use crate::manifest;
use meta_sync::sieve;
use semver::Version;
use serde_json::Value;
use std::io::Write;
use std::path::{Path, PathBuf};

/// Region that the PBE pass reads.
pub const PREVIEW_REGION: &str = "PBE1";

/// Size above which a dump is rejected as corrupted.
const MAX_DUMP_BYTES: u64 = 50 * 1024 * 1024;

/// Class count below which a dump is rejected as truncated.
const MIN_DUMP_CLASSES: usize = 1000;

/// Class fields that a reduced dump drops.
const REDUCED_CLASS_FIELDS: &[&str] = &["fn", "size", "alignment", "secondary_children"];

/// Property fields that a reduced dump drops.
const REDUCED_PROPERTY_FIELDS: &[&str] = &["offset", "bitmask", "unkptr"];

/// Fields that a reduced dump drops from the `container` and `map` objects of
/// a property.
const REDUCED_STORAGE_FIELDS: &[&str] = &["vtable", "storage", "value_size"];

/// Major and minor version of a build.
type Patch = (u64, u64);

/// What the PBE pass does with the newest PBE build.
#[derive(Debug, PartialEq)]
enum Decision {
    /// The patch of the build is not greater than the latest live patch.
    NotAhead,
    /// `dumps/pbe/` already holds the build or a newer build.
    UpToDate,
    /// The build is dumped and replaces the dumps in `dumps/pbe/`.
    Dump,
}

/// How the PBE pass runs.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Mode {
    /// Reports what the pass would do. No file is removed or written and no
    /// binary is downloaded.
    Plan,
    /// Applies the changes to `dumps/pbe/`.
    Apply,
}

/// What a PBE pass did, or in [`Mode::Plan`] what it would do.
///
/// A pass that fails fills the report up to the failure.
#[derive(Debug, Default, PartialEq)]
pub struct Report {
    /// Newest PBE build that sieve lists.
    pub newest: Option<Version>,
    /// Versions of the PBE dumps that the pass removes from `dumps/pbe/`.
    pub removed: Vec<Version>,
    /// Version of the PBE dump that the pass writes to `dumps/pbe/`.
    pub dumped: Option<Version>,
    /// Path of the full dump of [`Report::dumped`]. `None` in [`Mode::Plan`].
    pub full_dump: Option<PathBuf>,
}

impl Report {
    /// Returns `true` if the pass adds or removes a file in `dumps/pbe/`.
    pub fn changed(&self) -> bool {
        self.dumped.is_some() || !self.removed.is_empty()
    }

    /// Returns the report as step outputs.
    ///
    /// `needs_dumper` is `true` if the pass runs the dumper. `version`,
    /// `newest` and `full_dump` are empty if the report holds no value for
    /// them. `removed` separates versions with a space.
    pub fn outputs(&self) -> Vec<(&'static str, String)> {
        let version = |v: &Option<Version>| v.as_ref().map(Version::to_string).unwrap_or_default();
        let removed: Vec<String> = self.removed.iter().map(Version::to_string).collect();
        vec![
            ("changed", self.changed().to_string()),
            ("needs_dumper", self.dumped.is_some().to_string()),
            ("newest", version(&self.newest)),
            ("version", version(&self.dumped)),
            ("removed", removed.join(" ")),
            (
                "full_dump",
                self.full_dump
                    .as_ref()
                    .map(|p| p.display().to_string())
                    .unwrap_or_default(),
            ),
        ]
    }
}

/// Runs the PBE pass and records what it does in `report`.
///
/// Removes every PBE dump whose patch is not greater than the latest live
/// patch, and every PBE dump except the newest. Dumps the newest PBE build if
/// its patch is greater than the latest live patch and `dumps/pbe/` holds no
/// dump of that build or of a newer build. If the lookup, the download, the
/// dumper or the validation fails, returns the error and leaves the remaining
/// PBE dump in place.
///
/// In [`Mode::Plan`] the pass changes no file and downloads no binary.
pub async fn run(config: &Config, mode: Mode, report: &mut Report) -> Result<()> {
    let preview_dir = config.preview_dumps_dir();
    let live = latest_patch(&dump_versions(&config.dumps_dir)?);
    match live {
        Some((major, minor)) => println!("✓ Latest live patch: {}.{}", major, minor),
        None => println!("⚠️  No live dump found in {}", config.dumps_dir.display()),
    }

    let existing = dump_versions(&preview_dir)?;
    let stale = stale_dumps(&existing, live);
    for version in &stale {
        println!("🧹 PBE dump {} - live has reached its patch", version);
    }
    let existing: Vec<Version> = existing
        .into_iter()
        .filter(|v| !stale.contains(v))
        .collect();

    // Two sync PRs that merge out of order leave two dumps in the directory.
    let superseded = superseded_dumps(&existing);
    for version in &superseded {
        println!("🧹 PBE dump {} - a newer PBE dump exists", version);
    }
    let existing: Vec<Version> = existing.into_iter().max().into_iter().collect();

    let obsolete: Vec<Version> = stale.into_iter().chain(superseded).collect();
    if mode == Mode::Apply {
        remove_dumps(&preview_dir, &obsolete)?;
    }
    report.removed.extend(obsolete);

    let release = sieve::fetch_sieve_latest(PREVIEW_REGION).await?;
    println!("✓ {} is serving {}", PREVIEW_REGION, release.version);
    let newest = Version::parse(&release.version)
        .map_err(|_| SyncError::InvalidVersion(release.version.clone()))?;
    report.newest = Some(newest.clone());

    match decide(&newest, &existing, live) {
        Decision::NotAhead => {
            println!(
                "⏭️  Skipping {} - not ahead of the latest live patch",
                newest
            );
            return Ok(());
        }
        Decision::UpToDate => {
            println!("⏭️  Skipping {} - already dumped", newest);
            return Ok(());
        }
        Decision::Dump => {}
    }

    if mode == Mode::Plan {
        println!("🎮 {} is not dumped", newest);
        report.removed.extend(existing);
        report.dumped = Some(newest);
        return Ok(());
    }

    // Checked before the download, which is the slow step.
    if !config.dumper_path.exists() {
        return Err(SyncError::DumperNotFound {
            path: config.dumper_path.clone(),
        });
    }

    println!("\n🎮 Processing PBE version: {}", newest);
    println!("────────────────────────────────────────");

    let temp_binary_path = config.temp_binary_path(&release.version);
    manifest::download_game_binary(&release.manifest_url, &temp_binary_path).await?;

    // The dumper writes to the temporary directory, so a failed or rejected
    // dump never replaces the dump that `dumps/pbe/` holds.
    let temp_dir = config.temp_dir.join("pbe");
    let full_dump_path = dump_path(&temp_dir, &newest);
    dumper::execute_dumper(config, &temp_binary_path, &full_dump_path)?;
    dumper::cleanup_temp_file(&temp_binary_path)?;
    let mut dump = validate_dump(&full_dump_path)?;

    reduce_dump(&mut dump);
    let reduced_dump_path = temp_dir.join(format!("{}.reduced.json", newest));
    write_reduced_dump(&reduced_dump_path, &dump)?;

    let output_path = replace_dumps(&preview_dir, &existing, &newest, &reduced_dump_path)?;
    println!("✅ Wrote {}", output_path.display());
    println!("   Full dump: {}", full_dump_path.display());

    report.removed.extend(existing);
    report.dumped = Some(newest);
    report.full_dump = Some(full_dump_path);

    Ok(())
}

/// Returns the versions of the dumps directly inside `dir`.
///
/// A dump is a file named `<major>.<minor>.<build>.json`. Other entries are
/// skipped. Returns an empty list if `dir` does not exist.
fn dump_versions(dir: &Path) -> std::io::Result<Vec<Version>> {
    let entries = match std::fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => return Err(e),
    };

    let mut versions = Vec::new();
    for entry in entries {
        let entry = entry?;
        if !entry.file_type()?.is_file() {
            continue;
        }
        let name = entry.file_name();
        let Some(version) = name
            .to_str()
            .and_then(|name| name.strip_suffix(".json"))
            .and_then(|stem| Version::parse(stem).ok())
        else {
            continue;
        };
        versions.push(version);
    }
    versions.sort();
    Ok(versions)
}

/// Returns the greatest patch among `versions`, or `None` if `versions` is empty.
fn latest_patch(versions: &[Version]) -> Option<Patch> {
    versions.iter().map(|v| (v.major, v.minor)).max()
}

/// Returns `true` if the patch of `version` is greater than `live`.
///
/// Returns `true` if `live` is `None`.
fn is_ahead(version: &Version, live: Option<Patch>) -> bool {
    live.is_none_or(|live| (version.major, version.minor) > live)
}

/// Returns the versions in `existing` whose patch is not greater than `live`.
fn stale_dumps(existing: &[Version], live: Option<Patch>) -> Vec<Version> {
    existing
        .iter()
        .filter(|v| !is_ahead(v, live))
        .cloned()
        .collect()
}

/// Returns every version in `existing` except the greatest.
fn superseded_dumps(existing: &[Version]) -> Vec<Version> {
    let newest = existing.iter().max();
    existing
        .iter()
        .filter(|v| Some(*v) != newest)
        .cloned()
        .collect()
}

/// Decides what the PBE pass does with `newest`, the newest PBE build.
///
/// `existing` holds the versions of the dumps in `dumps/pbe/` and `live` is the
/// latest live patch.
fn decide(newest: &Version, existing: &[Version], live: Option<Patch>) -> Decision {
    if !is_ahead(newest, live) {
        Decision::NotAhead
    } else if existing.iter().any(|v| v >= newest) {
        Decision::UpToDate
    } else {
        Decision::Dump
    }
}

/// Returns the path of the dump of `version` inside `dir`.
fn dump_path(dir: &Path, version: &Version) -> PathBuf {
    dir.join(format!("{}.json", version))
}

/// Removes the dumps of `versions` from `dir`.
fn remove_dumps(dir: &Path, versions: &[Version]) -> std::io::Result<()> {
    for version in versions {
        std::fs::remove_file(dump_path(dir, version))?;
    }
    Ok(())
}

/// Moves the dump at `new_dump` into `dir` as the dump of `version`, then
/// removes the dumps of `existing` from `dir`. Returns the path of the moved
/// dump.
fn replace_dumps(
    dir: &Path,
    existing: &[Version],
    version: &Version,
    new_dump: &Path,
) -> std::io::Result<PathBuf> {
    std::fs::create_dir_all(dir)?;
    let output_path = dump_path(dir, version);
    // `rename` fails if the temporary directory is on another filesystem.
    if std::fs::rename(new_dump, &output_path).is_err() {
        std::fs::copy(new_dump, &output_path)?;
        std::fs::remove_file(new_dump)?;
    }
    remove_dumps(dir, existing)?;
    Ok(output_path)
}

/// Reads the dump at `path` and checks that it is plausible.
///
/// Fails if the file is larger than [`MAX_DUMP_BYTES`], is not JSON, or holds
/// fewer than [`MIN_DUMP_CLASSES`] classes.
fn validate_dump(path: &Path) -> Result<Value> {
    let invalid = |reason: String| SyncError::InvalidDump {
        path: path.to_path_buf(),
        reason,
    };

    let size = std::fs::metadata(path)?.len();
    if size > MAX_DUMP_BYTES {
        return Err(invalid(format!(
            "{} bytes exceeds the limit of {} bytes",
            size, MAX_DUMP_BYTES
        )));
    }

    let dump: Value = serde_json::from_reader(std::io::BufReader::new(std::fs::File::open(path)?))
        .map_err(|e| invalid(format!("not JSON: {}", e)))?;

    let classes = dump["classes"].as_object().map_or(0, |c| c.len());
    if classes < MIN_DUMP_CLASSES {
        return Err(invalid(format!(
            "{} classes is below the minimum of {}",
            classes, MIN_DUMP_CLASSES
        )));
    }

    println!("  ✓ Dump holds {} classes ({} bytes)", classes, size);
    Ok(dump)
}

/// Removes from `dump` the layout fields that the database build does not read.
///
/// The removed fields are function addresses, class size and alignment,
/// `secondary_children`, property offsets and bitmasks, and the vtable,
/// storage kind and element size of containers and maps. Every other field is
/// kept, including fields that this function does not know. Sets the top-level
/// `reduced` field to `true`.
fn reduce_dump(dump: &mut Value) {
    fn remove_fields(value: &mut Value, fields: &[&str]) {
        if let Some(object) = value.as_object_mut() {
            for field in fields {
                object.remove(*field);
            }
        }
    }

    let classes = dump["classes"].as_object_mut();
    for class in classes.into_iter().flat_map(|c| c.values_mut()) {
        remove_fields(class, REDUCED_CLASS_FIELDS);
        let properties = class.get_mut("properties").and_then(Value::as_object_mut);
        for property in properties.into_iter().flat_map(|p| p.values_mut()) {
            remove_fields(property, REDUCED_PROPERTY_FIELDS);
            for storage in ["container", "map"] {
                if let Some(storage) = property.get_mut(storage) {
                    remove_fields(storage, REDUCED_STORAGE_FIELDS);
                }
            }
        }
    }

    if let Some(object) = dump.as_object_mut() {
        object.insert("reduced".to_string(), Value::Bool(true));
    }
}

/// Writes `dump` to `path` with one top-level field per line and one class per
/// line, so that two builds diff by class.
///
/// Classes are ordered by hash. A class key that is not a hexadecimal hash
/// sorts after the hashes.
fn write_reduced_dump(path: &Path, dump: &Value) -> Result<()> {
    let invalid = |reason: &str| SyncError::InvalidDump {
        path: path.to_path_buf(),
        reason: reason.to_string(),
    };
    let top = dump.as_object().ok_or_else(|| invalid("not an object"))?;
    let classes = dump["classes"]
        .as_object()
        .ok_or_else(|| invalid("`classes` is not an object"))?;

    let mut class_keys: Vec<&String> = classes.keys().collect();
    class_keys.sort_by_key(|key| {
        let hash = key
            .strip_prefix("0x")
            .and_then(|hex| u64::from_str_radix(hex, 16).ok());
        (hash.is_none(), hash, *key)
    });

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut out = std::io::BufWriter::new(std::fs::File::create(path)?);
    let json = |value: &Value| serde_json::to_string(value).expect("a Value serializes");

    writeln!(out, "{{")?;
    for (key, value) in top.iter().filter(|(key, _)| *key != "classes") {
        writeln!(
            out,
            "{}: {},",
            json(&Value::String(key.clone())),
            json(value)
        )?;
    }
    writeln!(out, "\"classes\": {{")?;
    for (index, key) in class_keys.iter().enumerate() {
        let comma = if index + 1 < class_keys.len() {
            ","
        } else {
            ""
        };
        writeln!(
            out,
            "{}: {}{}",
            json(&Value::String((*key).clone())),
            json(&classes[*key]),
            comma
        )?;
    }
    writeln!(out, "}}")?;
    writeln!(out, "}}")?;
    out.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::fs;

    fn v(version: &str) -> Version {
        Version::parse(version).unwrap()
    }

    fn touch(dir: &Path, name: &str) {
        fs::create_dir_all(dir).unwrap();
        fs::write(dir.join(name), b"{}").unwrap();
    }

    fn write_dump(path: &Path, classes: usize) {
        let classes: serde_json::Map<String, Value> = (0..classes)
            .map(|i| (format!("{:#x}", i), json!({})))
            .collect();
        fs::write(path, json!({ "classes": classes }).to_string()).unwrap();
    }

    /// A dump with the field set that the dumper writes for one class.
    fn full_dump() -> Value {
        json!({
            "formatVersion": 3,
            "version": "16.21.8255794",
            "hashers": {
                "0x22f54c0": {
                    "storage_width": 4,
                    "hash_function": {"algorithm": "Fnv1a32", "lowercased": true}
                }
            },
            "classes": {
                "0x1003c990": {
                    "base": "0x91873e8a",
                    "secondary_bases": {"0xb0607142": 16},
                    "secondary_children": {"0x2": 8},
                    "size": 32,
                    "alignment": 8,
                    "is": {"interface": false, "value": true},
                    "fn": {"constructor": "0x3b2f20"},
                    "properties": {
                        "0xd4db947d": {
                            "other_class": "0xb84d0322",
                            "offset": 168,
                            "bitmask": 0,
                            "value_type": "List2",
                            "container": {
                                "vtable": "0x23efa40",
                                "value_type": "Embed",
                                "value_size": 56,
                                "fixed_size": null,
                                "storage": "UnknownVector"
                            },
                            "map": null,
                            "hasher": null,
                            "unkptr": "0x0"
                        },
                        "0xf0a363e3": {
                            "other_class": null,
                            "offset": 152,
                            "bitmask": 0,
                            "value_type": "Map",
                            "container": null,
                            "map": {
                                "vtable": "0x23ef948",
                                "key_type": "Hash",
                                "value_type": "Link",
                                "storage": "UnknownMap"
                            },
                            "hasher": "0x22f54c0",
                            "unkptr": "0x0"
                        }
                    },
                    "defaults": {"0xd4db947d": [], "0xf0a363e3": {}}
                }
            }
        })
    }

    #[test]
    fn dump_versions_returns_empty_if_directory_is_missing() {
        let tmp = tempfile::tempdir().unwrap();
        assert_eq!(dump_versions(&tmp.path().join("pbe")).unwrap(), []);
    }

    #[test]
    fn dump_versions_skips_entries_that_are_not_dumps() {
        let tmp = tempfile::tempdir().unwrap();
        touch(tmp.path(), "16.19.8230722.json");
        touch(tmp.path(), "16.9.7728292.json");
        touch(tmp.path(), "README.md");
        touch(tmp.path(), "notes.json");
        // The PBE directory sits inside the live directory and is not a dump.
        touch(&tmp.path().join("pbe"), "16.21.8255794.json");
        assert_eq!(
            dump_versions(tmp.path()).unwrap(),
            [v("16.9.7728292"), v("16.19.8230722")]
        );
    }

    #[test]
    fn latest_patch_returns_greatest_patch_by_number() {
        // As strings "16.9" sorts above "16.19".
        let versions = [v("16.9.7728292"), v("16.19.8230722"), v("15.24.7347485")];
        assert_eq!(latest_patch(&versions), Some((16, 19)));
        assert_eq!(latest_patch(&[]), None);
    }

    #[test]
    fn is_ahead_returns_false_if_patch_equals_live_patch() {
        assert!(!is_ahead(&v("16.19.9999999"), Some((16, 19))));
        assert!(!is_ahead(&v("16.18.8175716"), Some((16, 19))));
        assert!(is_ahead(&v("16.20.8245011"), Some((16, 19))));
        assert!(is_ahead(&v("17.1.1"), Some((16, 24))));
    }

    #[test]
    fn is_ahead_returns_true_if_no_live_patch_exists() {
        assert!(is_ahead(&v("16.21.8255794"), None));
    }

    #[test]
    fn stale_dumps_returns_dumps_whose_patch_live_has_reached() {
        let existing = [v("16.20.8245011"), v("16.21.8255794")];
        assert_eq!(stale_dumps(&existing, Some((16, 19))), []);
        assert_eq!(stale_dumps(&existing, Some((16, 20))), [v("16.20.8245011")]);
        assert_eq!(stale_dumps(&existing, Some((16, 21))), existing);
    }

    #[test]
    fn superseded_dumps_returns_every_dump_except_the_greatest() {
        let existing = [v("16.21.8250000"), v("16.21.8255794"), v("16.20.8245011")];
        assert_eq!(
            superseded_dumps(&existing),
            [v("16.21.8250000"), v("16.20.8245011")]
        );
        assert_eq!(superseded_dumps(&[v("16.21.8255794")]), []);
        assert_eq!(superseded_dumps(&[]), []);
    }

    #[test]
    fn decide_returns_not_ahead_if_patch_equals_live_patch() {
        assert_eq!(
            decide(&v("16.19.8198988"), &[], Some((16, 19))),
            Decision::NotAhead
        );
    }

    #[test]
    fn decide_returns_dump_if_build_is_below_live_build() {
        // Ordering is by channel. PBE 16.20 builds start at 8209878 and live
        // 16.19 reached 8230722.
        assert_eq!(
            decide(&v("16.20.8209878"), &[], Some((16, 19))),
            Decision::Dump
        );
    }

    #[test]
    fn decide_returns_up_to_date_if_build_is_dumped() {
        assert_eq!(
            decide(&v("16.21.8255794"), &[v("16.21.8255794")], Some((16, 19))),
            Decision::UpToDate
        );
    }

    #[test]
    fn decide_returns_up_to_date_if_newer_build_is_dumped() {
        assert_eq!(
            decide(&v("16.21.8250000"), &[v("16.21.8255794")], Some((16, 19))),
            Decision::UpToDate
        );
    }

    #[test]
    fn decide_returns_dump_if_only_older_builds_are_dumped() {
        assert_eq!(
            decide(&v("16.21.8255794"), &[v("16.20.8245011")], Some((16, 19))),
            Decision::Dump
        );
    }

    #[test]
    fn replace_dumps_moves_new_dump_and_removes_existing_dumps() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join("pbe");
        touch(&dir, "16.20.8245011.json");
        let new_dump = tmp.path().join("16.21.8255794.reduced.json");
        fs::write(&new_dump, b"new").unwrap();

        let output =
            replace_dumps(&dir, &[v("16.20.8245011")], &v("16.21.8255794"), &new_dump).unwrap();

        assert_eq!(output, dir.join("16.21.8255794.json"));
        assert_eq!(fs::read(&output).unwrap(), b"new");
        assert!(!new_dump.exists());
        assert_eq!(dump_versions(&dir).unwrap(), [v("16.21.8255794")]);
    }

    #[test]
    fn replace_dumps_creates_directory_if_missing() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join("pbe");
        let new_dump = tmp.path().join("16.21.8255794.reduced.json");
        fs::write(&new_dump, b"new").unwrap();

        replace_dumps(&dir, &[], &v("16.21.8255794"), &new_dump).unwrap();

        assert_eq!(dump_versions(&dir).unwrap(), [v("16.21.8255794")]);
    }

    #[test]
    fn validate_dump_returns_dump_with_minimum_class_count() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("dump.json");
        write_dump(&path, MIN_DUMP_CLASSES);
        let dump = validate_dump(&path).unwrap();
        assert_eq!(dump["classes"].as_object().unwrap().len(), MIN_DUMP_CLASSES);
    }

    #[test]
    fn validate_dump_fails_if_class_count_is_below_minimum() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("dump.json");
        write_dump(&path, MIN_DUMP_CLASSES - 1);
        assert!(matches!(
            validate_dump(&path),
            Err(SyncError::InvalidDump { .. })
        ));
    }

    #[test]
    fn validate_dump_fails_if_file_is_not_json() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("dump.json");
        fs::write(&path, b"{\"classes\": {").unwrap();
        assert!(matches!(
            validate_dump(&path),
            Err(SyncError::InvalidDump { .. })
        ));
    }

    #[test]
    fn reduce_dump_keeps_the_fields_that_the_database_build_reads() {
        let mut dump = full_dump();
        reduce_dump(&mut dump);
        assert_eq!(
            dump,
            json!({
                "formatVersion": 3,
                "version": "16.21.8255794",
                "reduced": true,
                "hashers": {
                    "0x22f54c0": {
                        "storage_width": 4,
                        "hash_function": {"algorithm": "Fnv1a32", "lowercased": true}
                    }
                },
                "classes": {
                    "0x1003c990": {
                        "base": "0x91873e8a",
                        "secondary_bases": {"0xb0607142": 16},
                        "is": {"interface": false, "value": true},
                        "properties": {
                            "0xd4db947d": {
                                "other_class": "0xb84d0322",
                                "value_type": "List2",
                                "container": {"value_type": "Embed", "fixed_size": null},
                                "map": null,
                                "hasher": null
                            },
                            "0xf0a363e3": {
                                "other_class": null,
                                "value_type": "Map",
                                "container": null,
                                "map": {"key_type": "Hash", "value_type": "Link"},
                                "hasher": "0x22f54c0"
                            }
                        },
                        "defaults": {"0xd4db947d": [], "0xf0a363e3": {}}
                    }
                }
            })
        );
    }

    #[test]
    fn reduce_dump_keeps_fields_that_it_does_not_know() {
        let mut dump = json!({
            "classes": {"0x1": {"future": 1, "size": 8, "properties": {"0x2": {"future": 2, "offset": 4}}}}
        });
        reduce_dump(&mut dump);
        assert_eq!(
            dump["classes"]["0x1"],
            json!({"future": 1, "properties": {"0x2": {"future": 2}}})
        );
    }

    #[test]
    fn write_reduced_dump_preserves_float_defaults() {
        // Defaults of 16.21.8255794 that the default float parser of serde_json
        // reads one unit in the last place off. The database build compares
        // defaults by value, so a changed float changes the overlay.
        let literals = [
            "-3.4028234663852886e+38",
            "0.014999999664723873",
            "1.7777800559997559",
            "0.029999999329447746",
        ];
        let tmp = tempfile::tempdir().unwrap();
        let full = tmp.path().join("full.json");
        let reduced = tmp.path().join("reduced.json");
        let filler: Vec<String> = (1..MIN_DUMP_CLASSES)
            .map(|i| format!(",\"{:#x}\": {{}}", i))
            .collect();
        fs::write(
            &full,
            format!(
                "{{\"classes\": {{\"0x0\": {{\"defaults\": {{\"0xa\": [{}]}}}}{}}}}}",
                literals.join(","),
                filler.concat()
            ),
        )
        .unwrap();

        let mut dump = validate_dump(&full).unwrap();
        reduce_dump(&mut dump);
        write_reduced_dump(&reduced, &dump).unwrap();

        let written: Value = serde_json::from_str(&fs::read_to_string(&reduced).unwrap()).unwrap();
        let written: Vec<f64> = written["classes"]["0x0"]["defaults"]["0xa"]
            .as_array()
            .unwrap()
            .iter()
            .map(|value| value.as_f64().unwrap())
            .collect();
        let expected: Vec<f64> = literals.iter().map(|l| l.parse().unwrap()).collect();
        assert_eq!(written, expected);
    }

    #[test]
    fn write_reduced_dump_writes_one_class_per_line_ordered_by_hash() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("pbe").join("dump.json");
        let dump = json!({
            "formatVersion": 3,
            "reduced": true,
            "classes": {"0x10": {"base": null}, "0x9": {"base": "0x10"}, "0xa": {}}
        });

        write_reduced_dump(&path, &dump).unwrap();

        let text = fs::read_to_string(&path).unwrap();
        assert_eq!(
            text,
            "{\n\
             \"formatVersion\": 3,\n\
             \"reduced\": true,\n\
             \"classes\": {\n\
             \"0x9\": {\"base\":\"0x10\"},\n\
             \"0xa\": {},\n\
             \"0x10\": {\"base\":null}\n\
             }\n\
             }\n"
        );
        assert_eq!(serde_json::from_str::<Value>(&text).unwrap(), dump);
    }

    #[test]
    fn report_outputs_returns_empty_values_if_nothing_changed() {
        let report = Report {
            newest: Some(v("16.21.8255794")),
            ..Report::default()
        };
        assert!(!report.changed());
        assert_eq!(
            report.outputs(),
            [
                ("changed", "false".to_string()),
                ("needs_dumper", "false".to_string()),
                ("newest", "16.21.8255794".to_string()),
                ("version", String::new()),
                ("removed", String::new()),
                ("full_dump", String::new()),
            ]
        );
    }

    #[test]
    fn report_outputs_returns_dumped_and_removed_versions() {
        let report = Report {
            newest: Some(v("16.21.8255794")),
            removed: vec![v("16.20.8245011"), v("16.21.8250000")],
            dumped: Some(v("16.21.8255794")),
            full_dump: Some(PathBuf::from("temp/pbe/16.21.8255794.json")),
        };
        assert!(report.changed());
        let outputs = report.outputs();
        assert_eq!(outputs[0], ("changed", "true".to_string()));
        assert_eq!(outputs[1], ("needs_dumper", "true".to_string()));
        assert_eq!(outputs[3], ("version", "16.21.8255794".to_string()));
        assert_eq!(
            outputs[4],
            ("removed", "16.20.8245011 16.21.8250000".to_string())
        );
        assert!(outputs[5].1.ends_with("16.21.8255794.json"));
    }

    #[test]
    fn report_changed_returns_true_if_only_dumps_are_removed() {
        let report = Report {
            removed: vec![v("16.20.8245011")],
            ..Report::default()
        };
        assert!(report.changed());
        assert_eq!(report.outputs()[1], ("needs_dumper", "false".to_string()));
    }
}
