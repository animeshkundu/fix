//! Progress indicators for long-running operations
//!
//! This module provides a wrapper around indicatif's ProgressBar to show
//! spinners and status messages during operations that take >100ms.

use indicatif::{ProgressBar, ProgressStyle};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

/// A progress spinner that can be shown or hidden based on operation duration
pub struct ProgressSpinner {
    pb: Option<ProgressBar>,
    start_time: Instant,
    quiet: bool,
    shown: Arc<AtomicBool>,
}

impl ProgressSpinner {
    /// Create a new progress spinner
    ///
    /// The spinner will only be shown if the operation takes longer than 100ms.
    ///
    /// # Arguments
    ///
    /// * `quiet` - If true, the spinner will never be shown
    pub fn new(quiet: bool) -> Self {
        Self {
            pb: None,
            start_time: Instant::now(),
            quiet,
            shown: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Set the message displayed by the spinner
    ///
    /// If enough time has elapsed (>100ms) and we're not in quiet mode,
    /// this will show the spinner with the given message.
    pub fn set_message(&mut self, msg: &str) {
        if self.quiet {
            return;
        }

        // Only show spinner if operation has taken more than 100ms
        if self.start_time.elapsed() > Duration::from_millis(100) && self.pb.is_none() {
            let pb = ProgressBar::new_spinner();
            pb.set_style(
                ProgressStyle::default_spinner()
                    .template("{spinner:.cyan} {msg}")
                    .unwrap()
            );
            pb.enable_steady_tick(Duration::from_millis(100));
            self.pb = Some(pb);
            self.shown.store(true, Ordering::Relaxed);
        }

        if let Some(ref pb) = self.pb {
            pb.set_message(msg.to_string());
        }
    }

    /// Finish the spinner with a completion message
    ///
    /// The spinner is hidden and the message is printed.
    pub fn finish_with_message(&self, msg: &str) {
        if let Some(ref pb) = self.pb {
            pb.finish_and_clear();
        }
        if self.shown.load(Ordering::Relaxed) && !self.quiet {
            eprintln!("{}", msg);
        }
    }

    /// Finish the spinner silently
    ///
    /// The spinner is hidden without printing any message.
    pub fn finish(&self) {
        if let Some(ref pb) = self.pb {
            pb.finish_and_clear();
        }
    }

    /// Check if the spinner is currently visible
    pub fn is_shown(&self) -> bool {
        self.shown.load(Ordering::Relaxed)
    }
}

impl Drop for ProgressSpinner {
    fn drop(&mut self) {
        if let Some(ref pb) = self.pb {
            pb.finish_and_clear();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_spinner_created_not_quiet() {
        let spinner = ProgressSpinner::new(false);
        assert!(!spinner.quiet);
        assert!(!spinner.is_shown());
    }

    #[test]
    fn test_spinner_created_quiet() {
        let spinner = ProgressSpinner::new(true);
        assert!(spinner.quiet);
        assert!(!spinner.is_shown());
    }

    #[test]
    fn test_spinner_not_shown_immediately() {
        let mut spinner = ProgressSpinner::new(false);
        spinner.set_message("Test message");
        // Should not show immediately (< 100ms)
        assert!(!spinner.is_shown());
    }

    #[test]
    fn test_spinner_shown_after_delay() {
        let mut spinner = ProgressSpinner::new(false);
        // Wait for 150ms to exceed the 100ms threshold
        thread::sleep(Duration::from_millis(150));
        spinner.set_message("Test message");
        // Should be shown after delay
        assert!(spinner.is_shown());
    }

    #[test]
    fn test_spinner_not_shown_in_quiet_mode() {
        let mut spinner = ProgressSpinner::new(true);
        thread::sleep(Duration::from_millis(150));
        spinner.set_message("Test message");
        // Should not be shown in quiet mode
        assert!(!spinner.is_shown());
    }

    #[test]
    fn test_spinner_finish() {
        let mut spinner = ProgressSpinner::new(false);
        thread::sleep(Duration::from_millis(150));
        spinner.set_message("Test message");
        assert!(spinner.is_shown());
        spinner.finish();
        // Spinner should still be marked as shown after finish
        assert!(spinner.is_shown());
    }

    #[test]
    fn test_spinner_finish_with_message() {
        let mut spinner = ProgressSpinner::new(false);
        thread::sleep(Duration::from_millis(150));
        spinner.set_message("Working...");
        assert!(spinner.is_shown());
        spinner.finish_with_message("✓ Done");
        // Spinner should still be marked as shown after finish
        assert!(spinner.is_shown());
    }

    #[test]
    fn test_spinner_multiple_messages() {
        let mut spinner = ProgressSpinner::new(false);
        thread::sleep(Duration::from_millis(150));
        spinner.set_message("First message");
        spinner.set_message("Second message");
        spinner.set_message("Third message");
        assert!(spinner.is_shown());
    }
}
