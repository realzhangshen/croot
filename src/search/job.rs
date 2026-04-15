use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use tokio::io::AsyncBufReadExt;
use tokio::process::Command;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

use super::grouper::parse_rg_json_match;
use super::types::{GlobalSearchResult, GlobalSearchType};

/// A batch of search results sent from the background search task.
#[derive(Debug)]
pub struct SearchBatch {
    pub generation: u64,
    pub results: Vec<GlobalSearchResult>,
    pub is_final: bool,
    pub error: Option<String>,
}

/// A running search job that can be cancelled.
///
/// On drop, the job is automatically cancelled.
pub struct SearchJob {
    pub generation: u64,
    handle: JoinHandle<()>,
    cancelled: Arc<AtomicBool>,
}

impl SearchJob {
    /// Spawn a new search job.
    ///
    /// - `debounce_ms`: milliseconds to wait before actually launching the child process.
    /// - `batch_size`: number of results to accumulate before sending an intermediate batch.
    #[allow(clippy::too_many_arguments)]
    pub fn spawn(
        generation: u64,
        query: String,
        search_type: GlobalSearchType,
        root: PathBuf,
        fd_cmd: String,
        rg_cmd: String,
        max_results: usize,
        tx: mpsc::Sender<SearchBatch>,
        debounce_ms: u64,
    ) -> Self {
        let cancelled = Arc::new(AtomicBool::new(false));
        let flag = cancelled.clone();

        let handle = tokio::spawn(async move {
            // Debounce: sleep in small increments, checking cancellation
            let debounce = std::time::Duration::from_millis(debounce_ms);
            let step = std::time::Duration::from_millis(20);
            let mut elapsed = std::time::Duration::ZERO;
            while elapsed < debounce {
                if flag.load(Ordering::Relaxed) {
                    return;
                }
                tokio::time::sleep(step.min(debounce - elapsed)).await;
                elapsed += step;
            }
            if flag.load(Ordering::Relaxed) {
                return;
            }

            // Build command
            let child_result = match search_type {
                GlobalSearchType::FileName => {
                    let parts =
                        shell_words::split(&fd_cmd).unwrap_or_else(|_| vec![fd_cmd.clone()]);
                    let (bin, extra) = parts.split_first().unwrap_or((&fd_cmd, &[]));
                    Command::new(bin)
                        .args(extra)
                        .args(["--type", "f", "--color", "never", "--", &query])
                        .current_dir(&root)
                        .stdout(std::process::Stdio::piped())
                        .stderr(std::process::Stdio::piped())
                        .kill_on_drop(true)
                        .spawn()
                }
                GlobalSearchType::Content => {
                    let parts =
                        shell_words::split(&rg_cmd).unwrap_or_else(|_| vec![rg_cmd.clone()]);
                    let (bin, extra) = parts.split_first().unwrap_or((&rg_cmd, &[]));
                    Command::new(bin)
                        .args(extra)
                        .args(["--json", "--line-number", "--max-count", "20", "--", &query])
                        .current_dir(&root)
                        .stdout(std::process::Stdio::piped())
                        .stderr(std::process::Stdio::piped())
                        .kill_on_drop(true)
                        .spawn()
                }
            };

            let mut child = match child_result {
                Ok(child) => child,
                Err(e) => {
                    let cmd_name = match search_type {
                        GlobalSearchType::FileName => &fd_cmd,
                        GlobalSearchType::Content => &rg_cmd,
                    };
                    let _ = tx
                        .send(SearchBatch {
                            generation,
                            results: Vec::new(),
                            is_final: true,
                            error: Some(format!("{cmd_name}: {e}")),
                        })
                        .await;
                    return;
                }
            };

            let stdout = child.stdout.take().expect("stdout was piped");
            let reader = tokio::io::BufReader::new(stdout);
            let mut lines = reader.lines();

            let mut batch = Vec::new();
            let mut total_count = 0usize;
            let mut parse_failed = false;
            // For content search, cap by unique files, not raw matches.
            let mut unique_file_count = 0usize;
            let mut last_file: Option<String> = None;
            let mut capped = false;

            const BATCH_SIZE: usize = 50;

            loop {
                if flag.load(Ordering::Relaxed) {
                    let _ = child.kill().await;
                    return;
                }

                let line_result = lines.next_line().await;
                let line = match line_result {
                    Ok(Some(line)) => line,
                    Ok(None) => break, // EOF
                    Err(_) => break,
                };

                if line.is_empty() {
                    continue;
                }

                match search_type {
                    GlobalSearchType::FileName => {
                        if total_count >= max_results {
                            capped = true;
                            break;
                        }
                        let path = root.join(&line);
                        batch.push(GlobalSearchResult {
                            path,
                            display: line,
                            line: None,
                            context: None,
                        });
                        total_count += 1;
                    }
                    GlobalSearchType::Content => match parse_rg_json_match(&line) {
                        Ok(Some(m)) => {
                            let is_new_file = last_file.as_ref().is_none_or(|f| f != &m.file);
                            if is_new_file {
                                unique_file_count += 1;
                                if unique_file_count > max_results {
                                    capped = true;
                                    break;
                                }
                                last_file = Some(m.file.clone());
                            }
                            let path = root.join(&m.file);
                            batch.push(GlobalSearchResult {
                                path,
                                display: m.file,
                                line: m.line_number,
                                context: m.context,
                            });
                            total_count += 1;
                        }
                        Ok(None) => {}
                        Err(_) => {
                            parse_failed = true;
                        }
                    },
                }

                // Send intermediate batch
                if batch.len() >= BATCH_SIZE {
                    let intermediate = std::mem::take(&mut batch);
                    let _ = tx
                        .send(SearchBatch {
                            generation,
                            results: intermediate,
                            is_final: false,
                            error: None,
                        })
                        .await;
                }
            }

            // If capped, kill the child to stop further output
            if capped {
                let _ = child.kill().await;
            }

            // Wait for child to finish and check status
            let status = child.wait().await;
            let stderr_output = {
                // stderr was piped; read it now (child has exited or been killed)
                let mut stderr_buf = String::new();
                if let Some(mut stderr) = child.stderr.take() {
                    use tokio::io::AsyncReadExt;
                    let _ = stderr.read_to_string(&mut stderr_buf).await;
                }
                stderr_buf
            };

            let error = if let Ok(status) = status {
                if !status.success() && total_count == 0 {
                    if stderr_output.contains("not found")
                        || stderr_output.contains("No such file")
                        || status.code() == Some(127)
                    {
                        let cmd_name = match search_type {
                            GlobalSearchType::FileName => &fd_cmd,
                            GlobalSearchType::Content => &rg_cmd,
                        };
                        Some(format!("{cmd_name} not found"))
                    } else if stderr_output.trim().is_empty() {
                        None
                    } else {
                        Some(stderr_output.trim().to_string())
                    }
                } else {
                    None
                }
            } else if parse_failed && total_count == 0 {
                Some("Failed to parse ripgrep JSON output".to_string())
            } else {
                None
            };

            // Send final batch
            let _ = tx
                .send(SearchBatch {
                    generation,
                    results: batch,
                    is_final: true,
                    error,
                })
                .await;
        });

        Self {
            generation,
            handle,
            cancelled,
        }
    }

