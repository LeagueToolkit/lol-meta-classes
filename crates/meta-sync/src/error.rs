//! Error types and error handling utilities for meta-sync.

use std::path::PathBuf;
use thiserror::Error;

/// Main error type for meta-sync operations.
#[derive(Error, Debug)]
pub enum SyncError {
    #[error("GitHub API error: {0}")]
    GitHub(#[from] octocrab::Error),

    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Sieve lookup failed: {0}")]
    Sieve(#[from] meta_sync::sieve::SieveError),

    #[error("Dump {path} was rejected: {reason}")]
    InvalidDump { path: PathBuf, reason: String },

    #[error("RMAN manifest parsing failed: {0}")]
    Manifest(String),

    #[error("Failed to download binary file: {0}")]
    Download(String),

    #[error("Dumper execution failed: {message}\nStderr: {stderr}")]
    DumperFailed { message: String, stderr: String },

    #[error("Dumper executable not found at: {path}")]
    DumperNotFound { path: PathBuf },

    #[error("Invalid version format: {0}")]
    InvalidVersion(String),

    #[error("Binary file not found in manifest: {expected}")]
    BinaryNotFound { expected: String },

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Temporary file operation failed: {0}")]
    TempFile(String),
}

/// Result type alias for meta-sync operations.
pub type Result<T> = std::result::Result<T, SyncError>;
