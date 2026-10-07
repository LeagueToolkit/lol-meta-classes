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
use meta_sync::sieve::{fetch_sieve_latest, fetch_sieve_releases};
use std::io::Cursor;
use std::path::PathBuf;

/// CDN base URL for downloading League of Legends files
const CDN_URL: &str = "http://lol.secure.dyn.riotcdn.net/channels/public/bundles";

/// GitHub repository info
const GITHUB_OWNER: &str = "Morilli";
const GITHUB_REPO: &str = "riot-manifests";
const GITHUB_BRANCH: &str = "master";

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
