//! Integration tests for end-to-end sync operations

use janus::core::{diff_scans, scan_directory, sync_changes, SyncOptions};
use std::fs;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;
use tempfile::TempDir;

/// Helper to create a test file with content
fn create_file(dir: &Path, rel_path: &str, content: &[u8]) -> PathBuf {
    let path = dir.join(rel_path);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(&path, content).unwrap();
    path
}

/// Helper to verify file content
fn assert_file_content(path: &Path, expected: &[u8]) {
    let actual = fs::read(path).unwrap();
    assert_eq!(actual, expected, "File content mismatch at {}", path.display());
}

#[test]
fn test_basic_scan() {
    let temp_dir = TempDir::new().unwrap();

    create_file(temp_dir.path(), "file1.txt", b"content1");
    create_file(temp_dir.path(), "file2.txt", b"content2");
    create_file(temp_dir.path(), "subdir/file3.txt", b"content3");

    let scan = scan_directory(temp_dir.path(), None).unwrap();

    assert_eq!(scan.files.len(), 3, "Should find all three files");
    assert_eq!(scan.root, temp_dir.path());
}

#[test]
fn test_scan_empty_directory() {
    let temp_dir = TempDir::new().unwrap();

    let scan = scan_directory(temp_dir.path(), None).unwrap();

    assert_eq!(scan.files.len(), 0, "Empty directory should have no files");
}

#[test]
fn test_diff_identical_directories() {
    let source = TempDir::new().unwrap();
    let dest = TempDir::new().unwrap();

    create_file(source.path(), "file.txt", b"content");
    create_file(dest.path(), "file.txt", b"content");

    let source_scan = scan_directory(source.path(), None).unwrap();
    let dest_scan = scan_directory(dest.path(), None).unwrap();

    let diff = diff_scans(&source_scan, &dest_scan).unwrap();

    assert_eq!(diff.added.len(), 0);
    assert_eq!(diff.removed.len(), 0);
    assert_eq!(diff.modified.len(), 0);
    assert_eq!(diff.renamed.len(), 0);
}

#[test]
fn test_diff_added_files() {
    let source = TempDir::new().unwrap();
    let dest = TempDir::new().unwrap();

    create_file(source.path(), "new_file.txt", b"new content");

    let source_scan = scan_directory(source.path(), None).unwrap();
    let dest_scan = scan_directory(dest.path(), None).unwrap();

    let diff = diff_scans(&source_scan, &dest_scan).unwrap();

    assert_eq!(diff.added.len(), 1);
    assert_eq!(diff.added[0].path, PathBuf::from("new_file.txt"));
}

#[test]
fn test_diff_removed_files() {
    let source = TempDir::new().unwrap();
    let dest = TempDir::new().unwrap();

    create_file(dest.path(), "old_file.txt", b"old content");

    let source_scan = scan_directory(source.path(), None).unwrap();
    let dest_scan = scan_directory(dest.path(), None).unwrap();

    let diff = diff_scans(&source_scan, &dest_scan).unwrap();

    assert_eq!(diff.removed.len(), 1);
    assert_eq!(diff.removed[0].path, PathBuf::from("old_file.txt"));
}

#[test]
fn test_diff_modified_files() {
    let source = TempDir::new().unwrap();
    let dest = TempDir::new().unwrap();

    create_file(source.path(), "file.txt", b"new content");
    create_file(dest.path(), "file.txt", b"old content");

    let source_scan = scan_directory(source.path(), None).unwrap();
    let dest_scan = scan_directory(dest.path(), None).unwrap();

    let diff = diff_scans(&source_scan, &dest_scan).unwrap();

    assert_eq!(diff.modified.len(), 1);
    assert_eq!(diff.modified[0].path, PathBuf::from("file.txt"));
}

#[test]
#[cfg(not(target_os = "windows"))]
fn test_sync_new_files() {
    let source = TempDir::new().unwrap();
    let dest = TempDir::new().unwrap();

    create_file(source.path(), "new_file.txt", b"new content");
    create_file(source.path(), "subdir/nested.txt", b"nested content");

    let source_scan = scan_directory(source.path(), None).unwrap();
    let dest_scan = scan_directory(dest.path(), None).unwrap();
    let diff = diff_scans(&source_scan, &dest_scan).unwrap();

    let options = SyncOptions::default();
    sync_changes(source.path(), dest.path(), &diff, &options, None).unwrap();

    // Verify files were copied
    assert_file_content(&dest.path().join("new_file.txt"), b"new content");
    assert_file_content(&dest.path().join("subdir/nested.txt"), b"nested content");
}

