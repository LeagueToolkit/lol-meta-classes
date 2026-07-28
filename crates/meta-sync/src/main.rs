//! # meta-sync
//!
//! Automated synchronization tool for League of Legends metaclass information.
//!
//! ## What it does
//!
//! 1. Fetches available game versions from GitHub (Morilli/riot-manifests)
//! 2. Downloads RMAN manifests and extracts macOS binaries
//! 3. Runs the dumper tool to extract metaclass definitions
//! 4. Saves structured JSON output to `dumps/{version}.json`
//!
//! ## Usage
//!
//! ```bash
//! cargo run --release --bin meta-sync
//! ```
//!
//! The tool will process all versions newer than the legacy cutoff (13.14.5227601)
//! and skip any versions that have already been dumped.

mod config;
mod dumper;
mod error;
mod github;
mod manifest;

use config::{Config, LEGACY_CUTOFF};
use error::Result;
use octocrab::Octocrab;
use std::io::Write;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🚀 Starting meta-sync");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    // Initialize configuration
    let config = Config::new();
    config.ensure_directories()?;

    // Verify dumper exists
    if !config.dumper_path.exists() {
        eprintln!("❌ Dumper not found at: {}", config.dumper_path.display());
        eprintln!("💡 Build the dumper first:");
        eprintln!("   cargo build --release --bin dumper");
        return Err(error::SyncError::DumperNotFound {
            path: config.dumper_path,
        });
    }
    println!("✓ Dumper found at: {}", config.dumper_path.display());

    // Create GitHub client
    let octocrab = Octocrab::default();

    // Fetch available versions
    let versions = github::fetch_game_versions(&octocrab).await?;
    println!("✓ Found {} game versions", versions.len());
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    let mut processed_count = 0;
    let mut skipped_count = 0;
    let mut failed_count = 0;

    // Process each version
    for game_version in versions {
        let version = &game_version.version;

        // Check if we should process this version
        if !github::should_process_version(version, LEGACY_CUTOFF)? {
            break;
        }

        // Skip if already dumped
        if config.has_dump(version) {
            println!("⏭️  Skipping {} - already dumped", version);
            skipped_count += 1;
            continue;
        }

        println!("\n🎮 Processing version: {}", version);
        println!("────────────────────────────────────────");

        // Process the version
        match process_version(&config, &game_version).await {
            Ok(_) => {
                println!("✅ Successfully processed {}", version);
                processed_count += 1;
            }
            Err(e) => {
                eprintln!("❌ Failed to process {}: {}", version, e);
                failed_count += 1;
                // Continue processing other versions
            }
        }
    }

    // Print summary
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("📊 Processing Summary");
    println!("   Processed: {}", processed_count);
    println!("   Skipped:   {}", skipped_count);
    println!("   Failed:    {}", failed_count);
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    if failed_count > 0 {
        println!("⚠️  {} version(s) failed to process. Check logs above for details.", failed_count);
    } else if processed_count > 0 {
        println!("🎉 All versions processed successfully!");
    } else {
        println!("✓ No new versions to process");
    }

    // Exit non-zero when any version failed. This used to return Ok(()) regardless,
    // so a run where every version failed still went green and the failures stayed
    // buried in the log. Versions that did succeed are already written to dumps/,
    // and the workflow commits them before acting on this exit code.
    if failed_count > 0 {
        std::io::stdout().flush()?;
        std::process::exit(1);
    }

    Ok(())
}

/// Processes a single game version: downloads binary and runs dumper
async fn process_version(config: &Config, game_version: &github::GameVersion) -> Result<()> {
    let version = &game_version.version;

    // Get manifest URL from GitHub file
    println!("  🔗 Fetching manifest URL...");
    let manifest_url = github::fetch_manifest_url(&game_version.download_url).await?;

    // Download binary
    let temp_binary_path = config.temp_binary_path(version);
    manifest::download_game_binary(&manifest_url, &temp_binary_path).await?;

    // Run dumper
    let output_path = config.dump_path(version);
    dumper::execute_dumper(config, &temp_binary_path, &output_path)?;

    // Cleanup temporary file
    dumper::cleanup_temp_file(&temp_binary_path)?;

    Ok(())
}
