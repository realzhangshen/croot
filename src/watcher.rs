use std::path::Path;
use std::time::Duration;

use notify_debouncer_mini::{new_debouncer, DebouncedEventKind};
use tokio::sync::mpsc;

/// Result of watcher setup: the debouncer (if successful) and an optional error message
/// for display in the status bar.
pub struct WatcherResult {
    pub debouncer: Option<notify_debouncer_mini::Debouncer<notify::RecommendedWatcher>>,
    pub error: Option<String>,
}

/// Set up a file system watcher that sends a signal on changes (100ms debounce).
/// Returns a `WatcherResult` with an error message suitable for the status bar
/// instead of printing to stderr (which is invisible in TUI alternate screen mode).
pub fn setup_watcher(root: &Path, tx: mpsc::Sender<()>) -> WatcherResult {
    let debouncer = new_debouncer(
        Duration::from_millis(100),
        move |events: Result<Vec<notify_debouncer_mini::DebouncedEvent>, notify::Error>| {
            if let Ok(events) = events {
                let has_real_change = events.iter().any(|e| e.kind == DebouncedEventKind::Any);
                if has_real_change {
                    let _ = tx.try_send(());
                }
            }
        },
    );

    match debouncer {
        Ok(mut d) => {
            if let Err(e) = d.watcher().watch(root, notify::RecursiveMode::Recursive) {
                return WatcherResult {
                    debouncer: None,
                    error: Some(format!("Failed to watch {}: {e}", root.display())),
                };
            }
            WatcherResult {
                debouncer: Some(d),
                error: None,
            }
        }
        Err(e) => WatcherResult {
            debouncer: None,
            error: Some(format!("Failed to initialize file watcher: {e}")),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn setup_watcher_returns_error_for_nonexistent_path() {
        let (tx, _rx) = mpsc::channel(1);
        let result = setup_watcher(Path::new("/nonexistent/path/croot_test"), tx);
        // Should fail gracefully with an error message, not panic
        assert!(result.debouncer.is_none());
        assert!(result.error.is_some());
        assert!(result.error.unwrap().contains("Failed to"));
    }

    #[test]
    fn setup_watcher_succeeds_for_valid_path() {
        let tmp = tempfile::tempdir().unwrap();
        let (tx, _rx) = mpsc::channel(1);
        let result = setup_watcher(tmp.path(), tx);
        assert!(result.error.is_none());
        assert!(result.debouncer.is_some());
    }
}
