mod common;

use croot::config::Config;
use croot::git::diff::GitDiffHint;
use croot::preview::loader::{load_preview, PreviewRequest};
use croot::preview::state::PreviewKind;
use croot::tree::forest::FileTree;
use tempfile::tempdir;

use common::{create_file, init_colors};

#[test]
fn parse_with_warning_handles_valid_and_invalid_toml() {
    let mut warning = None;
    let config = Config::parse_with_warning(
        r#"
[tree]
show_hidden = false
dirs_first = false
exclude = ["skip-me"]

[preview]
max_file_size_kb = 1
render_markdown = false
image_preview = false

[editor]
command = "nano"
"#,
        &mut warning,
    );

    assert!(warning.is_none());
    assert!(!config.tree.show_hidden);
    assert!(!config.tree.dirs_first);
    assert_eq!(config.tree.exclude, vec!["skip-me".to_string()]);
    assert_eq!(config.preview.max_file_size_kb, 1);
    assert!(!config.preview.render_markdown);
    assert!(!config.preview.image_preview);
    assert_eq!(config.editor.command.as_deref(), Some("nano"));

    let mut invalid_warning = Some(String::new());
    let fallback = Config::parse_with_warning("not = [valid", &mut invalid_warning);
    assert_eq!(
        fallback.tree.show_hidden,
        Config::default().tree.show_hidden
    );
    assert!(invalid_warning
        .expect("invalid config should produce warning")
        .contains("config parse error"));
}

#[test]
fn parsed_tree_config_changes_file_tree_behavior() {
    let dir = tempdir().expect("create temp dir");
    std::fs::create_dir(dir.path().join("adir")).expect("create visible dir");
    std::fs::create_dir(dir.path().join("skip-me")).expect("create excluded dir");
    create_file(dir.path(), ".hidden", "secret\n");
    create_file(dir.path(), "visible.txt", "shown\n");

    let mut warning = None;
    let config = Config::parse_with_warning(
        r#"
[tree]
show_hidden = false
dirs_first = true
exclude = ["skip-me"]
"#,
        &mut warning,
    );

    let tree = FileTree::new(dir.path().to_path_buf(), config.tree.clone());
    let names: Vec<&str> = tree.nodes.iter().map(|node| node.name.as_str()).collect();

    assert_eq!(names.first().copied(), Some("adir"));
    assert!(names.contains(&"visible.txt"));
    assert!(!names.contains(&".hidden"));
    assert!(!names.contains(&"skip-me"));
}

#[test]
fn parsed_preview_config_changes_load_preview_behavior() {
    init_colors();

    let dir = tempdir().expect("create temp dir");
    let markdown = create_file(dir.path(), "README.md", "# Title\n");
    let large = create_file(dir.path(), "large.txt", &"x".repeat(2_048));

    let mut warning = None;
    let config = Config::parse_with_warning(
        r"
[preview]
max_file_size_kb = 1
render_markdown = false
image_preview = false
",
        &mut warning,
    );

    let req = PreviewRequest {
        max_file_size_kb: config.preview.max_file_size_kb,
        syntax_highlight: config.preview.syntax_highlight,
        render_markdown: config.preview.render_markdown,
        preview_width: 80,
        image_preview: config.preview.image_preview,
        repo_root: None,
        git_diff_hint: GitDiffHint::Skip,
    };
    let markdown_preview = load_preview(&markdown, &req);
    let large_preview = load_preview(&large, &req);

    assert_eq!(markdown_preview.kind, PreviewKind::Text);
    assert_eq!(large_preview.kind, PreviewKind::TooLarge);
}

#[test]
fn resolved_toml_round_trip_preserves_explicit_values() {
    let mut warning = None;
    let config = Config::parse_with_warning(
        r#"
[tree]
show_hidden = false
dirs_first = false

[preview]
max_file_size_kb = 64
render_markdown = false
image_preview = false

[editor]
command = "hx"
"#,
        &mut warning,
    );

    let serialized = config.to_toml_string().unwrap();
    let mut roundtrip_warning = None;
    let roundtrip = Config::parse_with_warning(&serialized, &mut roundtrip_warning);

    assert!(roundtrip_warning.is_none());
    assert_eq!(roundtrip.tree.show_hidden, config.tree.show_hidden);
    assert_eq!(roundtrip.tree.dirs_first, config.tree.dirs_first);
    assert_eq!(
        roundtrip.preview.max_file_size_kb,
        config.preview.max_file_size_kb
    );
    assert_eq!(
        roundtrip.preview.render_markdown,
        config.preview.render_markdown
    );
    assert_eq!(
        roundtrip.preview.image_preview,
        config.preview.image_preview
    );
    assert_eq!(roundtrip.editor.command, config.editor.command);
}
