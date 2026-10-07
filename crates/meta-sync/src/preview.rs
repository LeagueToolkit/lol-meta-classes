//! PBE pass: keeps the newest PBE build as the only dump in `dumps/pbe/`.
//!
//! `scripts/db_build.py` folds that dump on top of the live history and writes
//! the difference under the `preview` key of `db/meta.db.json`. A PBE dump is
//! kept only while its patch is greater than the latest live patch.
//!
//! The newest build is resolved through sieve. The manifest archive lags sieve
//! by up to a day and drops a build that was superseded inside one snapshot
//! window.

use crate::config::Config;
use crate::dumper;
use crate::error::{Result, SyncError};
use crate::manifest;
use meta_sync::sieve;
use semver::Version;
use std::path::{Path, PathBuf};

/// Region that the PBE pass reads.
pub const PREVIEW_REGION: &str = "PBE1";

/// Size above which a dump is rejected as corrupted.
const MAX_DUMP_BYTES: u64 = 50 * 1024 * 1024;

/// Class count below which a dump is rejected as truncated.
const MIN_DUMP_CLASSES: usize = 1000;

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

/// Runs the PBE pass.
///
/// Removes every PBE dump whose patch is not greater than the latest live
/// patch, and every PBE dump except the newest. Dumps the newest PBE build if
/// its patch is greater than the latest live patch and `dumps/pbe/` holds no
/// dump of that build or of a newer build. If the lookup, the download, the
/// dumper or the validation fails, returns the error and leaves the remaining
/// PBE dump in place.
pub async fn run(config: &Config) -> Result<()> {
    let preview_dir = config.preview_dumps_dir();
    let live = latest_patch(&dump_versions(&config.dumps_dir)?);
    match live {
        Some((major, minor)) => println!("✓ Latest live patch: {}.{}", major, minor),
        None => println!("⚠️  No live dump found in {}", config.dumps_dir.display()),
    }

    let existing = dump_versions(&preview_dir)?;
    let stale = stale_dumps(&existing, live);
    for version in &stale {
        println!(
            "🧹 Removing PBE dump {} - live has reached its patch",
            version
        );
    }
    remove_dumps(&preview_dir, &stale)?;
    let existing: Vec<Version> = existing
        .into_iter()
        .filter(|v| !stale.contains(v))
        .collect();

    // Two sync PRs that merge out of order leave two dumps in the directory.
    let superseded = superseded_dumps(&existing);
    for version in &superseded {
        println!("🧹 Removing PBE dump {} - a newer PBE dump exists", version);
    }
    remove_dumps(&preview_dir, &superseded)?;
    let existing: Vec<Version> = existing.into_iter().max().into_iter().collect();

    let release = sieve::fetch_sieve_latest(PREVIEW_REGION).await?;
    println!("✓ {} is serving {}", PREVIEW_REGION, release.version);
    let newest = Version::parse(&release.version)
        .map_err(|_| SyncError::InvalidVersion(release.version.clone()))?;

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

    println!("\n🎮 Processing PBE version: {}", newest);
    println!("────────────────────────────────────────");

    let temp_binary_path = config.temp_binary_path(&release.version);
    manifest::download_game_binary(&release.manifest_url, &temp_binary_path).await?;

    // The dumper writes to the temporary directory, so a failed or rejected
    // dump never replaces the dump that `dumps/pbe/` holds.
    let temp_dump_path = config
        .temp_dir
        .join(format!("pbe-{}.json", release.version));
    dumper::execute_dumper(config, &temp_binary_path, &temp_dump_path)?;
    dumper::cleanup_temp_file(&temp_binary_path)?;
    validate_dump(&temp_dump_path)?;

    let output_path = replace_dumps(&preview_dir, &existing, &newest, &temp_dump_path)?;
    println!("✅ Wrote {}", output_path.display());

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

/// Checks that the file at `path` is a plausible dump.
///
/// Fails if the file is larger than [`MAX_DUMP_BYTES`], is not JSON, or holds
/// fewer than [`MIN_DUMP_CLASSES`] classes.
fn validate_dump(path: &Path) -> Result<()> {
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

    let dump: serde_json::Value =
        serde_json::from_reader(std::io::BufReader::new(std::fs::File::open(path)?))
            .map_err(|e| invalid(format!("not JSON: {}", e)))?;

    let classes = dump["classes"].as_object().map_or(0, |c| c.len());
    if classes < MIN_DUMP_CLASSES {
        return Err(invalid(format!(
            "{} classes is below the minimum of {}",
            classes, MIN_DUMP_CLASSES
        )));
    }

    println!("  ✓ Dump holds {} classes ({} bytes)", classes, size);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn v(version: &str) -> Version {
        Version::parse(version).unwrap()
    }

    fn touch(dir: &Path, name: &str) {
        fs::create_dir_all(dir).unwrap();
        fs::write(dir.join(name), b"{}").unwrap();
    }

    fn write_dump(path: &Path, classes: usize) {
        let classes: serde_json::Map<String, serde_json::Value> = (0..classes)
            .map(|i| (format!("{:#x}", i), serde_json::json!({})))
            .collect();
        fs::write(path, serde_json::json!({ "classes": classes }).to_string()).unwrap();
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
        let new_dump = tmp.path().join("pbe-16.21.8255794.json");
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
        let new_dump = tmp.path().join("pbe-16.21.8255794.json");
        fs::write(&new_dump, b"new").unwrap();

        replace_dumps(&dir, &[], &v("16.21.8255794"), &new_dump).unwrap();

        assert_eq!(dump_versions(&dir).unwrap(), [v("16.21.8255794")]);
    }

    #[test]
    fn validate_dump_accepts_dump_with_minimum_class_count() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("dump.json");
        write_dump(&path, MIN_DUMP_CLASSES);
        assert!(validate_dump(&path).is_ok());
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
}
