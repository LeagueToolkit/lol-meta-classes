//! Client for sieve (`sieve.services.riotcdn.net`), Riot's release index.
//!
//! Sieve is authoritative for the builds that a region serves at the time of the
//! request, and each release carries its RMAN manifest URL. Sieve lists a rolling
//! window of a few releases, so it cannot answer for history.

use thiserror::Error;

/// Riot's live release index, keyed by region.
const SIEVE_URL: &str = "https://sieve.services.riotcdn.net/api/v1/products/lol/version-sets";

/// The artifact type we want out of a version set. Each set also carries
/// `lol-standalone-client-content` at the same version, which is not the binary.
const SIEVE_ARTIFACT: &str = "lol-game-client";

/// Platform to ask sieve for. The target binary is the macOS app, so the two have
/// to stay in step.
const SIEVE_PLATFORM: &str = "macos";

/// Sieve accepts any agent; an identifiable one just makes the traffic legible.
const USER_AGENT: &str = concat!(
    "meta-sync/",
    env!("CARGO_PKG_VERSION"),
    " (lol-meta-classes)"
);

/// Error of a sieve lookup.
#[derive(Error, Debug)]
pub enum SieveError {
    #[error("{0}")]
    Http(#[from] reqwest::Error),

    #[error("sieve returned a body that is not JSON: {0}")]
    Json(#[from] serde_json::Error),

    #[error(
        "sieve has no version set for region {region}. A region Riot has retired - \
         STAGING, for one - exists only in the manifest archive."
    )]
    UnknownRegion { region: String },

    #[error("sieve listed no {SIEVE_ARTIFACT} release for region {region}")]
    NoRelease { region: String },
}

/// One `lol-game-client` release as sieve currently serves it.
#[derive(Debug, Clone, PartialEq)]
pub struct SieveRelease {
    pub version: String,
    pub manifest_url: String,
    /// When Riot published the release. Informational, and absent on older records.
    pub created_at: Option<String>,
}

/// The newest `lol-game-client` release sieve lists for a region.
pub async fn fetch_sieve_latest(region: &str) -> Result<SieveRelease, SieveError> {
    fetch_sieve_releases(region)
        .await?
        .into_iter()
        .next()
        .ok_or_else(|| SieveError::NoRelease {
            region: region.to_string(),
        })
}

/// Every `lol-game-client` release sieve currently lists for a region, newest first.
pub async fn fetch_sieve_releases(region: &str) -> Result<Vec<SieveRelease>, SieveError> {
    let response = reqwest::Client::builder()
        .user_agent(USER_AGENT)
        .build()?
        .get(format!("{}/{}", SIEVE_URL, region))
        .query(&[("q[platform]", SIEVE_PLATFORM)])
        .send()
        .await?;

    if response.status() == reqwest::StatusCode::NOT_FOUND {
        return Err(SieveError::UnknownRegion {
            region: region.to_string(),
        });
    }

    let body: serde_json::Value =
        serde_json::from_str(&response.error_for_status()?.text().await?)?;

    Ok(parse_sieve_releases(&body))
}

