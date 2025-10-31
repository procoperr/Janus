//! Progress reporting using indicatif
//!
//! This module provides a thin wrapper around `indicatif` for displaying
//! progress information during long-running operations like scanning and syncing.
//!
//! ## Design
//!
//! - Minimal overhead when progress is disabled
//! - Support for multiple concurrent progress bars
//! - Integration with rayon for parallel operations
//! - Clean output that can be disabled for scripting

use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::sync::Arc;
use std::time::Duration;

/// Progress reporter for tracking long-running operations
///
/// This struct manages progress bars for various operations like scanning
/// directories, hashing files, and copying data.
///
/// # Example
///
/// ```no_run
/// use janus::progress::ProgressReporter;
///
/// let reporter = ProgressReporter::new();
/// let pb = reporter.add_task("Scanning", 100);
///
/// for i in 0..100 {
///     pb.inc(1);
/// }
/// pb.finish_with_message("Done");
/// ```
pub struct ProgressReporter {
    multi: Arc<MultiProgress>,
    enabled: bool,
}

impl ProgressReporter {
    /// Create a new progress reporter
    pub fn new() -> Self {
        Self {
            multi: Arc::new(MultiProgress::new()),
            enabled: true,
        }
    }

    /// Create a disabled progress reporter (no output)
    pub fn disabled() -> Self {
        Self {
            multi: Arc::new(MultiProgress::new()),
            enabled: false,
        }
    }

    /// Add a new progress task with a known total
    ///
    /// # Arguments
    ///
    /// * `name` - Name of the task (e.g., "Scanning files")
    /// * `total` - Total number of items to process
    ///
    /// # Returns
    ///
    /// A ProgressBar that can be updated as work progresses
    pub fn add_task(&self, name: &str, total: u64) -> ProgressBar {
        if !self.enabled {
            return ProgressBar::hidden();
        }

        let pb = self.multi.add(ProgressBar::new(total));
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{msg} [{bar:40.cyan/blue}] {pos}/{len} ({percent}%) [{elapsed_precise}]")
                .unwrap()
                .progress_chars("=>-"),
        );
        pb.set_message(name.to_string());
        pb
    }

    /// Add a spinner for indeterminate progress
    ///
    /// Use this when you don't know the total amount of work.
    pub fn add_spinner(&self, name: &str) -> ProgressBar {
        if !self.enabled {
            return ProgressBar::hidden();
        }

        let pb = self.multi.add(ProgressBar::new_spinner());
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.green} {msg} [{elapsed_precise}]")
                .unwrap(),
        );
        pb.set_message(name.to_string());
        pb.enable_steady_tick(Duration::from_millis(100));
        pb
    }

    /// Add a progress bar for byte-based operations (file copying, hashing)
    ///
    /// This uses human-readable byte formatting (KB, MB, GB).
    pub fn add_bytes_task(&self, name: &str, total_bytes: u64) -> ProgressBar {
        if !self.enabled {
            return ProgressBar::hidden();
        }

        let pb = self.multi.add(ProgressBar::new(total_bytes));
        pb.set_style(
            ProgressStyle::default_bar()
                .template(
                    "{msg} [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({percent}%) [{elapsed_precise}] {bytes_per_sec}",
                )
                .unwrap()
                .progress_chars("=>-"),
        );
        pb.set_message(name.to_string());
        pb
    }

    /// Print a message without disrupting progress bars
    pub fn println(&self, msg: &str) {
        if self.enabled {
            self.multi.println(msg).ok();
        } else {
            println!("{msg}");
        }
    }

    /// Check if progress reporting is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
}

impl Default for ProgressReporter {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper for rayon progress tracking
///
/// This struct can be cloned and used across rayon threads to update
/// a shared progress bar.
#[derive(Clone)]
pub struct ParallelProgress {
    pb: ProgressBar,
}

impl ParallelProgress {
    /// Create a new parallel progress tracker
    pub fn new(pb: ProgressBar) -> Self {
        Self { pb }
    }

    /// Increment progress by one
    pub fn inc(&self) {
        self.pb.inc(1);
    }

    /// Increment progress by a specific amount
    pub fn inc_by(&self, delta: u64) {
        self.pb.inc(delta);
    }

    /// Set progress to a specific position
    pub fn set_position(&self, pos: u64) {
        self.pb.set_position(pos);
    }

    /// Update the message
    pub fn set_message(&self, msg: String) {
        self.pb.set_message(msg);
    }

    /// Mark as finished
    pub fn finish(&self) {
        self.pb.finish();
    }

    /// Mark as finished with a custom message
    pub fn finish_with_message(&self, msg: &str) {
        self.pb.finish_with_message(msg.to_string());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reporter_creation() {
        let reporter = ProgressReporter::new();
        assert!(reporter.is_enabled());

        let disabled = ProgressReporter::disabled();
        assert!(!disabled.is_enabled());
    }

    #[test]
    fn test_add_task() {
        let reporter = ProgressReporter::new();
        let pb = reporter.add_task("Test task", 100);
        pb.inc(50);
        assert_eq!(pb.position(), 50);
        pb.finish();
    }

    #[test]
    fn test_disabled_progress() {
        let reporter = ProgressReporter::disabled();
        let pb = reporter.add_task("Test task", 100);
        // Should work but not display anything
        pb.inc(50);
        pb.finish();
    }

    #[test]
    fn test_bytes_task() {
        let reporter = ProgressReporter::new();
        let pb = reporter.add_bytes_task("Copying", 1024 * 1024);
        pb.inc(512 * 1024);
        assert_eq!(pb.position(), 512 * 1024);
        pb.finish();
    }

    #[test]
    fn test_parallel_progress() {
        let reporter = ProgressReporter::new();
        let pb = reporter.add_task("Parallel task", 100);
        let parallel = ParallelProgress::new(pb.clone());

        parallel.inc();
        assert_eq!(pb.position(), 1);

        parallel.inc_by(10);
        assert_eq!(pb.position(), 11);

        parallel.finish();
    }

    #[test]
    fn test_spinner() {
        let reporter = ProgressReporter::new();
        let spinner = reporter.add_spinner("Working");
        spinner.finish_with_message("Complete");
    }
}
