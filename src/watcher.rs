use std::path::Path;
use std::time::Duration;

use notify_debouncer_mini::{new_debouncer, DebouncedEventKind};
use tokio::sync::mpsc;

/// Set up a file system watcher that sends a signal on changes (100ms debounce).
pub fn setup_watcher(
    root: &Path,
    tx: mpsc::Sender<()>,
) -> Option<notify_debouncer_mini::Debouncer<notify::RecommendedWatcher>> {
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
                eprintln!("croot: failed to watch {}: {e}", root.display());
                return None;
            }
            Some(d)
        }
        Err(e) => {
            eprintln!("croot: failed to initialize file watcher: {e}");
            None
        }
    }
}
