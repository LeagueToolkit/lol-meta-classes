//! Download a League of Legends binary for a specific version.
//!
//! This tool downloads the macOS League of Legends binary from Riot's CDN
//! so you can analyze it in IDA, Ghidra, or other disassemblers - or feed it
//! straight to `dumper`.
//!
//! ## Where versions come from
//!
//! Two sources, each for what it is actually good at:
//!
//! - **sieve** (`sieve.services.riotcdn.net`) is Riot's own release index, and is
//!   authoritative for what a region is serving *right now*. It hands back the RMAN
//!   manifest URL along with the version, so resolving through it needs no GitHub
//!   round trip at all. It only exposes a rolling window of a few releases, so it
//!   cannot answer for history.
//! - **`Morilli/riot-manifests`** is a once-a-day snapshot of sieve (16:00 UTC, run
//!   off a local scheduler rather than CI). It holds the full archive, which is what
//!   a named version resolves against, but it lags a live patch by up to a day and
//!   silently drops any build that appeared and was superseded inside one window.
//!
//! So `--latest` asks sieve and everything else asks the archive. If sieve is
//! unreachable, `--latest` falls back to the archive and says so on stderr.
//!
//! ## Usage
//!
//! ```bash
//! # Download a specific version (live EUW1 by default)
//! cargo run --release --bin download-binary -- 16.1.7374870
//!
//! # Newest build on PBE
//! cargo run --release --bin download-binary -- --region PBE1 --latest
//!
//! # Just print which version --latest would pick, and exit
//! cargo run --release --bin download-binary -- --region PBE1 --latest --resolve
//!
//! # Download to a custom output path
//! cargo run --release --bin download-binary -- 16.1.7374870 -o /tmp/lol_16.1.bin
//!
//! # List archived versions for a region, and what it is serving now
//! cargo run --release --bin download-binary -- --region KR --list
//! ```
//!
//! Set `GITHUB_TOKEN` to raise the API rate limit; without it GitHub allows 60
//! requests an hour per IP, which CI runners share. Only the archive paths need it.

use clap::Parser;
use std::io::Cursor;
use std::path::PathBuf;

/// CDN base URL for downloading League of Legends files
const CDN_URL: &str = "http://lol.secure.dyn.riotcdn.net/channels/public/bundles";

/// GitHub repository info
const GITHUB_OWNER: &str = "Morilli";
const GITHUB_REPO: &str = "riot-manifests";
const GITHUB_BRANCH: &str = "master";

/// Riot's live release index, keyed by region.
const SIEVE_URL: &str = "https://sieve.services.riotcdn.net/api/v1/products/lol/version-sets";

/// The artifact type we want out of a version set. Each set also carries
/// `lol-standalone-client-content` at the same version, which is not the binary.
const SIEVE_ARTIFACT: &str = "lol-game-client";

/// Platform to ask sieve for. `TARGET_BINARY` is the macOS app, so the two have to
/// stay in step.
const SIEVE_PLATFORM: &str = "macos";

/// Sieve accepts any agent; an identifiable one just makes the traffic legible.
const USER_AGENT: &str = concat!(
    "download-binary/",
    env!("CARGO_PKG_VERSION"),
    " (lol-meta-classes)"
);

/// Default region. EUW1 is what `meta-sync` tracks, so it stays the default here.
const DEFAULT_REGION: &str = "EUW1";

/// The specific binary file we're looking for in the manifest
const TARGET_BINARY: &str = "LeagueofLegends.app/Contents/MacOS/LeagueofLegends";

/// Path to the per-version manifest pointers for a region.
fn manifest_dir(region: &str) -> String {
    format!("LoL/{}/macos/lol-game-client", region)
}

/// Raw URL of a single version's manifest pointer file.
fn version_file_url(region: &str, version: &str) -> String {
    format!(
        "https://raw.githubusercontent.com/{}/{}/{}/{}/{}.txt",
        GITHUB_OWNER,
        GITHUB_REPO,
        GITHUB_BRANCH,
        manifest_dir(region),
        version
    )
}

#[derive(Parser)]
#[command(name = "download-binary")]
#[command(about = "Download League of Legends binary for analysis in IDA/Ghidra")]
struct Args {
    /// Version to download (e.g., "16.1.7374870"). Ignored when --latest is set.
    #[arg(value_name = "VERSION")]
    version: Option<String>,

