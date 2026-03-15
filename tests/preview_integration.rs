mod common;

use std::fs;

use croot::preview::loader::load_preview;
use croot::preview::state::{PreviewKind, PreviewState, StyledSpan};
use ratatui::style::Style;
use tempfile::tempdir;

use common::{create_file, init_colors};

fn line_text(line: &[StyledSpan]) -> String {
    line.iter().map(|(text, _)| text.as_str()).collect()
}

#[test]
fn markdown_rendering_respects_preview_mode() {
    init_colors();

    let dir = tempdir().expect("create temp dir");
    let markdown = create_file(dir.path(), "README.md", "# Title\n\nHello preview\n");

    let rendered = load_preview(&markdown, 1024, false, true, 80, false);
    assert_eq!(rendered.kind, PreviewKind::Rendered);
    assert!(rendered
        .content
        .iter()
        .any(|line| line_text(line).contains("Title")));

    let raw = load_preview(&markdown, 1024, false, false, 80, false);
    assert_eq!(raw.kind, PreviewKind::Text);
    assert_eq!(line_text(&raw.content[0]), "# Title");
}

#[test]
fn preview_loader_handles_text_and_empty_files() {
    init_colors();

    let dir = tempdir().expect("create temp dir");
    let text_path = create_file(dir.path(), "notes.txt", "line one\nline two\nline three\n");
    let empty_path = create_file(dir.path(), "empty.txt", "");

    let text = load_preview(&text_path, 1024, false, true, 80, false);
    let empty = load_preview(&empty_path, 1024, false, true, 80, false);

    assert_eq!(text.kind, PreviewKind::Text);
    assert!(!text.content.is_empty());
    assert_eq!(empty.kind, PreviewKind::Text);
}

#[test]
fn preview_loader_handles_binary_and_directory_inputs() {
    init_colors();

    let dir = tempdir().expect("create temp dir");
    let binary_path = dir.path().join("data.bin");
    fs::write(&binary_path, [0_u8, 159, 146, 150]).expect("write binary test file");
    fs::create_dir(dir.path().join("docs")).expect("create docs dir");
    create_file(dir.path(), "docs/guide.md", "# guide\n");

    let binary = load_preview(&binary_path, 1024, false, true, 80, false);
    let directory = load_preview(&dir.path().join("docs"), 1024, false, true, 80, false);

    assert_eq!(binary.kind, PreviewKind::Binary);
    assert!(line_text(&binary.content[0]).starts_with("00000000"));
    assert_eq!(directory.kind, PreviewKind::Directory);
    assert!(directory
        .content
        .iter()
        .any(|line| line_text(line).contains("guide.md")));
}

#[test]
fn preview_loader_honors_max_file_size_limit() {
    init_colors();

    let dir = tempdir().expect("create temp dir");
    let large = create_file(dir.path(), "large.txt", &"a".repeat(2_048));

    let preview = load_preview(&large, 1, false, true, 80, false);

    assert_eq!(preview.kind, PreviewKind::TooLarge);
}

#[test]
fn preview_state_scroll_stays_in_bounds_after_apply() {
    init_colors();

    let dir = tempdir().expect("create temp dir");
    let path = create_file(dir.path(), "scroll.txt", "a\nb\nc\n");
    let loaded = load_preview(&path, 1024, false, true, 80, false);

    let mut state = PreviewState::new();
    state.apply(
        path.clone(),
        loaded.kind.clone(),
        loaded.content,
        loaded.file_info,
    );

    state.scroll_down(10);
    assert_eq!(state.scroll_offset, state.total_lines.saturating_sub(1));

    state.scroll_up(10);
    assert_eq!(state.scroll_offset, 0);

    let mut manual = PreviewState::new();
    manual.apply(
        path,
        PreviewKind::Text,
        vec![
            vec![("alpha".to_string(), Style::default())],
            vec![("beta".to_string(), Style::default())],
            vec![("gamma".to_string(), Style::default())],
        ],
        "3 lines".to_string(),
    );
    manual.scroll_down(99);
    assert_eq!(manual.scroll_offset, 2);
    manual.scroll_up(99);
    assert_eq!(manual.scroll_offset, 0);
}
