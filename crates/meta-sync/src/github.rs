//! GitHub API interactions for fetching version manifests.

use crate::config::{GITHUB_OWNER, GITHUB_REPO, MANIFEST_PATH};
use crate::error::{Result, SyncError};
use octocrab::Octocrab;
use semver::Version;
use std::path::Path;

/// Represents a League of Legends game version
#[derive(Debug, Clone)]
pub struct GameVersion {
    /// Semantic version (e.g., "15.1.123456")
    pub version: String,
    /// URL to download the manifest file content
    pub download_url: String,
}

/// Fetches available League of Legends game versions from GitHub, newest first
pub async fn fetch_game_versions(octocrab: &Octocrab) -> Result<Vec<GameVersion>> {
    println!("🔍 Fetching game versions from {}/{}...", GITHUB_OWNER, GITHUB_REPO);

    let contents = octocrab
        .repos(GITHUB_OWNER, GITHUB_REPO)
        .get_content()
        .path(MANIFEST_PATH)
        .send()
        .await?;

    println!("📦 Found {} version files", contents.items.len());

    Ok(order_newest_first(
        contents
            .items
            .into_iter()
            .map(|item| (item.name, item.download_url.map(|u| u.to_string()))),
    ))
}

/// Orders manifest entries newest first, by *semantic* version.
///
/// The comparison has to be on parsed versions, not on the filename: as strings
/// "16.9.7728292" sorts above "16.15.7996036", which put a whole patch in the
/// wrong place in the list. That matters beyond cosmetics, because the caller
/// stops at the first version below the legacy cutoff and so relies on this
/// order being monotonic - lexicographically the first 13.x reached is 13.9,
/// which is below the cutoff and ended the run with 13.14-13.24 never visited.
///
/// An entry that is not a version filename, or that GitHub gave no download URL
/// for, is dropped here rather than carried: the caller would only fail on it
/// later, and one stray file in the directory should not end a sync.
fn order_newest_first(
    entries: impl IntoIterator<Item = (String, Option<String>)>,
) -> Vec<GameVersion> {
    let mut versions: Vec<(Version, GameVersion)> = Vec::new();

    for (name, download_url) in entries {
        let Some(version) = extract_version(&name) else {
            continue;
        };
        let Ok(parsed) = Version::parse(&version) else {
            println!("⚠️  Ignoring {} - not a version filename", name);
            continue;
        };
        let Some(download_url) = download_url else {
            println!("⚠️  Ignoring {} - no download URL", name);
            continue;
        };
        versions.push((
            parsed,
            GameVersion {
                version,
                download_url,
            },
        ));
    }

    versions.sort_by(|(a, _), (b, _)| b.cmp(a));
    versions.into_iter().map(|(_, v)| v).collect()
}

/// Fetches the manifest URL from a GitHub file
pub async fn fetch_manifest_url(download_url: &str) -> Result<String> {
    let content = reqwest::get(download_url).await?;
    Ok(content.text().await?)
}

/// Extracts version string from a filename (e.g., "15.1.123456.txt" -> "15.1.123456")
fn extract_version(filename: &str) -> Option<String> {
    Path::new(filename)
        .file_stem()
        .and_then(|s| s.to_str())
        .map(|s| s.to_string())
}

/// Checks if a version should be processed based on the legacy cutoff
pub fn should_process_version(version: &str, cutoff: &str) -> Result<bool> {
    let cutoff_version = Version::parse(cutoff)
        .map_err(|_| SyncError::InvalidVersion(cutoff.to_string()))?;
    
    let current_version = Version::parse(version)
        .map_err(|_| SyncError::InvalidVersion(version.to_string()))?;

    if current_version <= cutoff_version {
        println!(
            "⏹️  Stopping at version {} - reached legacy cutoff (≤ {})",
            version, cutoff_version
        );
        return Ok(false);
    }

    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_version() {
        assert_eq!(
            extract_version("15.1.123456.txt"),
            Some("15.1.123456".to_string())
        );
        assert_eq!(
            extract_version("14.23.987654.txt"),
            Some("14.23.987654".to_string())
        );
    }

    fn ordered(names: &[&str]) -> Vec<String> {
        let entries = names
            .iter()
            .map(|n| (n.to_string(), Some(format!("https://example.invalid/{n}"))));
        order_newest_first(entries)
            .into_iter()
            .map(|v| v.version)
            .collect()
    }

    #[test]
    fn test_order_newest_first_is_semver_not_lexicographic() {
        // The regression: as strings, "16.9" sorts above both "16.15" and "16.10".
        assert_eq!(
            ordered(&["16.15.7996036.txt", "16.9.7728292.txt", "16.10.7747445.txt"]),
            ["16.15.7996036", "16.10.7747445", "16.9.7728292"]
        );
    }

    #[test]
    fn test_order_newest_first_orders_across_majors_and_builds() {
        assert_eq!(
            ordered(&[
                "13.9.4000000.txt",
                "16.14.7949266.txt",
                "9.24.3000000.txt",
                "16.14.7945912.txt",
                "13.24.5000000.txt",
            ]),
            [
                "16.14.7949266",
                "16.14.7945912",
                "13.24.5000000",
                "13.9.4000000",
                "9.24.3000000",
            ]
        );
    }

    #[test]
    fn test_order_newest_first_drops_unusable_entries() {
        let entries = vec![
            ("README.md".to_string(), Some("https://example.invalid/r".to_string())),
            ("16.15.7996036.txt".to_string(), Some("https://example.invalid/m".to_string())),
            ("16.14.7949266.txt".to_string(), None),
        ];
        let out = order_newest_first(entries);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].version, "16.15.7996036");
    }

    #[test]
    fn test_should_process_version() {
        assert!(should_process_version("15.1.123456", "13.14.5227601").unwrap());
        assert!(!should_process_version("13.14.5227601", "13.14.5227601").unwrap());
        assert!(!should_process_version("13.13.0", "13.14.5227601").unwrap());
    }
}