    /// Region to take the build from (e.g. EUW1, NA1, KR, PBE1)
    #[arg(short, long, default_value = DEFAULT_REGION, value_name = "REGION")]
    region: String,

    /// Use the newest version the region is serving, according to sieve
    #[arg(long)]
    latest: bool,

    /// Print the resolved version and exit without downloading
    #[arg(long)]
    resolve: bool,

    /// Output file path (defaults to ./{version}.bin)
    #[arg(short, long, value_name = "PATH")]
    output: Option<PathBuf>,

    /// List available versions
    #[arg(short, long)]
    list: bool,

    /// Show the N most recent versions when listing
    #[arg(short = 'n', long, default_value = "20")]
    count: usize,
}

/// A version, plus the manifest URL when the source handed one over.
///
/// sieve returns the manifest URL alongside the version, so resolving through it
/// skips `get_manifest_url` entirely. The archive only stores a pointer file that
/// still has to be read, so that path leaves this `None`.
#[derive(Debug, Clone, PartialEq)]
struct Resolved {
    version: String,
    manifest_url: Option<String>,
}

/// One `lol-game-client` release as sieve currently serves it.
#[derive(Debug, Clone, PartialEq)]
struct SieveRelease {
    version: String,
    manifest_url: String,
    /// When Riot published the release. Informational, and absent on older records.
    created_at: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    if args.list {
        list_versions(&args.region, args.count).await?;
        return Ok(());
    }

    let resolved = resolve_version(&args.region, args.version.as_deref(), args.latest).await?;

    // `--resolve` is what CI uses to name the dump and the artifact before doing
    // any work, so it must print the bare version and nothing else on stdout.
    // Everything the resolution has to say goes to stderr.
    if args.resolve {
        println!("{}", resolved.version);
        return Ok(());
    }

    let output = args
        .output
        .unwrap_or_else(|| PathBuf::from(format!("{}.bin", resolved.version)));

    download_binary(&args.region, &resolved, &output).await?;

    Ok(())
}

/// Works out which version to act on.
///
/// `--latest`, or the literal version string "latest", resolves against sieve;
/// anything else is taken at face value and validated when its manifest pointer is
/// fetched.
async fn resolve_version(
    region: &str,
    version: Option<&str>,
    latest: bool,
) -> Result<Resolved, Box<dyn std::error::Error>> {
    let wants_latest = latest || matches!(version, Some(v) if v.eq_ignore_ascii_case("latest"));

    if wants_latest {
        // The archive is a fallback rather than the source: it is a daily snapshot,
        // so on patch day it can be a full day behind what the region is serving.
        match fetch_sieve_latest(region).await {
            Ok(release) => {
                eprintln!(
                    "sieve: {} is serving {}{}",
                    region,
                    release.version,
                    release
                        .created_at
                        .as_deref()
                        .map(|t| format!(", published {}", t))
                        .unwrap_or_default()
                );
                return Ok(Resolved {
                    version: release.version,
                    manifest_url: Some(release.manifest_url),
                });
            }
            Err(e) => eprintln!(
                "warning: sieve lookup failed ({}); falling back to the manifest \
                 archive, which lags a live patch by up to a day",
                e
            ),
        }

        let versions = fetch_versions(region).await?;
        return versions
            .into_iter()
            .next()
            .map(|version| Resolved {
                version,
                manifest_url: None,
            })
            .ok_or_else(|| format!("No versions found for region {}", region).into());
    }

    match version {
        Some(v) => Ok(Resolved {
            version: v.to_string(),
            manifest_url: None,
        }),
        None => {
            Err("Please provide a version, or --latest. Use --list to see available versions."
                .into())
        }
    }
}

/// The newest `lol-game-client` release sieve lists for a region.
async fn fetch_sieve_latest(region: &str) -> Result<SieveRelease, Box<dyn std::error::Error>> {
    fetch_sieve_releases(region)
        .await?
        .into_iter()
        .next()
        .ok_or_else(|| {
            format!(
                "sieve listed no {} release for region {}",
                SIEVE_ARTIFACT, region
            )
            .into()
        })
}