#[test]
#[cfg(not(target_os = "windows"))]
fn test_sync_modified_files() {
    let source = TempDir::new().unwrap();
    let dest = TempDir::new().unwrap();

    create_file(source.path(), "file.txt", b"updated content");
    create_file(dest.path(), "file.txt", b"old content");

    let source_scan = scan_directory(source.path(), None).unwrap();
    let dest_scan = scan_directory(dest.path(), None).unwrap();
    let diff = diff_scans(&source_scan, &dest_scan).unwrap();

    let options = SyncOptions::default();
    sync_changes(source.path(), dest.path(), &diff, &options, None).unwrap();

    // Verify file was updated
    assert_file_content(&dest.path().join("file.txt"), b"updated content");
}

#[test]
fn test_sync_with_delete() {
    let source = TempDir::new().unwrap();
    let dest = TempDir::new().unwrap();

    create_file(source.path(), "keep.txt", b"keep this");
    create_file(dest.path(), "keep.txt", b"keep this");
    create_file(dest.path(), "delete.txt", b"remove this");

    let source_scan = scan_directory(source.path(), None).unwrap();
    let dest_scan = scan_directory(dest.path(), None).unwrap();
    let diff = diff_scans(&source_scan, &dest_scan).unwrap();

    let options = SyncOptions {
        delete_removed: true,
        ..Default::default()
    };
    sync_changes(source.path(), dest.path(), &diff, &options, None).unwrap();

    // Verify file was deleted
    assert!(!dest.path().join("delete.txt").exists());
    assert!(dest.path().join("keep.txt").exists());
}

#[test]
#[cfg(not(target_os = "windows"))]
fn test_sync_without_delete() {
    let source = TempDir::new().unwrap();
    let dest = TempDir::new().unwrap();

    create_file(dest.path(), "old_file.txt", b"old content");

    let source_scan = scan_directory(source.path(), None).unwrap();
    let dest_scan = scan_directory(dest.path(), None).unwrap();
    let diff = diff_scans(&source_scan, &dest_scan).unwrap();

    let options = SyncOptions {
        delete_removed: false,
        ..Default::default()
    };
    sync_changes(source.path(), dest.path(), &diff, &options, None).unwrap();

    // File should still exist
    assert!(dest.path().join("old_file.txt").exists());
}

#[test]
#[cfg(not(target_os = "windows"))]
fn test_rename_detection_in_sync() {
    let source = TempDir::new().unwrap();
    let dest = TempDir::new().unwrap();

    let content = b"unique file content for rename detection";
    create_file(source.path(), "new_name.txt", content);
    create_file(dest.path(), "old_name.txt", content);

    let source_scan = scan_directory(source.path(), None).unwrap();
    let dest_scan = scan_directory(dest.path(), None).unwrap();
    let diff = diff_scans(&source_scan, &dest_scan).unwrap();

    assert_eq!(diff.renamed.len(), 1, "Should detect rename");

    let options = SyncOptions::default();
    sync_changes(source.path(), dest.path(), &diff, &options, None).unwrap();

    // After sync, new name should exist, old name should not
    assert!(dest.path().join("new_name.txt").exists());
    assert!(!dest.path().join("old_name.txt").exists());
    assert_file_content(&dest.path().join("new_name.txt"), content);
}

#[test]
#[cfg(not(target_os = "windows"))]
fn test_complex_sync_scenario() {
    let source = TempDir::new().unwrap();
    let dest = TempDir::new().unwrap();

    // Setup complex scenario
    create_file(source.path(), "added.txt", b"new file");
    create_file(source.path(), "modified.txt", b"updated content");
    create_file(source.path(), "renamed_new.txt", b"renamed content");
    create_file(source.path(), "unchanged.txt", b"same content");

    create_file(dest.path(), "modified.txt", b"old content");
    create_file(dest.path(), "renamed_old.txt", b"renamed content");
    create_file(dest.path(), "removed.txt", b"will be deleted");
    create_file(dest.path(), "unchanged.txt", b"same content");

    let source_scan = scan_directory(source.path(), None).unwrap();
    let dest_scan = scan_directory(dest.path(), None).unwrap();
    let diff = diff_scans(&source_scan, &dest_scan).unwrap();

    // Verify diff results
    assert_eq!(diff.added.len(), 1);
    assert_eq!(diff.modified.len(), 1);
    assert_eq!(diff.renamed.len(), 1);
    assert_eq!(diff.removed.len(), 1);

    let options = SyncOptions {
        delete_removed: true,
        ..Default::default()
    };
    sync_changes(source.path(), dest.path(), &diff, &options, None).unwrap();

    // Verify final state
    assert!(dest.path().join("added.txt").exists());
    assert!(dest.path().join("modified.txt").exists());
    assert!(dest.path().join("renamed_new.txt").exists());
    assert!(dest.path().join("unchanged.txt").exists());
    assert!(!dest.path().join("renamed_old.txt").exists());
    assert!(!dest.path().join("removed.txt").exists());

    assert_file_content(&dest.path().join("added.txt"), b"new file");
    assert_file_content(&dest.path().join("modified.txt"), b"updated content");
    assert_file_content(&dest.path().join("renamed_new.txt"), b"renamed content");
}