    /// Cancel this search job. Sets the cancellation flag and aborts the task.
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::Relaxed);
        self.handle.abort();
    }
}

impl Drop for SearchJob {
    fn drop(&mut self) {
        self.cancel();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn cancel_during_debounce_produces_no_results() {
        let (tx, mut rx) = mpsc::channel::<SearchBatch>(16);

        let job = SearchJob::spawn(
            1,
            "test_query".to_string(),
            GlobalSearchType::FileName,
            PathBuf::from("/tmp"),
            "fd".to_string(),
            "rg".to_string(),
            100,
            tx,
            5000, // long debounce
        );

        // Cancel immediately
        job.cancel();

        // Give a moment for the task to notice cancellation
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        // No results should come through
        assert!(rx.try_recv().is_err());
    }

    #[tokio::test]
    async fn spawn_with_nonexistent_command_returns_error() {
        let (tx, mut rx) = mpsc::channel::<SearchBatch>(16);

        let _job = SearchJob::spawn(
            42,
            "test_query".to_string(),
            GlobalSearchType::FileName,
            PathBuf::from("/tmp"),
            "this_command_definitely_does_not_exist_xyz123".to_string(),
            "rg".to_string(),
            100,
            tx,
            0, // no debounce
        );

        // Wait for the final batch
        let batch = tokio::time::timeout(std::time::Duration::from_secs(5), rx.recv())
            .await
            .expect("timeout waiting for batch")
            .expect("channel closed");

        assert_eq!(batch.generation, 42);
        assert!(batch.is_final);
        assert!(batch.error.is_some());
        let err = batch.error.unwrap();
        assert!(
            err.contains("this_command_definitely_does_not_exist_xyz123"),
            "error should mention the command: {err}"
        );
        assert!(batch.results.is_empty());
    }

    #[tokio::test]
    async fn cancel_sets_flag_and_aborts() {
        let (tx, _rx) = mpsc::channel::<SearchBatch>(16);

        let job = SearchJob::spawn(
            1,
            "test".to_string(),
            GlobalSearchType::FileName,
            PathBuf::from("/tmp"),
            "fd".to_string(),
            "rg".to_string(),
            100,
            tx,
            10_000, // very long debounce so task is still sleeping
        );

        job.cancel();

        // After cancel, the handle should be aborted
        // Give it a moment for the abort to take effect
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        assert!(job.handle.is_finished());
    }

    #[tokio::test]
    async fn drop_cancels_job() {
        let (tx, _rx) = mpsc::channel::<SearchBatch>(16);

        let job = SearchJob::spawn(
            1,
            "test".to_string(),
            GlobalSearchType::FileName,
            PathBuf::from("/tmp"),
            "fd".to_string(),
            "rg".to_string(),
            100,
            tx,
            10_000,
        );

        // We need a way to check after drop. Let's use the cancelled flag.
        let flag = job.cancelled.clone();

        drop(job);

        assert!(flag.load(Ordering::Relaxed));
        // After drop, the cancel flag should be set
        // The handle is also aborted, but we can't easily check it after drop
        // since it was moved. The flag check is sufficient.
    }
}
