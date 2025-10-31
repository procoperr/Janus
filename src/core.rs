//! Core synchronization logic for scanning, diffing, and syncing directories.

use crate::hash::{ContentHash, Hasher};
use crate::io::{copy_file_with_metadata, remove_file_safe};
use crate::progress::ProgressReporter;
use anyhow::Result;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use thiserror::Error;

/// Errors that can occur during synchronization operations
#[derive(Error, Debug)]
pub enum SyncError {
    #[error("Failed to read directory: {0}")]
    DirectoryRead(String),

    #[error("Failed to hash file: {0}")]
    HashError(String),

    #[error("Failed to copy file: {0}")]
    CopyError(String),

    #[error("Invalid path: {0}")]
    InvalidPath(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

/// Metadata for a single file including content hash
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMeta {
    /// Relative path from scan root
    pub path: PathBuf,
    /// File size in bytes
    pub size: u64,
    /// Last modified time
    #[serde(with = "systemtime_serde")]
    pub mtime: SystemTime,
    /// Content hash (BLAKE3 or SHA-256)
    pub hash: ContentHash,
    /// Unix permissions (if available)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permissions: Option<u32>,
}

// Helper module for SystemTime serialization
mod systemtime_serde {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    pub fn serialize<S>(time: &SystemTime, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let duration = time.duration_since(UNIX_EPOCH).unwrap_or(Duration::from_secs(0));
        duration.as_secs().serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<SystemTime, D::Error>
    where
        D: Deserializer<'de>,
    {
        let secs = u64::deserialize(deserializer)?;
        Ok(UNIX_EPOCH + Duration::from_secs(secs))
    }
}

/// Result of scanning a directory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResult {
    /// Root directory that was scanned
    pub root: PathBuf,
    /// List of all files found
    pub files: Vec<FileMeta>,
    /// Timestamp when scan was performed
    #[serde(with = "systemtime_serde")]
    pub scan_time: SystemTime,
}

impl ScanResult {
    /// Calculate total size of all files
    pub fn total_size(&self) -> u64 {
        self.files.iter().map(|f| f.size).sum()
    }

    /// Save scan results to a JSON file
    pub fn save_to_file(&self, path: &Path) -> Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        fs::write(path, json)?;
        Ok(())
    }

    /// Load scan results from a JSON file
    pub fn load_from_file(path: &Path) -> Result<Self> {
        let json = fs::read_to_string(path)?;
        let scan = serde_json::from_str(&json)?;
        Ok(scan)
    }
}

/// Result of comparing two scans
#[derive(Debug, Clone)]
pub struct DiffResult {
    /// Files present in source but not in destination
    pub added: Vec<FileMeta>,
    /// Files present in destination but not in source
    pub removed: Vec<FileMeta>,
    /// Files present in both but with different content
    pub modified: Vec<FileMeta>,
    /// Files that were renamed (old, new)
    pub renamed: Vec<(FileMeta, FileMeta)>,
}

/// Options for sync operations
#[derive(Debug, Clone)]
pub struct SyncOptions {
    /// Delete files in destination not present in source
    pub delete_removed: bool,
    /// Preserve file timestamps
    pub preserve_timestamps: bool,
    /// Verify file hash after copying
    pub verify_after_copy: bool,
}

impl Default for SyncOptions {
    fn default() -> Self {
        Self {
            delete_removed: false,
            preserve_timestamps: true,
            verify_after_copy: false,
        }
    }
}

/// Scan a directory and compute content hashes for all files
///
/// This function walks the directory tree in parallel, computing content hashes
/// for each file using streaming I/O to minimize memory usage.
///
/// # Arguments
///
/// * `root` - Root directory to scan
/// * `progress` - Optional progress reporter
///
/// # Performance
///
/// - Uses `ignore` crate for parallel directory traversal
/// - Hashes files in parallel using `rayon`
/// - Streaming hash computation for constant memory usage
/// - Respects .gitignore patterns for efficiency
pub fn scan_directory(root: &Path, progress: Option<&ProgressReporter>) -> Result<ScanResult> {
    if !root.exists() {
        return Err(SyncError::InvalidPath(format!(
            "Directory does not exist: {}",
            root.display()
        ))
        .into());
    }

    if progress.is_some() {
        println!("Scanning: {}", root.display());
    }

    // Collect all file paths first
    let walker = ignore::WalkBuilder::new(root)
        .hidden(false)
        .git_ignore(true)
        .git_exclude(true)
        .threads(num_cpus::get())
        .build_parallel();

    let files = std::sync::Mutex::new(Vec::new());

    walker.run(|| {
        Box::new(|entry_result| {
            if let Ok(entry) = entry_result {
                if let Some(file_type) = entry.file_type() {
                    if file_type.is_file() {
                        files.lock().unwrap().push(entry.path().to_path_buf());
                    }
                }
            }
            ignore::WalkState::Continue
        })
    });

    let file_paths = files.into_inner().unwrap();
    let total_files = file_paths.len();

    if progress.is_some() {
        println!("Found {total_files} files, computing hashes...");
    }

    // Hash files in parallel
    let file_metas: Vec<Result<FileMeta>> = file_paths
        .par_iter()
        .map(|path| {
            let metadata = fs::metadata(path)?;
            let size = metadata.len();
            let mtime = metadata.modified()?;

            // Get permissions on Unix systems
            #[cfg(unix)]
            let permissions = {
                use std::os::unix::fs::PermissionsExt;
                Some(metadata.permissions().mode())
            };
            #[cfg(not(unix))]
            let permissions = None;

            // Compute content hash using streaming
            let mut hasher = Hasher::new();
            hasher.hash_file(path)?;
            let hash = hasher.finalize();

            // Make path relative to root
            let rel_path = path
                .strip_prefix(root)
                .map_err(|_| {
                    SyncError::InvalidPath(format!("Path not under root: {}", path.display()))
                })?
                .to_path_buf();

            Ok(FileMeta {
                path: rel_path,
                size,
                mtime,
                hash,
                permissions,
            })
        })
        .collect();

    // Collect results, logging errors but not failing the entire scan
    let mut successful_files = Vec::new();
    let mut error_count = 0;

    for result in file_metas {
        match result {
            Ok(meta) => successful_files.push(meta),
            Err(e) => {
                error_count += 1;
                eprintln!("Warning: Failed to process file: {e}");
            },
        }
    }

    if error_count > 0 {
        eprintln!("Warning: {error_count} files could not be processed");
    }

    Ok(ScanResult {
        root: root.to_path_buf(),
        files: successful_files,
        scan_time: SystemTime::now(),
    })
}

/// Compare two scan results and identify differences
///
/// This function performs intelligent diff computation with rename detection:
/// 1. Build hash maps for fast lookup
/// 2. Identify added/removed/modified files
/// 3. Detect renames by matching content hashes
/// 4. Use path similarity as fallback for ambiguous renames
///
/// # Performance
///
/// - O(n) hash map construction
/// - O(1) lookups for most operations
/// - Rename detection is O(n*m) worst case but typically O(n) with hash matching
pub fn diff_scans(source: &ScanResult, dest: &ScanResult) -> Result<DiffResult> {
    // Build hash maps for fast lookup
    let source_by_path: HashMap<&PathBuf, &FileMeta> =
        source.files.iter().map(|f| (&f.path, f)).collect();
    let dest_by_path: HashMap<&PathBuf, &FileMeta> =
        dest.files.iter().map(|f| (&f.path, f)).collect();

    // Build hash-to-files maps for rename detection
    let mut source_by_hash: HashMap<&ContentHash, Vec<&FileMeta>> = HashMap::new();
    for file in &source.files {
        source_by_hash.entry(&file.hash).or_default().push(file);
    }

    let mut dest_by_hash: HashMap<&ContentHash, Vec<&FileMeta>> = HashMap::new();
    for file in &dest.files {
        dest_by_hash.entry(&file.hash).or_default().push(file);
    }

    let mut added = Vec::new();
    let mut removed = Vec::new();
    let mut modified = Vec::new();
    let mut renamed = Vec::new();
    let mut processed_dest_paths = HashSet::new();

    // Find added and modified files
    for source_file in &source.files {
        if let Some(dest_file) = dest_by_path.get(&source_file.path) {
            // File exists in both locations
            if source_file.hash != dest_file.hash {
                // Content changed
                modified.push(source_file.clone());
            }
            processed_dest_paths.insert(&dest_file.path);
        } else {
            // File not at same path in destination
            // Check if it might be a rename (same content, different path)
            if let Some(dest_files_with_hash) = dest_by_hash.get(&source_file.hash) {
                // Find best match from files with same hash
                let mut best_match: Option<&FileMeta> = None;
                let mut best_score = 0.0;

                for candidate in dest_files_with_hash {
                    if processed_dest_paths.contains(&candidate.path) {
                        continue;
                    }

                    let score = path_similarity(&source_file.path, &candidate.path);
                    if score > best_score {
                        best_score = score;
                        best_match = Some(candidate);
                    }
                }

                if let Some(matched_dest) = best_match {
                    // Detected rename
                    renamed.push(((*matched_dest).clone(), source_file.clone()));
                    processed_dest_paths.insert(&matched_dest.path);
                } else {
                    // Hash matches but all candidates already processed - treat as new file
                    added.push(source_file.clone());
                }
            } else {
                // New file
                added.push(source_file.clone());
            }
        }
    }

    // Find removed files (in dest but not in source, and not part of a rename)
    for dest_file in &dest.files {
        if !source_by_path.contains_key(&dest_file.path)
            && !processed_dest_paths.contains(&dest_file.path)
        {
            removed.push(dest_file.clone());
        }
    }

    Ok(DiffResult { added, removed, modified, renamed })
}

/// Compute path similarity score between two paths (0.0 to 1.0)
///
/// Uses a simple token-based approach: compares path components and filenames.
/// Higher score indicates more similar paths.
fn path_similarity(path1: &Path, path2: &Path) -> f64 {
    let name1 = path1.file_name().and_then(|n| n.to_str()).unwrap_or("");
    let name2 = path2.file_name().and_then(|n| n.to_str()).unwrap_or("");

    // Exact filename match is a strong signal
    if name1 == name2 {
        return 0.9;
    }

    // Compute simple string similarity for filenames
    let filename_sim = simple_string_similarity(name1, name2);

    // Also consider directory similarity
    let dir1 = path1.parent().map(|p| p.to_string_lossy());
    let dir2 = path2.parent().map(|p| p.to_string_lossy());

    let dir_sim = match (dir1, dir2) {
        (Some(d1), Some(d2)) => simple_string_similarity(&d1, &d2),
        _ => 0.0,
    };

    // Weight filename more heavily than directory
    filename_sim * 0.7 + dir_sim * 0.3
}

/// Simple string similarity using character overlap (Jaccard-like)
fn simple_string_similarity(s1: &str, s2: &str) -> f64 {
    // Handle exact matches first (including empty strings)
    if s1 == s2 {
        return 1.0;
    }
    // If one is empty but not both (already handled above), they're completely different
    if s1.is_empty() || s2.is_empty() {
        return 0.0;
    }

    let chars1: HashSet<char> = s1.chars().collect();
    let chars2: HashSet<char> = s2.chars().collect();

    let intersection = chars1.intersection(&chars2).count();
    let union = chars1.union(&chars2).count();

    if union == 0 {
        0.0
    } else {
        intersection as f64 / union as f64
    }
}

/// Synchronize changes from source to destination based on diff results
///
/// This function applies the changes identified in a diff:
/// - Copies new and modified files
/// - Handles renames (moves files if possible, copies otherwise)
/// - Optionally deletes removed files
///
/// # Arguments
///
/// * `source_root` - Source directory root
/// * `dest_root` - Destination directory root
/// * `diff` - Diff results to apply
/// * `options` - Sync options
/// * `progress` - Optional progress reporter
pub fn sync_changes(
    source_root: &Path,
    dest_root: &Path,
    diff: &DiffResult,
    options: &SyncOptions,
    progress: Option<&ProgressReporter>,
) -> Result<()> {
    let total_ops = diff.added.len()
        + diff.modified.len()
        + diff.renamed.len()
        + if options.delete_removed {
            diff.removed.len()
        } else {
            0
        };

    if progress.is_some() {
        println!("Applying {total_ops} changes...");
    }

    // Copy new and modified files
    let files_to_copy: Vec<&FileMeta> = diff.added.iter().chain(diff.modified.iter()).collect();

    files_to_copy.par_iter().try_for_each(|file| {
        let source_path = source_root.join(&file.path);
        let dest_path = dest_root.join(&file.path);

        // Ensure parent directory exists
        if let Some(parent) = dest_path.parent() {
            fs::create_dir_all(parent)?;
        }

        copy_file_with_metadata(&source_path, &dest_path, options.preserve_timestamps)?;
        Ok::<_, anyhow::Error>(())
    })?;

    // Handle renames - for now, just copy to new location
    // TODO: Optimize by moving files when possible (requires checking if old location should be deleted)
    diff.renamed.par_iter().try_for_each(|(old, new)| {
        let source_path = source_root.join(&new.path);
        let dest_path = dest_root.join(&new.path);

        if let Some(parent) = dest_path.parent() {
            fs::create_dir_all(parent)?;
        }

        copy_file_with_metadata(&source_path, &dest_path, options.preserve_timestamps)?;

        // Remove old file in destination
        let old_dest_path = dest_root.join(&old.path);
        remove_file_safe(&old_dest_path)?;

        Ok::<_, anyhow::Error>(())
    })?;

    // Delete removed files if requested
    if options.delete_removed {
        for file in &diff.removed {
            let dest_path = dest_root.join(&file.path);
            remove_file_safe(&dest_path)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_similarity() {
        // Exact filename match
        let p1 = Path::new("dir1/file.txt");
        let p2 = Path::new("dir2/file.txt");
        assert!(path_similarity(p1, p2) > 0.8);

        // Different files in same directory (directory similarity pulls score up)
        let p1 = Path::new("dir/foo.txt");
        let p2 = Path::new("dir/bar.txt");
        assert!(path_similarity(p1, p2) > 0.3); // Same dir boosts similarity
        assert!(path_similarity(p1, p2) < 0.7); // But still not very similar
    }

    #[test]
    fn test_string_similarity() {
        assert_eq!(simple_string_similarity("hello", "hello"), 1.0);
        assert_eq!(simple_string_similarity("", ""), 1.0); // Equal empty strings
        assert!(simple_string_similarity("hello", "hallo") > 0.5);
    }
}