/// Every `lol-game-client` release sieve currently lists for a region, newest first.
async fn fetch_sieve_releases(
    region: &str,
) -> Result<Vec<SieveRelease>, Box<dyn std::error::Error>> {
    let response = reqwest::Client::builder()
        .user_agent(USER_AGENT)
        .build()?
        .get(format!("{}/{}", SIEVE_URL, region))
        .query(&[("q[platform]", SIEVE_PLATFORM)])
        .send()
        .await?;

    if response.status() == reqwest::StatusCode::NOT_FOUND {
        return Err(format!(
            "sieve has no version set for region {}. A region Riot has retired - \
             STAGING, for one - exists only in the manifest archive.",
            region
        )
        .into());
    }

    let body: serde_json::Value = serde_json::from_str(&response.error_for_status()?.text().await?)?;

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
fn parse_sieve_releases(body: &serde_json::Value) -> Vec<SieveRelease> {
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

/// Builds an octocrab client, authenticated if `GITHUB_TOKEN` is set.
fn client() -> Result<octocrab::Octocrab, Box<dyn std::error::Error>> {
    match std::env::var("GITHUB_TOKEN") {
        Ok(token) if !token.is_empty() => {
            Ok(octocrab::Octocrab::builder().personal_token(token).build()?)
        }
        _ => Ok(octocrab::Octocrab::default()),
    }
}

/// Every archived version for a region, newest first.
///
/// This goes through the **git trees** API rather than the contents API. The
/// contents API silently truncates a directory at 1000 entries, and PBE1 is well
/// past that - it lists nothing newer than 14.24 while the region is on 16.17.
/// Trees returns the whole subtree in one call and sets `truncated` when it
/// cannot, so an over-large listing fails loudly instead of quietly going stale.
async fn fetch_versions(region: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let octocrab = client()?;

    // `<ref>:<path>` addresses the subtree directly, so this is one request.
    let route = format!(
        "/repos/{}/{}/git/trees/{}:{}",
        GITHUB_OWNER,
        GITHUB_REPO,
        GITHUB_BRANCH,
        manifest_dir(region)
    );

    let tree: serde_json::Value = octocrab
        .get(&route, None::<&()>)
        .await
        .map_err(|e| format!("Failed to list region {}: {}", region, e))?;

    if tree["truncated"].as_bool().unwrap_or(false) {
        return Err(format!(
            "GitHub truncated the listing for {} - too many manifests to enumerate in one call",
            region
        )
        .into());
    }

    let entries = tree["tree"]
        .as_array()
        .ok_or_else(|| format!("Unexpected tree response for region {}", region))?;

    let mut versions: Vec<(semver::Version, String)> = entries
        .iter()
        .filter_map(|e| e["path"].as_str())
        .filter_map(|p| p.strip_suffix(".txt"))
        .filter_map(|v| semver::Version::parse(v).ok().map(|s| (s, v.to_string())))
        .collect();

    // Newest first, by parsed version - as strings "16.9" sorts above "16.15".
    versions.sort_by(|(a, _), (b, _)| b.cmp(a));

    Ok(versions.into_iter().map(|(_, v)| v).collect())
}

async fn list_versions(region: &str, count: usize) -> Result<(), Box<dyn std::error::Error>> {
    println!(
        "Fetching archived versions for {} from {}/{}...",
        region, GITHUB_OWNER, GITHUB_REPO
    );

    let versions = fetch_versions(region).await?;

    println!(
        "\nArchived versions (showing {} most recent):\n",
        count.min(versions.len())
    );

    for version in versions.iter().take(count) {
        println!("  {}", version);
    }

    println!("\nTotal: {} versions archived", versions.len());

    // The archive is a daily snapshot, so say plainly when the region has already
    // moved past what it holds. Not being able to reach sieve is not fatal here.
    match fetch_sieve_latest(region).await {
        Ok(release) => println!(
            "Serving now: {}{}",
            release.version,
            if versions.contains(&release.version) {
                ""
            } else {
                "  <- not archived yet, --latest will still get it"
            }
        ),
        Err(e) => println!("Serving now: unavailable ({})", e),
    }

    println!("\nUsage: download-binary <VERSION> [-r REGION] [-o output.bin]");

    Ok(())
}

async fn download_binary(
    region: &str,
    resolved: &Resolved,
    output: &PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Downloading League of Legends binary");
    println!("  Region:  {}", region);
    println!("  Version: {}", resolved.version);
    println!("  Output:  {}", output.display());
    println!();

    // Step 1: Locate the manifest
    println!("[1/4] Locating the RMAN manifest...");
    let manifest_url = match &resolved.manifest_url {
        Some(url) => {
            println!("       Came back with the version from sieve");
            url.clone()
        }
        None => {
            println!("       Looking it up by version...");
            get_manifest_url(region, &resolved.version).await?
        }
    };

    // Step 2: Download manifest
    println!("[2/4] Downloading RMAN manifest...");
    let manifest_response = reqwest::get(&manifest_url).await?;
    let manifest_bytes = manifest_response.bytes().await?;
    println!("       Downloaded {} bytes", manifest_bytes.len());

    // Step 3: Parse manifest and find binary
    println!("[3/4] Parsing manifest...");
    let mut manifest_reader = Cursor::new(manifest_bytes);
    let manifest = rman::Manifest::read(&mut manifest_reader)?;

    let target_file = manifest
        .files
        .iter()
        .find(|f| f.name == TARGET_BINARY)
        .ok_or_else(|| format!("Binary '{}' not found in manifest", TARGET_BINARY))?;

    println!("       Found binary: {}", TARGET_BINARY);
    println!("       Chunks: {}", target_file.chunks.len());

    // Step 4: Download binary
    println!("[4/4] Downloading binary chunks from CDN...");

    // Ensure output directory exists
    if let Some(parent) = output.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)?;
        }
    }

    let mut output_file = std::fs::File::create(output)?;

    target_file
        .download_all()
        .download(&mut ureq::Agent::new(), CDN_URL, &mut output_file)?;

    // Get file size
    let metadata = std::fs::metadata(output)?;
    let size_mb = metadata.len() as f64 / (1024.0 * 1024.0);

    println!();
    println!("Download complete!");
    println!("  File: {}", output.display());
    println!("  Size: {:.2} MB", size_mb);
    println!();
    println!("You can now open this file in IDA, Ghidra, or another disassembler.");
    println!("Note: This is a Mach-O binary (macOS x86_64).");

    Ok(())
}