/// The `lol-game-client` releases in a sieve version set, newest first.
///
/// Ordering is on the parsed version for the same reason the archive listing does
/// it: as strings "16.9" sorts above "16.15". Position is no help either - sieve
/// returns the set in no useful order, and currently puts the newest release last.
///
/// A release that is the wrong artifact, the wrong platform, unparseable, or has no
/// download URL is dropped rather than carried, so one odd record cannot take out
/// the whole lookup.
pub fn parse_sieve_releases(body: &serde_json::Value) -> Vec<SieveRelease> {
    fn label<'a>(release: &'a serde_json::Value, key: &str) -> Option<&'a str> {
        release["labels"][key]["values"][0].as_str()
    }

    let mut releases: Vec<(semver::Version, SieveRelease)> = Vec::new();

    for entry in body["releases"].as_array().into_iter().flatten() {
        let release = &entry["release"];

        if label(release, "riot:artifact_type_id") != Some(SIEVE_ARTIFACT)
            || label(release, "riot:platform") != Some(SIEVE_PLATFORM)
        {
            continue;
        }

        // "16.17.8104348+branch.releases-16-17.code.public..." -> "16.17.8104348"
        let Some(version) =
            label(release, "riot:artifact_version_id").and_then(|v| v.split('+').next())
        else {
            continue;
        };
        let Ok(parsed) = semver::Version::parse(version) else {
            continue;
        };
        let Some(manifest_url) = entry["download"]["url"].as_str() else {
            continue;
        };

        releases.push((
            parsed,
            SieveRelease {
                version: version.to_string(),
                manifest_url: manifest_url.to_string(),
                created_at: release["created_at"].as_str().map(str::to_string),
            },
        ));
    }

    releases.sort_by(|(a, _), (b, _)| b.cmp(a));
    releases.into_iter().map(|(_, r)| r).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A trimmed sieve version set, keeping the shape that matters: the newest
    /// release listed last, a second artifact type riding along at a *higher*
    /// version, a lexicographic ordering trap, and a record with no download URL.
    const SIEVE_FIXTURE: &str = r#"{
      "releases": [
        {
          "release": {
            "labels": {
              "platform": {"values": ["macos"]},
              "riot:artifact_type_id": {"values": ["lol-game-client"]},
              "riot:artifact_version_id": {"values": ["16.16.8049184+branch.releases-16-16.code.public"]},
              "riot:platform": {"values": ["macos"]}
            },
            "created_at": "2026-08-10T23:23:37.000Z"
          },
          "download": {"url": "https://cdn.invalid/releases/4A744F7CD02B5174.manifest"}
        },
        {
          "release": {
            "labels": {
              "riot:artifact_type_id": {"values": ["lol-standalone-client-content"]},
              "riot:artifact_version_id": {"values": ["16.18.9999999+branch.main.code.public"]},
              "riot:platform": {"values": ["macos"]}
            }
          },
          "download": {"url": "https://cdn.invalid/releases/CONTENT.manifest"}
        },
        {
          "release": {
            "labels": {
              "riot:artifact_type_id": {"values": ["lol-game-client"]},
              "riot:artifact_version_id": {"values": ["16.9.7728292+branch.releases-16-9.code.public"]},
              "riot:platform": {"values": ["macos"]}
            }
          },
          "download": {"url": "https://cdn.invalid/releases/OLD.manifest"}
        },
        {
          "release": {
            "labels": {
              "riot:artifact_type_id": {"values": ["lol-game-client"]},
              "riot:artifact_version_id": {"values": ["16.17.8104348+branch.releases-16-17.code.public"]},
              "riot:platform": {"values": ["windows"]}
            }
          },
          "download": {"url": "https://cdn.invalid/releases/WINDOWS.manifest"}
        },
        {
          "release": {
            "labels": {
              "riot:artifact_type_id": {"values": ["lol-game-client"]},
              "riot:artifact_version_id": {"values": ["16.17.8110000+branch.releases-16-17.code.public"]},
              "riot:platform": {"values": ["macos"]}
            }
          }
        },
        {
          "release": {
            "labels": {
              "riot:artifact_type_id": {"values": ["lol-game-client"]},
              "riot:artifact_version_id": {"values": ["16.17.8104348+branch.releases-16-17.code.public"]},
              "riot:platform": {"values": ["macos"]}
            },
            "created_at": "2026-08-24T23:43:08.963Z"
          },
          "download": {"url": "https://cdn.invalid/releases/A85CF8825A9BC0A4.manifest"}
        }
      ]
    }"#;

    fn parsed_fixture() -> Vec<SieveRelease> {
        parse_sieve_releases(&serde_json::from_str(SIEVE_FIXTURE).unwrap())
    }

    #[test]
    fn sieve_picks_the_newest_by_version_not_by_position() {
        let newest = parsed_fixture().into_iter().next().unwrap();
        assert_eq!(
            newest,
            SieveRelease {
                version: "16.17.8104348".to_string(),
                manifest_url: "https://cdn.invalid/releases/A85CF8825A9BC0A4.manifest".to_string(),
                created_at: Some("2026-08-24T23:43:08.963Z".to_string()),
            }
        );
    }

    #[test]
    fn sieve_orders_newest_first_across_patches() {
        // "16.9" sorts above both of the others as a string, and the archive
        // listing had exactly that regression once already.
        let versions: Vec<_> = parsed_fixture().into_iter().map(|r| r.version).collect();
        assert_eq!(versions, ["16.17.8104348", "16.16.8049184", "16.9.7728292"]);
    }

    #[test]
    fn sieve_keeps_only_the_game_client_on_the_asked_for_platform() {
        // Both would sort to the front if they leaked through: the content
        // artifact is 16.18, and the windows build shares the newest version but
        // is not the binary this tool downloads.
        let urls: Vec<_> = parsed_fixture()
            .into_iter()
            .map(|r| r.manifest_url)
            .collect();
        assert!(!urls.iter().any(|u| u.contains("CONTENT")));
        assert!(!urls.iter().any(|u| u.contains("WINDOWS")));
    }

    #[test]
    fn sieve_releases_are_matchable_by_bare_version() {
        // The dump workflows resolve through sieve and then ask for that version by
        // name, and `get_manifest_url` falls back to matching it against this list.
        // That only works while the `+branch...` suffix is stripped off.
        let found = parsed_fixture()
            .into_iter()
            .find(|r| r.version == "16.16.8049184");
        assert_eq!(
            found.map(|r| r.manifest_url),
            Some("https://cdn.invalid/releases/4A744F7CD02B5174.manifest".to_string())
        );
    }

    #[test]
    fn sieve_drops_a_release_with_no_download_url() {
        // 16.17.8110000 is the highest game-client version in the fixture, so it
        // would be picked if a missing URL were not enough to disqualify it.
        assert!(!parsed_fixture()
            .into_iter()
            .any(|r| r.version == "16.17.8110000"));
    }

    #[test]
    fn sieve_tolerates_an_empty_or_shapeless_body() {
        assert!(parse_sieve_releases(&serde_json::json!({})).is_empty());
        assert!(parse_sieve_releases(&serde_json::json!({"releases": []})).is_empty());
        assert!(parse_sieve_releases(&serde_json::json!({"releases": [{}]})).is_empty());
    }
}
