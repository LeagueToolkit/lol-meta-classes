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
//! cargo run --release --bin meta-sync -- --channel pbe
//! ```
//!
//! The tool will process all versions newer than the legacy cutoff (13.14.5227601)
//! and skip any versions that have already been dumped.
//!
//! `--channel pbe` runs the PBE pass instead: it keeps the newest PBE build as
//! the only dump in `dumps/pbe/`. See [`preview`]. With `--plan` the PBE pass
//! reports what it would do and changes nothing. `--outputs <path>` appends the
//! result of the PBE pass to a file as `key=value` lines.

mod config;
mod dumper;
mod error;
mod github;
mod manifest;
mod outputs;
mod preview;

use clap::error::ErrorKind;
use clap::{CommandFactory, Parser, ValueEnum};
use config::{Config, LEGACY_CUTOFF};
use error::Result;
use octocrab::Octocrab;
use std::io::Write;
use std::path::PathBuf;

/// Release channel that a run syncs.
#[derive(Clone, Copy, Debug, PartialEq, ValueEnum)]
enum Channel {
    /// Dumps every live EUW1 build that `dumps/` lacks
    Live,
    /// Keeps the newest PBE build as the only dump in `dumps/pbe/`
    Pbe,
}

#[derive(Parser)]
#[command(name = "meta-sync")]
#[command(about = "Dumps League of Legends metaclass information into dumps/")]
struct Args {
    /// Release channel to sync
    #[arg(long, value_enum, default_value_t = Channel::Live)]
    channel: Channel,

    /// Report what the PBE pass would do, without changing a file or
    /// downloading a binary
    #[arg(long)]
    plan: bool,

    /// File that receives the result of the PBE pass as `key=value` lines, for
    /// example `$GITHUB_OUTPUT`
    #[arg(long, value_name = "PATH")]
    outputs: Option<PathBuf>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    if args.channel == Channel::Live && (args.plan || args.outputs.is_some()) {
        Args::command()
            .error(
                ErrorKind::ArgumentConflict,
                "--plan and --outputs require --channel pbe",
            )
            .exit();
    }

    println!("🚀 Starting meta-sync");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    // Initialize configuration
    let config = Config::new();
    config.ensure_directories()?;

    match args.channel {
        Channel::Live => sync_live(&config).await,
        Channel::Pbe => {
            let mode = if args.plan {
                preview::Mode::Plan
            } else {
                preview::Mode::Apply
            };
            let mut report = preview::Report::default();
            let result = preview::run(&config, mode, &mut report).await;

            // Written for a failed pass too: the pass removes obsolete dumps
            // before the steps that can fail.
            if let Some(path) = &args.outputs {
                outputs::append_outputs(path, &report.outputs())?;
            }

            // The error is printed here because the `Debug` output that a
            // returned error gets does not expand the dumper's stderr.
            if let Err(e) = result {
                eprintln!("❌ PBE pass failed: {}", e);
                std::io::stdout().flush()?;
                std::process::exit(1);
            }
            Ok(())
        }
    }
}

/// Dumps every live build that `dumps/` lacks.
async fn sync_live(config: &Config) -> Result<()> {
    // Verify dumper exists
    if !config.dumper_path.exists() {
        eprintln!("❌ Dumper not found at: {}", config.dumper_path.display());
        eprintln!("💡 Build the dumper first:");
        eprintln!("   cargo build --release --bin dumper");
        return Err(error::SyncError::DumperNotFound {
            path: config.dumper_path.clone(),
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

        // Check if we should process this version. Breaking rather than
        // continuing is only sound because the list is ordered newest first by
        // semantic version: everything after the first version below the cutoff
        // is older still.
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
        match process_version(config, &game_version).await {
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
