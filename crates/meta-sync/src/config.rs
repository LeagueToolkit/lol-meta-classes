//! Configuration constants and settings for meta-sync.

use std::path::PathBuf;

/// CDN base URL for downloading League of Legends files
pub const CDN_URL: &str = "http://lol.secure.dyn.riotcdn.net/channels/public/bundles";

/// GitHub repository owner
pub const GITHUB_OWNER: &str = "Morilli";

/// GitHub repository name
pub const GITHUB_REPO: &str = "riot-manifests";

/// Path to version manifests in the GitHub repository
pub const MANIFEST_PATH: &str = "LoL/EUW1/macos/lol-game-client";

/// The specific binary file we're looking for in the manifest
pub const TARGET_BINARY: &str = "LeagueofLegends.app/Contents/MacOS/LeagueofLegends";

/// Legacy version cutoff - versions at or below this are not processed
/// These use an older metaclass format that requires different parsing
pub const LEGACY_CUTOFF: &str = "13.14.5227601";

/// Configuration for meta-sync operations
#[derive(Debug, Clone)]
pub struct Config {
    /// Directory where version dumps are stored
    pub dumps_dir: PathBuf,
    
    /// Temporary directory for downloaded binaries
    pub temp_dir: PathBuf,
    
    /// Path to the dumper executable
    pub dumper_path: PathBuf,
}

impl Config {
    /// Creates a new configuration with default paths
    pub fn new() -> Self {
        let workspace_root = std::env::current_dir()
            .expect("Failed to get current directory");

        Self {
            dumps_dir: workspace_root.join("dumps"),
            temp_dir: workspace_root.join("temp"),
            dumper_path: Self::default_dumper_path(),
        }
    }

    /// Gets the default dumper executable path
    /// 
    /// By default, looks in the standard release directory.
    /// For cross-compilation, set the DUMPER_PATH environment variable.
    fn default_dumper_path() -> PathBuf {
        // Allow override via environment variable for cross-compilation scenarios
        if let Ok(path) = std::env::var("DUMPER_PATH") {
            return PathBuf::from(path);
        }

        let mut exe = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        exe.push("../../target/release");
        exe.push("dumper");
        
        #[cfg(target_os = "windows")]
        exe.set_extension("exe");
        
        exe
    }

    /// Returns the path for a version dump file
    pub fn dump_path(&self, version: &str) -> PathBuf {
        self.dumps_dir.join(format!("{}.json", version))
    }

    /// Returns the directory that holds the PBE dump
    pub fn preview_dumps_dir(&self) -> PathBuf {
        self.dumps_dir.join("pbe")
    }

    /// Returns the path for a temporary binary file
    pub fn temp_binary_path(&self, version: &str) -> PathBuf {
        self.temp_dir.join(version)
    }

    /// Checks if a dump already exists for a version
    pub fn has_dump(&self, version: &str) -> bool {
        self.dump_path(version).exists()
    }

    /// Ensures necessary directories exist
    pub fn ensure_directories(&self) -> std::io::Result<()> {
        std::fs::create_dir_all(&self.dumps_dir)?;
        std::fs::create_dir_all(&self.temp_dir)?;
        Ok(())
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}
