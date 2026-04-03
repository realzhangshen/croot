mod common;

use std::collections::HashSet;
use std::path::PathBuf;

use croot::config::TreeConfig;
use croot::search::{GlobalSearchType, SearchBatch, SearchJob};
use croot::tree::forest::FileTree;
use tokio::sync::mpsc;

// ── Search cancellation ───────────────────────────────────────────────

#[tokio::test]
async fn search_job_cancel_during_debounce_produces_no_results() {
    let (tx, mut rx) = mpsc::channel::<SearchBatch>(16);
    let job = SearchJob::spawn(
        1,
        "test".to_string(),
        GlobalSearchType::FileName,
        std::env::temp_dir(),
        "fd".to_string(),
        "rg".to_string(),
        100,
        tx,
        5000, // Long debounce ensures cancel fires first
    );
    job.cancel();
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;

    match rx.try_recv() {
        Ok(batch) => assert!(batch.results.is_empty()),
        Err(_) => {} // Expected -- cancelled before any send
    }
}

// ── Search generation tracking ────────────────────────────────────────

#[tokio::test]
async fn search_job_reports_correct_generation() {
    let (tx, mut rx) = mpsc::channel::<SearchBatch>(16);
    let _job = SearchJob::spawn(
        42,
        "zzz_nonexistent_croot_test".to_string(),
        GlobalSearchType::FileName,
        std::env::temp_dir(),
        "fd".to_string(),
        "rg".to_string(),
        10,
        tx,
        10, // Short debounce
    );

    let timeout = tokio::time::timeout(std::time::Duration::from_secs(5), rx.recv()).await;

    if let Ok(Some(batch)) = timeout {
        assert_eq!(batch.generation, 42);
        assert!(batch.is_final);
    }
}

// ── Snapshot refresh preserves expanded dirs ──────────────────────────

#[test]
fn snapshot_refresh_preserves_expanded_dirs() {
    let tmp = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(tmp.path().join("a/b")).unwrap();
    std::fs::write(tmp.path().join("a/b/c.txt"), "c").unwrap();
    std::fs::write(tmp.path().join("d.txt"), "d").unwrap();

    let config = TreeConfig::default();
    let mut tree = FileTree::new(tmp.path().to_path_buf(), config.clone());
    let a_idx = tree.nodes.iter().position(|n| n.name == "a").unwrap();
    tree.expand(a_idx);

    let expanded: HashSet<PathBuf> = tree
        .nodes
        .iter()
        .filter(|n| n.is_dir() && n.is_expanded)
        .map(|n| n.path.clone())
        .collect();

    let refreshed = FileTree::snapshot_refresh(tmp.path().to_path_buf(), config, &expanded);

    let a = refreshed.nodes.iter().find(|n| n.name == "a").unwrap();
    assert!(a.is_expanded);
    assert!(refreshed.nodes.iter().any(|n| n.name == "b"));
}

// ── Cache consistency under rapid mutations ───────────────────────────

#[test]
fn cache_stays_consistent_through_rapid_mutations() {
    let tmp = tempfile::tempdir().unwrap();
    for i in 0..10 {
        let dir = tmp.path().join(format!("dir_{i:02}"));
        std::fs::create_dir(&dir).unwrap();
        std::fs::write(dir.join("file.txt"), "x").unwrap();
    }

    let mut tree = FileTree::new(tmp.path().to_path_buf(), TreeConfig::default());

    // Rapid expand/collapse cycles
    for _ in 0..20 {
        // Expand first unexpanded dir
        for i in 0..tree.nodes.len() {
            if tree.nodes[i].is_dir() && !tree.nodes[i].is_expanded {
                tree.expand(i);
                let _ = tree.cached_displayable_indices();
                break;
            }
        }
        // Collapse last expanded dir
        for i in (0..tree.nodes.len()).rev() {
            if tree.nodes[i].is_dir() && tree.nodes[i].is_expanded {
                tree.collapse(i);
                let _ = tree.cached_displayable_indices();
                break;
            }
        }
    }

    // Verify cache matches fresh computation
    let cached = tree.cached_displayable_indices().to_vec();
    let mut fresh = Vec::new();
    let mut i = 0;
    while i < tree.nodes.len() {
        fresh.push(i);
        let chain = tree.compact_chain_len(i);
        i += chain + 1;
    }
    assert_eq!(cached, fresh);
}

// ── Viewport indices consistency ──────────────────────────────────────

#[test]
fn viewport_indices_consistent_with_displayable_cache() {
    let tmp = tempfile::tempdir().unwrap();
    for i in 0..30 {
        std::fs::write(tmp.path().join(format!("file_{i:02}.txt")), "x").unwrap();
    }
    let mut tree = FileTree::new(tmp.path().to_path_buf(), TreeConfig::default());
    tree.scroll_offset = 10;

    let viewport = tree.viewport_indices(5);
    let all = tree.cached_displayable_indices().to_vec();

    assert_eq!(viewport, &all[10..15]);
}
