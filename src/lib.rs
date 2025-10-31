//! Beautifully fast, simple & reliable file syncing.

pub mod core;
pub mod hash;
pub mod io;
pub mod progress;

pub use core::{
    diff_scans, scan_directory, sync_changes, DiffResult, FileMeta, ScanResult, SyncOptions,
};
pub use hash::{hash_bytes, hash_file, ContentHash, Hasher};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
