//! Download a League of Legends binary for a specific version.
//!
//! This tool downloads the macOS League of Legends binary from Riot's CDN
//! so you can analyze it in IDA, Ghidra, or other disassemblers - or feed it
//! straight to `dumper`.
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
//! # List available versions for a region
//! cargo run --release --bin download-binary -- --region KR --list
//! ```
//!
//! Set `GITHUB_TOKEN` to raise the API rate limit; without it GitHub allows 60
//! requests an hour per IP, which CI runners share.

use clap::Parser;
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

    /// Use the newest version available for the region
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    if args.list {
        list_versions(&args.region, args.count).await?;
        return Ok(());
    }

    let version = resolve_version(&args.region, args.version.as_deref(), args.latest).await?;

    // `--resolve` is what CI uses to name the dump and the artifact before doing
    // any work, so it must print the bare version and nothing else on stdout.
    if args.resolve {
        println!("{}", version);
        return Ok(());
    }

    let output = args
        .output
        .unwrap_or_else(|| PathBuf::from(format!("{}.bin", version)));

    download_binary(&args.region, &version, &output).await?;

    Ok(())
}

/// Works out which version to act on.
///
/// `--latest`, or the literal version string "latest", resolves against the region
/// listing; anything else is taken at face value and validated when its manifest
/// pointer is fetched.
async fn resolve_version(
    region: &str,
    version: Option<&str>,
    latest: bool,
) -> Result<String, Box<dyn std::error::Error>> {
    let wants_latest = latest || matches!(version, Some(v) if v.eq_ignore_ascii_case("latest"));

    if wants_latest {
        let versions = fetch_versions(region).await?;
        return versions
            .into_iter()
            .next()
            .ok_or_else(|| format!("No versions found for region {}", region).into());
    }

    match version {
        Some(v) => Ok(v.to_string()),
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

/// Every version available for a region, newest first.
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
        "Fetching available versions for {} from {}/{}...",
        region, GITHUB_OWNER, GITHUB_REPO
    );

    let versions = fetch_versions(region).await?;

    println!(
        "\nAvailable versions (showing {} most recent):\n",
        count.min(versions.len())
    );

    for version in versions.iter().take(count) {
        println!("  {}", version);
    }

    println!("\nTotal: {} versions available", versions.len());
    println!("\nUsage: download-binary <VERSION> [-r REGION] [-o output.bin]");

    Ok(())
}

async fn download_binary(
    region: &str,
    version: &str,
    output: &PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Downloading League of Legends binary");
    println!("  Region:  {}", region);
    println!("  Version: {}", version);
    println!("  Output:  {}", output.display());
    println!();

    // Step 1: Get manifest URL from GitHub
    println!("[1/4] Fetching manifest URL from GitHub...");
    let manifest_url = get_manifest_url(region, version).await?;
    println!("       Found manifest URL");

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

/// Reads the manifest pointer for one version straight off raw.githubusercontent.
///
/// The file is a single URL. Fetching it by path avoids listing the directory at
/// all, which is what makes a targeted download work on regions whose listing is
/// too large for the contents API.
async fn get_manifest_url(
    region: &str,
    version: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let url = version_file_url(region, version);
    let response = reqwest::get(&url).await?;

    if response.status() == reqwest::StatusCode::NOT_FOUND {
        return Err(format!(
            "Version {} not found for region {}. Use --region {} --list to see what is available.",
            version, region, region
        )
        .into());
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
        let v = resolve_version("EUW1", Some("16.1.7374870"), false)
            .await
            .unwrap();
        assert_eq!(v, "16.1.7374870");
    }

    #[tokio::test]
    async fn no_version_and_no_latest_is_an_error() {
        assert!(resolve_version("EUW1", None, false).await.is_err());
    }
}