#[test]
#[cfg(not(target_os = "windows"))]
fn test_preserve_timestamps() {
    let source = TempDir::new().unwrap();
    let dest = TempDir::new().unwrap();

    let source_file = create_file(source.path(), "file.txt", b"content");

    // Get original timestamp
    let original_mtime = fs::metadata(&source_file).unwrap().modified().unwrap();

    // Wait a bit to ensure time difference
    thread::sleep(Duration::from_millis(10));

    let source_scan = scan_directory(source.path(), None).unwrap();
    let dest_scan = scan_directory(dest.path(), None).unwrap();
    let diff = diff_scans(&source_scan, &dest_scan).unwrap();

    let options = SyncOptions {
        preserve_timestamps: true,
        ..Default::default()
    };
    sync_changes(source.path(), dest.path(), &diff, &options, None).unwrap();

    let dest_file = dest.path().join("file.txt");
    let dest_mtime = fs::metadata(&dest_file).unwrap().modified().unwrap();

    // Timestamps should be close (within 1 second due to filesystem precision)
    let diff = dest_mtime
        .duration_since(original_mtime)
        .unwrap_or_else(|_| original_mtime.duration_since(dest_mtime).unwrap());
    assert!(diff.as_secs() < 2, "Timestamps should be preserved");
}

#[test]
#[cfg(not(target_os = "windows"))]
fn test_nested_directories() {
    let source = TempDir::new().unwrap();
    let dest = TempDir::new().unwrap();

    create_file(source.path(), "a/b/c/deep.txt", b"deep content");
    create_file(source.path(), "x/y/file.txt", b"other content");

    let source_scan = scan_directory(source.path(), None).unwrap();
    let dest_scan = scan_directory(dest.path(), None).unwrap();
    let diff = diff_scans(&source_scan, &dest_scan).unwrap();

    let options = SyncOptions::default();
    sync_changes(source.path(), dest.path(), &diff, &options, None).unwrap();

    // Verify nested directories were created
    assert!(dest.path().join("a/b/c/deep.txt").exists());
    assert!(dest.path().join("x/y/file.txt").exists());
    assert_file_content(&dest.path().join("a/b/c/deep.txt"), b"deep content");
}

#[test]
#[cfg(not(target_os = "windows"))]
fn test_large_file_sync() {
    let source = TempDir::new().unwrap();
    let dest = TempDir::new().unwrap();

    // Create a 1MB file
    let large_content = vec![0x42u8; 1024 * 1024];
    let source_file = source.path().join("large.bin");
    fs::write(&source_file, &large_content).unwrap();

    let source_scan = scan_directory(source.path(), None).unwrap();
    let dest_scan = scan_directory(dest.path(), None).unwrap();
    let diff = diff_scans(&source_scan, &dest_scan).unwrap();

    let options = SyncOptions::default();
    sync_changes(source.path(), dest.path(), &diff, &options, None).unwrap();

    // Verify large file was copied correctly
    let dest_file = dest.path().join("large.bin");
    let dest_content = fs::read(&dest_file).unwrap();
    assert_eq!(dest_content.len(), large_content.len());
    assert_eq!(dest_content, large_content);
}

#[test]
#[cfg(not(target_os = "windows"))]
fn test_empty_file_sync() {
    let source = TempDir::new().unwrap();
    let dest = TempDir::new().unwrap();

    create_file(source.path(), "empty.txt", b"");

    let source_scan = scan_directory(source.path(), None).unwrap();
    let dest_scan = scan_directory(dest.path(), None).unwrap();
    let diff = diff_scans(&source_scan, &dest_scan).unwrap();

    let options = SyncOptions::default();
    sync_changes(source.path(), dest.path(), &diff, &options, None).unwrap();

    // Verify empty file was copied
    let dest_file = dest.path().join("empty.txt");
    assert!(dest_file.exists());
    assert_eq!(fs::read(&dest_file).unwrap().len(), 0);
}

#[test]
fn test_no_changes_sync() {
    let source = TempDir::new().unwrap();
    let dest = TempDir::new().unwrap();

    create_file(source.path(), "file.txt", b"content");
    create_file(dest.path(), "file.txt", b"content");

    let source_scan = scan_directory(source.path(), None).unwrap();
    let dest_scan = scan_directory(dest.path(), None).unwrap();
    let diff = diff_scans(&source_scan, &dest_scan).unwrap();

    // No changes, so this should complete without error
    let options = SyncOptions::default();
    sync_changes(source.path(), dest.path(), &diff, &options, None).unwrap();

    // File should still exist and be unchanged
    assert_file_content(&dest.path().join("file.txt"), b"content");
}