/// Reads the manifest pointer for one version straight off raw.githubusercontent,
/// falling back to sieve for a build the archive has not snapshotted yet.
///
/// The pointer file is a single URL. Fetching it by path avoids listing the
/// directory at all, which is what makes a targeted download work on regions whose
/// listing is too large for the contents API.
///
/// The archive is tried first because it is the only source with history. A miss
/// there is not yet a wrong version though: on patch day a build is served for
/// hours before the daily snapshot records it, and naming that build explicitly is
/// exactly what the dump workflows do after resolving it through sieve.
async fn get_manifest_url(
    region: &str,
    version: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let url = version_file_url(region, version);
    let response = reqwest::get(&url).await?;

    if response.status() == reqwest::StatusCode::NOT_FOUND {
        return match fetch_sieve_releases(region).await {
            Ok(releases) => releases
                .into_iter()
                .find(|r| r.version == version)
                .map(|r| {
                    eprintln!(
                        "{} is not in the manifest archive yet; taking it from sieve",
                        version
                    );
                    r.manifest_url
                })
                .ok_or_else(|| {
                    format!(
                        "Version {} not found for region {} - not in the manifest \
                         archive, and not one sieve is currently serving. Use \
                         --region {} --list to see what is available.",
                        version, region, region
                    )
                    .into()
                }),
            Err(e) => Err(format!(
                "Version {} not found for region {} in the manifest archive, and \
                 sieve could not be asked either ({}).",
                version, region, e
            )
            .into()),
        };
    }
    let response = response.error_for_status()?;

    Ok(response.text().await?.trim().to_string())
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
    fn manifest_dir_is_region_scoped() {
        assert_eq!(manifest_dir("PBE1"), "LoL/PBE1/macos/lol-game-client");
        assert_eq!(manifest_dir("EUW1"), "LoL/EUW1/macos/lol-game-client");
    }

    #[test]
    fn version_file_url_points_at_raw() {
        assert_eq!(
            version_file_url("PBE1", "16.17.8057408"),
            "https://raw.githubusercontent.com/Morilli/riot-manifests/master/\
LoL/PBE1/macos/lol-game-client/16.17.8057408.txt"
        );
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

    #[tokio::test]
    async fn explicit_version_is_taken_as_given() {
        let resolved = resolve_version("EUW1", Some("16.1.7374870"), false)
            .await
            .unwrap();
        assert_eq!(resolved.version, "16.1.7374870");
        // Nothing looked it up, so the manifest still has to be fetched.
        assert_eq!(resolved.manifest_url, None);
    }

    #[tokio::test]
    async fn no_version_and_no_latest_is_an_error() {
        assert!(resolve_version("EUW1", None, false).await.is_err());
    }
}
