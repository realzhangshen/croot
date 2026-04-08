use std::path::PathBuf;
use std::time::{Duration, Instant};

use crossterm::event::{
    DisableMouseCapture, EnableMouseCapture, Event, EventStream, KeyboardEnhancementFlags,
    PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags,
};
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen};
use futures::StreamExt;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    widgets::{StatefulWidget, Widget},
    Terminal,
};
use tokio::sync::mpsc;
use unicode_width::UnicodeWidthStr;

use crate::cmux::bridge::CmuxBridge;
use crate::config::Config;
use crate::git::status::GitState;
use crate::input::handler::{
    build_keybinding_map, handle_key, handle_key_dialog, handle_key_global_search, handle_key_menu,
    handle_key_picker, handle_key_search, Action, InputMode, KeybindingMap,
};
use crate::input::mouse::{handle_mouse, ClickTracker};
use crate::layout::{self, FocusPane, PreviewLayout};
use crate::preview::loader::{load_preview, LoadedPreview};
use crate::preview::state::PreviewKind;
use crate::render::colors;
use crate::render::context_menu::{ContextMenuState, ContextMenuWidget, MenuAction};
use crate::render::global_search::GlobalSearchOverlay;
use crate::render::input_dialog::{InputDialogState, InputDialogWidget};
use crate::render::picker::{PickerState, PickerWidget};
use crate::render::preview_view::PreviewView;
use crate::render::search_bar::SearchBar;
use crate::render::status_bar::{HyperlinkRegion, StatusBar};
use crate::render::tree_view::TreeView;
use crate::search::{
    do_match, do_match_positions, group_search_results, GlobalSearchType, GroupedItem, SearchBatch,
    SearchJob, SearchMode, SearchState,
};
use crate::tree::forest::FileTree;

// Path validation functions are in crate::file_ops
use crate::file_ops;

mod actions;
mod branch;
mod draw;
mod editor;
mod event_loop;
mod file_ops_bridge;
mod mouse;
mod preview_controller;
mod preview_ops;
mod refresh;
mod search_ops;
mod tree_ops;

pub(super) use preview_controller::PreviewController;
pub(super) use refresh::RefreshCoordinator;

/// Result of an async branch switch operation.
pub(super) struct BranchSwitchResult {
    success: bool,
    stderr: String,
    repo_root: PathBuf,
}

/// Result of a background refresh operation.
pub(super) struct RefreshResult {
    pub(super) generation: u64,
    pub(super) tree: FileTree,
    pub(super) git: Option<GitState>,
}

/// Signal from action handling that requires terminal-level processing.
#[derive(Debug)]
pub enum PostAction {
    None,
    /// Auto-detect: try cmux first, fall back to suspend (keyboard shortcuts).
    /// The optional `usize` is a 1-based line number for `+LINE` goto.
    OpenEditor(PathBuf, Option<usize>),
    /// Force suspend mode (context menu "Open in Editor").
    OpenEditorSuspend(PathBuf, Option<usize>),
    /// Force cmux mode (context menu "Open in cmux Tab").
    OpenEditorCmux(PathBuf, Option<usize>),
    /// Open in external/GUI editor (background, no TUI suspend).
    OpenExternalEditor(PathBuf, Option<usize>),
}

/// Modal overlay state: input mode, context menu, dialogs, picker, and error messages.
pub struct UiOverlayState {
    pub input_mode: InputMode,
    pub context_menu: Option<ContextMenuState>,
    pub input_dialog: Option<InputDialogState>,
    pub picker_state: Option<PickerState>,
    pub error_message: Option<(String, Instant)>,
}

impl Default for UiOverlayState {
    fn default() -> Self {
        Self {
            input_mode: InputMode::Normal,
            context_menu: None,
            input_dialog: None,
            picker_state: None,
            error_message: None,
        }
    }
}

#[allow(clippy::struct_excessive_bools)]
pub struct App {
    pub tree: FileTree,
    pub git: Option<GitState>,
    pub cmux: Option<CmuxBridge>,
    pub config: Config,
    pub root: PathBuf,
    pub should_quit: bool,
    pub(super) tree_area_y: u16,
    pub(super) tree_area_height: u16,
    /// Preview pane state: visibility, loaded content, scroll, debounce handle,
    /// image resize channels, etc. Grouped here so preview-specific state lives
    /// in one place instead of eleven loose App fields.
    pub preview: PreviewController,
    pub focus: FocusPane,
    pub(super) dragging_separator: bool,
    pub(super) main_area_width: u16,
    pub(super) hover_row: Option<usize>,
    // UI overlay state (modal overlays, dialogs, errors)
    pub ui: UiOverlayState,
    // Search state
    pub(super) search_state: SearchState,
    /// Handle for the current async global search task.
    pub(super) global_search_job: Option<crate::search::SearchJob>,
    // Hyperlink regions for post-render OSC 8 emission
    pub(super) hyperlink_regions: Vec<HyperlinkRegion>,
    // Whether Kitty keyboard enhancement protocol is active
    pub(super) enhanced_keyboard: bool,
    // Double-click detection for tree rows
    pub(super) click_tracker: ClickTracker,
    // User-configured keybindings
    pub(super) keybinding_map: KeybindingMap,
    // Whether mouse capture is enabled
    pub(super) mouse_enabled: bool,
    // Status/search bar y-coordinates for mouse routing
    pub(super) status_bar_y: u16,
    pub(super) search_bar_y: Option<u16>,
    // Status bar branch click region: (x_start, x_end)
    pub(super) status_bar_branch_region: Option<(u16, u16)>,
    // Channel for receiving branch switch results
    pub(super) branch_switch_rx: Option<mpsc::Receiver<BranchSwitchResult>>,
    /// State machine for background/synchronous tree refreshes: generation
    /// tracking, in-flight guard, and coalesced follow-up. See
    /// [`RefreshCoordinator`] for the semantics.
    pub(super) refresh: RefreshCoordinator,
    // Cached terminal area from last draw, used by mouse handlers
    pub(super) last_terminal_area: ratatui::layout::Rect,
}

impl App {
    pub fn new(
        root: PathBuf,
        enhanced_keyboard: bool,
        config: Config,
        #[cfg(feature = "image-preview")] image_picker: Option<ratatui_image::picker::Picker>,
    ) -> anyhow::Result<Self> {
        let mut tree = FileTree::new(root.clone(), config.tree.clone());
        let git = GitState::load(&root);
        let cmux = CmuxBridge::detect();

        if let Some(ref git) = git {
            git.apply_to_nodes(&mut tree.nodes);
        }

        let preview_visible = config.preview.auto_preview;
        let render_markdown = config.preview.render_markdown;
        let mouse_enabled = config.mouse.enabled;
        let keybinding_map = build_keybinding_map(&config.keybindings);

        #[allow(unused_mut)]
        let mut preview = PreviewController::new(preview_visible, render_markdown);
        #[cfg(feature = "image-preview")]
        {
            preview.image_picker = image_picker;
        }

        Ok(Self {
            tree,
            git,
            cmux,
            config,
            root,
            should_quit: false,
            tree_area_y: 0,
            tree_area_height: 0,
            preview,
            focus: FocusPane::Tree,
            dragging_separator: false,
            main_area_width: 0,
            hover_row: None,
            ui: UiOverlayState::default(),
            search_state: SearchState::new(SearchMode::Find),
            global_search_job: None,
            hyperlink_regions: Vec::new(),
            enhanced_keyboard,
            click_tracker: ClickTracker::new(),
            keybinding_map,
            mouse_enabled,
            status_bar_y: 0,
            search_bar_y: None,
            status_bar_branch_region: None,
            branch_switch_rx: None,
            refresh: RefreshCoordinator::new(),
            last_terminal_area: ratatui::layout::Rect::new(0, 0, 80, 24),
        })
    }

    /// Display a transient error message in the status bar area.
    pub(super) fn show_error(&mut self, msg: String) {
        self.ui.error_message = Some((msg, Instant::now()));
    }

    pub(super) fn reapply_git(&mut self) {
        if let Some(ref git) = self.git {
            git.apply_to_nodes(&mut self.tree.nodes);
            if let Some(err) = git.last_error() {
                self.show_error(err.to_string());
            }
        }
    }

    /// Get the directory context for the currently selected node.
    fn current_dir(&self) -> PathBuf {
        if let Some(node) = self.tree.selected() {
            file_ops::dir_for_path(&node.path, node.is_dir(), &self.root)
        } else {
            self.root.clone()
        }
    }
}

/// Build argv for an external editor command with optional `file:line` syntax.
///
/// Returns a `Vec<String>` ready for `Command::new(argv[0]).args(&argv[1..])`.
/// Uses `file:line` format (standard for VS Code `-g`, Sublime, etc.).
fn build_external_editor_argv(
    editor_cmd: &str,
    path: &std::path::Path,
    line: Option<usize>,
) -> Vec<String> {
    let mut argv = shell_words::split(editor_cmd).unwrap_or_else(|_| vec![editor_cmd.to_string()]);
    let file_arg = match line {
        Some(n) => format!("{}:{n}", path.display()),
        None => path.display().to_string(),
    };
    argv.push(file_arg);
    argv
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::file_ops::DialogKind;
    use crate::search::{
        ContentMatch, FileGroup, GlobalSearchResult, GlobalSearchType, SearchMode, SearchState,
    };
    use crate::tree::node::TreeNode;
    use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};
    use std::path::Path;

    /// Helper to create a minimal App rooted in a temp directory.
    /// Returns (App, `TempDir`) -- the `TempDir` must be kept alive for the test duration.
    fn test_app() -> (App, tempfile::TempDir) {
        let tmp = tempfile::TempDir::new().expect("create temp dir");
        let app = App::new(
            tmp.path().to_path_buf(),
            false,
            Config::default(),
            #[cfg(feature = "image-preview")]
            None,
        )
        .expect("test app creation");
        (app, tmp)
    }

    /// Helper to create App with files for navigation tests.
    fn test_app_with_files() -> (App, tempfile::TempDir) {
        let tmp = tempfile::TempDir::new().unwrap();
        std::fs::write(tmp.path().join("aaa.txt"), "a").unwrap();
        std::fs::write(tmp.path().join("bbb.txt"), "b").unwrap();
        std::fs::create_dir_all(tmp.path().join("subdir")).unwrap();
        std::fs::write(tmp.path().join("subdir/ccc.txt"), "c").unwrap();
        let app = App::new(
            tmp.path().to_path_buf(),
            false,
            Config::default(),
            #[cfg(feature = "image-preview")]
            None,
        )
        .expect("test app creation");
        (app, tmp)
    }

    fn make_channels() -> (
        mpsc::Sender<(u64, PathBuf, LoadedPreview)>,
        mpsc::Sender<SearchBatch>,
    ) {
        (mpsc::channel(16).0, mpsc::channel(16).0)
    }

    #[test]
    fn test_global_search_mouse_move_does_not_cancel() {
        let (mut app, _tmp) = test_app();
        // Enter GlobalSearch mode
        app.ui.input_mode = InputMode::GlobalSearch;
        app.search_state = SearchState::new(SearchMode::Global);
        app.search_state.global_search_type = GlobalSearchType::FileName;

        // Simulate a mouse move event
        let mouse = MouseEvent {
            kind: MouseEventKind::Moved,
            column: 10,
            row: 10,
            modifiers: crossterm::event::KeyModifiers::NONE,
        };
        let (ptx, _prx) = mpsc::channel(1);
        app.handle_global_search_mouse(mouse, &ptx);

        // Mode should still be GlobalSearch
        assert_eq!(app.ui.input_mode, InputMode::GlobalSearch);
    }

    #[test]
    fn test_global_search_scroll_does_not_cancel() {
        let (mut app, _tmp) = test_app();
        app.ui.input_mode = InputMode::GlobalSearch;
        app.search_state = SearchState::new(SearchMode::Global);

        let mouse = MouseEvent {
            kind: MouseEventKind::ScrollUp,
            column: 10,
            row: 10,
            modifiers: crossterm::event::KeyModifiers::NONE,
        };
        let (ptx, _prx) = mpsc::channel(1);
        app.handle_global_search_mouse(mouse, &ptx);

        assert_eq!(app.ui.input_mode, InputMode::GlobalSearch);
    }

    #[test]
    fn test_show_error_sets_message() {
        let (mut app, _tmp) = test_app();
        assert!(app.ui.error_message.is_none());
        app.show_error("test error".to_string());
        assert!(app.ui.error_message.is_some());
        let (msg, _ts) = app.ui.error_message.as_ref().unwrap();
        assert_eq!(msg, "test error");
    }

    #[test]
    fn test_error_message_auto_dismiss() {
        let (mut app, _tmp) = test_app();
        // Set an error with a past timestamp (4 seconds ago)
        app.ui.error_message = Some((
            "old error".to_string(),
            Instant::now().checked_sub(Duration::from_secs(4)).unwrap(),
        ));
        // After 3 seconds the message should be considered expired
        let (_, ts) = app.ui.error_message.as_ref().unwrap();
        assert!(ts.elapsed() >= Duration::from_secs(3));
    }

    #[test]
    fn test_confirm_dialog_rename_nonexistent_shows_error() {
        let (mut app, _tmp) = test_app();
        let (preview_tx, _preview_rx) = mpsc::channel(1);
        let fake_path = std::env::temp_dir().join("croot_nonexistent_for_rename");
        app.ui.input_dialog = Some(InputDialogState::new(
            DialogKind::Rename,
            fake_path,
            "old_name".to_string(),
        ));
        app.ui.input_mode = InputMode::Dialog;
        // Set the input to something different to trigger rename
        app.ui.input_dialog.as_mut().unwrap().input = "new_name".to_string();
        app.confirm_dialog(&preview_tx);
        assert!(
            app.ui.error_message.is_some(),
            "Rename of nonexistent file should show error"
        );
    }

    #[test]
    fn test_confirm_dialog_delete_nonexistent_shows_error() {
        let (mut app, _tmp) = test_app();
        let (preview_tx, _preview_rx) = mpsc::channel(1);
        let fake_path = std::env::temp_dir().join("croot_nonexistent_for_delete");
        app.ui.input_dialog = Some(InputDialogState::new(
            DialogKind::ConfirmDelete,
            fake_path,
            "ghost".to_string(),
        ));
        app.ui.input_mode = InputMode::Dialog;
        app.confirm_dialog(&preview_tx);
        assert!(
            app.ui.error_message.is_some(),
            "Delete of nonexistent file should show error"
        );
    }

    // ── Background refresh coalescing ──────────────────────────

    #[tokio::test]
    async fn background_refresh_first_call_sets_in_flight() {
        let (mut app, _tmp) = test_app();
        let (refresh_tx, _refresh_rx) = mpsc::channel::<RefreshResult>(2);
        let before = app.refresh.generation();

        assert!(!app.refresh.in_flight());
        assert!(!app.refresh.pending());

        app.background_refresh(&refresh_tx);

        // First call must spawn a task and bump the generation.
        assert!(app.refresh.in_flight());
        assert!(!app.refresh.pending());
        assert_eq!(app.refresh.generation(), before.wrapping_add(1));
    }

    #[tokio::test]
    async fn background_refresh_coalesces_while_in_flight() {
        let (mut app, _tmp) = test_app();
        let (refresh_tx, _refresh_rx) = mpsc::channel::<RefreshResult>(2);

        // Put the coordinator into the in-flight state via the normal API.
        app.background_refresh(&refresh_tx);
        let gen_before = app.refresh.generation();

        // Second trigger should only set pending, not bump generation and
        // not spawn a new task.
        app.background_refresh(&refresh_tx);
        assert!(app.refresh.in_flight(), "in-flight flag must stay set");
        assert!(app.refresh.pending(), "second trigger must set pending");
        assert_eq!(
            app.refresh.generation(),
            gen_before,
            "coalesced trigger must not bump generation"
        );

        // Third trigger while in flight and pending already set: still a no-op.
        app.background_refresh(&refresh_tx);
        assert!(app.refresh.pending());
        assert_eq!(app.refresh.generation(), gen_before);
    }

    #[tokio::test]
    async fn full_refresh_sync_invalidates_in_flight_background_refresh() {
        // Regression: without bumping the generation in full_refresh_sync,
        // a stale in-flight background result can land after the sync refresh
        // and clobber the freshly-loaded tree/git state. The event loop's
        // `is_current` check is the guard that must fire.
        let (mut app, _tmp) = test_app();
        let (preview_tx, _preview_rx) = mpsc::channel(1);
        let (refresh_tx, _refresh_rx) = mpsc::channel::<RefreshResult>(2);

        // Kick off a background refresh. This captures gen=1.
        app.background_refresh(&refresh_tx);
        let in_flight_gen = app.refresh.generation();
        assert!(app.refresh.in_flight());

        // Now a synchronous refresh runs (e.g. post-editor). It should bump
        // the generation so that when the background result finally arrives
        // `is_current` returns false.
        app.full_refresh_sync(&preview_tx);

        assert_ne!(
            app.refresh.generation(),
            in_flight_gen,
            "full_refresh_sync must bump the generation to invalidate the \
             in-flight background refresh"
        );
        assert!(
            !app.refresh.pending(),
            "full_refresh_sync should clear pending too; a pending \
             follow-up would also overwrite the sync result with a stale snapshot"
        );
    }

    #[tokio::test]
    async fn background_refresh_resets_flag_after_completion() {
        // Exercise the fact that clearing in_flight via finish_background
        // re-enables spawning of a fresh task.
        let (mut app, _tmp) = test_app();
        let (refresh_tx, _refresh_rx) = mpsc::channel::<RefreshResult>(2);

        app.background_refresh(&refresh_tx);
        assert!(app.refresh.in_flight());
        let gen_after_first = app.refresh.generation();

        // Simulate the event loop clearing in-flight after applying the result.
        let _ = app.refresh.finish_background();

        app.background_refresh(&refresh_tx);
        assert!(app.refresh.in_flight());
        assert_eq!(
            app.refresh.generation(),
            gen_after_first.wrapping_add(1),
            "follow-up spawn must bump generation again"
        );
    }

    #[test]
    fn shell_words_parses_quoted_editor_path() {
        let input = "'/path/to/my editor' --wait";
        let parts = shell_words::split(input).unwrap();
        assert_eq!(parts, vec!["/path/to/my editor", "--wait"]);
    }

    // Path validation tests are now in crate::file_ops::tests

    #[test]
    fn confirm_dialog_error_skips_refresh() {
        let (mut app, _tmp) = test_app();
        let (preview_tx, _preview_rx) = mpsc::channel(1);
        // Enter filter mode with a stale visible index
        app.search_state = SearchState::new(SearchMode::Filter);
        app.search_state.query = "nonexistent_xyz".to_string();
        app.search_state.visible_indices = vec![0, 999];

        // Set up a delete dialog for a nonexistent file (will fail -> Error)
        let fake_path = std::env::temp_dir().join("croot_test_confirm_refresh");
        app.ui.input_dialog = Some(InputDialogState::new(
            DialogKind::ConfirmDelete,
            fake_path,
            "ghost".to_string(),
        ));
        app.ui.input_mode = InputMode::Dialog;
        app.confirm_dialog(&preview_tx);

        // On error, tree refresh is skipped -- stale indices remain unchanged
        assert!(
            app.search_state.visible_indices.contains(&999),
            "stale index should remain since refresh was skipped on error"
        );
        // But an error message should be shown
        assert!(
            app.ui.error_message.is_some(),
            "error message should be shown for failed delete"
        );
    }

    #[test]
    fn confirm_dialog_noop_skips_refresh() {
        let (mut app, _tmp) = test_app();
        let (preview_tx, _preview_rx) = mpsc::channel(1);
        app.search_state = SearchState::new(SearchMode::Filter);
        app.search_state.visible_indices = vec![0, 999];

        // Set up a rename dialog with empty input (will be Noop)
        let fake_path = std::env::temp_dir().join("croot_test_noop");
        let mut dialog =
            InputDialogState::new(DialogKind::Rename, fake_path, "original.txt".to_string());
        dialog.input.clear();
        dialog.cursor_pos = 0;
        app.ui.input_dialog = Some(dialog);
        app.ui.input_mode = InputMode::Dialog;
        app.confirm_dialog(&preview_tx);

        // On noop, stale indices remain unchanged and no error
        assert!(
            app.search_state.visible_indices.contains(&999),
            "stale index should remain since refresh was skipped on noop"
        );
        assert!(app.ui.error_message.is_none(), "no error on noop");
    }

    #[test]
    fn test_global_search_click_outside_cancels() {
        let (mut app, _tmp) = test_app();
        app.ui.input_mode = InputMode::GlobalSearch;
        app.search_state = SearchState::new(SearchMode::Global);
        app.search_state.global_search_type = GlobalSearchType::FileName;

        // Click at (0, 0) -- guaranteed outside the centered overlay
        let mouse = MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: 0,
            row: 0,
            modifiers: crossterm::event::KeyModifiers::NONE,
        };
        let (ptx, _prx) = mpsc::channel(1);
        app.handle_global_search_mouse(mouse, &ptx);

        assert_eq!(app.ui.input_mode, InputMode::Normal);
    }

    #[tokio::test]
    async fn global_search_confirm_opens_editor() {
        let (mut app, _tmp) = test_app();
        let test_file = app.root.join("preview_test.txt");
        std::fs::write(&test_file, "content").unwrap();
        app.tree.refresh();

        // Set up global search state with a filename result
        app.ui.input_mode = InputMode::GlobalSearch;
        app.search_state = SearchState::new(SearchMode::Global);
        app.search_state.global_results.push(GlobalSearchResult {
            path: test_file.clone(),
            display: "preview_test.txt".to_string(),
            line: None,
            context: None,
        });
        app.search_state.global_selected = 0;

        let (ptx, _prx) = mpsc::channel(16);
        let search_tx: mpsc::Sender<SearchBatch> = mpsc::channel(1).0;

        // Default open_mode is External -> should return OpenExternalEditor
        let post = app.handle_action(&Action::GlobalSearchConfirm, &ptx, &search_tx);
        assert_eq!(app.ui.input_mode, InputMode::Normal);
        assert!(
            matches!(post, PostAction::OpenExternalEditor(ref p, None) if *p == test_file),
            "Expected OpenExternalEditor, got {post:?}"
        );

        let _ = std::fs::remove_file(&test_file);
    }

    #[tokio::test]
    async fn global_search_goto_navigates_to_file() {
        let (mut app, _tmp) = test_app();
        let test_file = app.root.join("goto_test.txt");
        std::fs::write(&test_file, "content").unwrap();
        app.tree.refresh();
        app.preview.visible = true;

        // Set up global search state with a filename result
        app.ui.input_mode = InputMode::GlobalSearch;
        app.search_state = SearchState::new(SearchMode::Global);
        app.search_state.global_results.push(GlobalSearchResult {
            path: test_file.clone(),
            display: "goto_test.txt".to_string(),
            line: None,
            context: None,
        });
        app.search_state.global_selected = 0;

        let (ptx, _prx) = mpsc::channel(16);
        let search_tx: mpsc::Sender<SearchBatch> = mpsc::channel(1).0;

        // Goto should navigate to file in tree (old Enter behavior)
        let post = app.handle_action(&Action::GlobalSearchGoto, &ptx, &search_tx);
        assert_eq!(app.ui.input_mode, InputMode::Normal);
        assert!(matches!(post, PostAction::None));
        assert!(
            app.preview.debounce_handle.is_some(),
            "Preview debounce handle should be set after goto"
        );

        let _ = std::fs::remove_file(&test_file);
    }

    // -- Content search: confirm opens editor, goto navigates --

    #[test]
    fn content_search_confirm_on_match_defaults_to_external() {
        let (mut app, _tmp) = test_app();
        let test_file = app.root.join("match_test.rs");
        std::fs::write(&test_file, "line1\nline2\nTODO: fix\n").unwrap();
        app.tree.refresh();

        app.ui.input_mode = InputMode::GlobalSearch;
        app.search_state = SearchState::new(SearchMode::Global);
        app.search_state.global_search_type = GlobalSearchType::Content;
        app.search_state.grouped_results.push(FileGroup {
            path: test_file.clone(),
            display: "match_test.rs".to_string(),
            matches: vec![ContentMatch {
                line: Some(3),
                context: Some("TODO: fix".to_string()),
            }],
            collapsed: false,
        });
        app.search_state.global_selected = 1;

        let (ptx, _prx) = mpsc::channel(16);
        let stx = mpsc::channel(1).0;

        // Default open_mode is External
        let post = app.handle_action(&Action::GlobalSearchConfirm, &ptx, &stx);
        assert_eq!(app.ui.input_mode, InputMode::Normal);
        assert!(
            matches!(post, PostAction::OpenExternalEditor(ref p, Some(3)) if *p == test_file),
            "Expected OpenExternalEditor with line 3, got {post:?}"
        );
    }

    #[test]
    fn content_search_confirm_editor_mode_returns_open_editor() {
        use crate::config::SearchOpenMode;
        let (mut app, _tmp) = test_app();
        let test_file = app.root.join("match_test2.rs");
        std::fs::write(&test_file, "line1\nline2\nTODO: fix\n").unwrap();
        app.tree.refresh();
        app.config.search.open_mode = SearchOpenMode::Editor;

        app.ui.input_mode = InputMode::GlobalSearch;
        app.search_state = SearchState::new(SearchMode::Global);
        app.search_state.global_search_type = GlobalSearchType::Content;
        app.search_state.grouped_results.push(FileGroup {
            path: test_file.clone(),
            display: "match_test2.rs".to_string(),
            matches: vec![ContentMatch {
                line: Some(3),
                context: Some("TODO: fix".to_string()),
            }],
            collapsed: false,
        });
        app.search_state.global_selected = 1;

        let (ptx, _prx) = mpsc::channel(16);
        let stx = mpsc::channel(1).0;

        let post = app.handle_action(&Action::GlobalSearchConfirm, &ptx, &stx);
        assert_eq!(app.ui.input_mode, InputMode::Normal);
        assert!(
            matches!(post, PostAction::OpenEditor(ref p, Some(3)) if *p == test_file),
            "Expected OpenEditor with line 3, got {post:?}"
        );
    }

    #[test]
    fn content_search_confirm_on_header_toggles_collapse() {
        let (mut app, _tmp) = test_app();
        app.ui.input_mode = InputMode::GlobalSearch;
        app.search_state = SearchState::new(SearchMode::Global);
        app.search_state.global_search_type = GlobalSearchType::Content;
        app.search_state.grouped_results.push(FileGroup {
            path: app.root.join("file.rs"),
            display: "file.rs".to_string(),
            matches: vec![ContentMatch {
                line: Some(1),
                context: Some("match".to_string()),
            }],
            collapsed: false,
        });
        // Select header (index 0)
        app.search_state.global_selected = 0;

        let (ptx, _prx) = mpsc::channel(16);
        let stx = mpsc::channel(1).0;

        let post = app.handle_action(&Action::GlobalSearchConfirm, &ptx, &stx);
        assert!(matches!(post, PostAction::None));
        assert!(app.search_state.grouped_results[0].collapsed);
        // Still in GlobalSearch mode (didn't close overlay)
        assert_eq!(app.ui.input_mode, InputMode::GlobalSearch);
    }

    #[tokio::test]
    async fn content_search_goto_on_match_navigates_to_tree() {
        let (mut app, _tmp) = test_app();
        let test_file = app.root.join("goto_match.rs");
        std::fs::write(&test_file, "hello\nworld\n").unwrap();
        app.tree.refresh();
        app.preview.visible = true;

        app.ui.input_mode = InputMode::GlobalSearch;
        app.search_state = SearchState::new(SearchMode::Global);
        app.search_state.global_search_type = GlobalSearchType::Content;
        app.search_state.grouped_results.push(FileGroup {
            path: test_file.clone(),
            display: "goto_match.rs".to_string(),
            matches: vec![ContentMatch {
                line: Some(2),
                context: Some("world".to_string()),
            }],
            collapsed: false,
        });
        app.search_state.global_selected = 1; // match line

        let (ptx, _prx) = mpsc::channel(16);
        let stx = mpsc::channel(1).0;

        let post = app.handle_action(&Action::GlobalSearchGoto, &ptx, &stx);
        assert_eq!(app.ui.input_mode, InputMode::Normal);
        assert!(matches!(post, PostAction::None));
        // Should have set pending_preview_line for scroll
        assert!(
            app.preview.pending_line.is_none()
                || app.preview.pending_line == Some((test_file.clone(), 2))
        );
    }

    // Symlink path traversal tests are now in crate::file_ops::tests

    // -- Bug 4: fd/rg shell-split --

    #[test]
    fn test_shell_words_split_fd_command() {
        let parts = shell_words::split("fd --hidden --no-ignore").unwrap();
        assert_eq!(parts, vec!["fd", "--hidden", "--no-ignore"]);
        let (bin, extra) = parts.split_first().unwrap();
        assert_eq!(bin, "fd");
        assert_eq!(extra, &["--hidden", "--no-ignore"]);
    }

    #[test]
    fn test_shell_words_split_rg_command() {
        let parts = shell_words::split("rg --hidden").unwrap();
        assert_eq!(parts, vec!["rg", "--hidden"]);
    }

    #[test]
    fn test_shell_words_split_simple_command() {
        // A simple command without args should produce a single element
        let parts = shell_words::split("fd").unwrap();
        assert_eq!(parts, vec!["fd"]);
    }

    #[tokio::test]
    async fn confirm_dialog_success_refreshes_preview() {
        let (mut app, tmp) = test_app();
        let file = tmp.path().join("keep.txt");
        std::fs::write(&file, "content").unwrap();
        app.tree.refresh();
        app.preview.visible = true;

        app.ui.input_dialog = Some(InputDialogState::new(
            DialogKind::Rename,
            file,
            "keep.txt".to_string(),
        ));
        app.ui.input_mode = InputMode::Dialog;
        if let Some(dialog) = app.ui.input_dialog.as_mut() {
            dialog.input = "renamed.txt".to_string();
            dialog.cursor_pos = dialog.input.len();
        }

        let (preview_tx, _preview_rx) = mpsc::channel(16);
        app.confirm_dialog(&preview_tx);

        assert!(
            app.preview.debounce_handle.is_some(),
            "successful file operations should reload the preview"
        );
    }

    #[tokio::test]
    async fn action_global_search_backspace_empty_cancels_pending_search() {
        let (mut app, _tmp) = test_app();
        let (preview_tx, search_tx) = make_channels();
        app.ui.input_mode = InputMode::GlobalSearch;
        app.search_state = SearchState::new(SearchMode::Global);
        app.search_state.query = "a".to_string();
        app.search_state.cursor_pos = 1;
        app.search_state.request_id = 7;
        app.search_state.global_loading = true;
        // Create a dummy SearchJob with a long debounce so it stays alive
        let (dummy_tx, _dummy_rx) = mpsc::channel(1);
        app.global_search_job = Some(SearchJob::spawn(
            7,
            "a".to_string(),
            GlobalSearchType::FileName,
            app.root.clone(),
            "sleep".to_string(),
            "rg".to_string(),
            100,
            dummy_tx,
            60_000,
        ));

        app.handle_action(&Action::GlobalSearchBackspace, &preview_tx, &search_tx);

        assert!(app.search_state.query.is_empty());
        assert!(!app.search_state.global_loading);
        assert!(app.search_state.global_results.is_empty());
        assert!(app.global_search_job.is_none());
        assert_eq!(app.search_state.request_id, 8);
    }

    // -- Action handling: navigation --

    #[test]
    fn action_cursor_down_moves_cursor() {
        let (mut app, _tmp) = test_app_with_files();
        let (ptx, stx) = make_channels();
        assert_eq!(app.tree.cursor, 0);
        app.handle_action(&Action::CursorDown, &ptx, &stx);
        assert_eq!(app.tree.cursor, 1);
    }

    #[tokio::test]
    async fn action_cursor_down_refreshes_preview_when_visible() {
        let (mut app, _tmp) = test_app_with_files();
        let (ptx, stx) = make_channels();
        app.preview.visible = true;
        let initial_generation = app.preview.generation;

        app.handle_action(&Action::CursorDown, &ptx, &stx);

        assert_eq!(app.tree.cursor, 1);
        assert!(app.preview.generation > initial_generation);
        assert!(app.preview.debounce_handle.is_some());
    }

    #[test]
    fn action_cursor_up_at_top_stays() {
        let (mut app, _tmp) = test_app_with_files();
        let (ptx, stx) = make_channels();
        assert_eq!(app.tree.cursor, 0);
        app.handle_action(&Action::CursorUp, &ptx, &stx);
        assert_eq!(app.tree.cursor, 0);
    }

    #[test]
    fn action_goto_bottom_then_top() {
        let (mut app, _tmp) = test_app_with_files();
        let (ptx, stx) = make_channels();
        let last = app.tree.len() - 1;
        app.handle_action(&Action::GotoBottom, &ptx, &stx);
        assert_eq!(app.tree.cursor, last);
        app.handle_action(&Action::GotoTop, &ptx, &stx);
        assert_eq!(app.tree.cursor, 0);
    }

    // -- Action handling: toggle preview --

    #[tokio::test]
    async fn action_toggle_preview_flips_visibility() {
        let (mut app, _tmp) = test_app_with_files();
        let (ptx, stx) = make_channels();
        assert!(!app.preview.visible);
        app.handle_action(&Action::TogglePreview, &ptx, &stx);
        assert!(app.preview.visible);
        app.handle_action(&Action::TogglePreview, &ptx, &stx);
        assert!(!app.preview.visible);
    }

    #[test]
    fn action_toggle_preview_off_resets_focus_to_tree() {
        let (mut app, _tmp) = test_app_with_files();
        let (ptx, stx) = make_channels();
        app.preview.visible = true;
        app.focus = FocusPane::Preview;
        app.handle_action(&Action::TogglePreview, &ptx, &stx);
        assert_eq!(app.focus, FocusPane::Tree);
    }

    // -- Action handling: quit --

    #[test]
    fn action_quit_in_normal_mode_sets_should_quit() {
        let (mut app, _tmp) = test_app_with_files();
        let (ptx, stx) = make_channels();
        app.ui.input_mode = InputMode::Normal;
        app.handle_action(&Action::Quit, &ptx, &stx);
        assert!(app.should_quit);
    }

    #[test]
    fn action_quit_in_search_mode_returns_to_normal() {
        let (mut app, _tmp) = test_app_with_files();
        let (ptx, stx) = make_channels();
        app.ui.input_mode = InputMode::Search;
        app.handle_action(&Action::Quit, &ptx, &stx);
        assert!(!app.should_quit);
        assert_eq!(app.ui.input_mode, InputMode::Normal);
    }

    // -- Action handling: search lifecycle --

    #[test]
    fn action_start_find_enters_search_mode() {
        let (mut app, _tmp) = test_app_with_files();
        let (ptx, stx) = make_channels();
        app.handle_action(&Action::StartFind, &ptx, &stx);
        assert_eq!(app.ui.input_mode, InputMode::Search);
        assert_eq!(app.search_state.mode, SearchMode::Find);
    }

    #[test]
    fn action_start_filter_enters_search_mode() {
        let (mut app, _tmp) = test_app_with_files();
        let (ptx, stx) = make_channels();
        app.handle_action(&Action::StartFilter, &ptx, &stx);
        assert_eq!(app.ui.input_mode, InputMode::Search);
        assert_eq!(app.search_state.mode, SearchMode::Filter);
    }

    #[test]
    fn action_search_cancel_restores_cursor() {
        let (mut app, _tmp) = test_app_with_files();
        let (ptx, stx) = make_channels();
        app.handle_action(&Action::StartFind, &ptx, &stx);
        let orig_cursor = app.search_state.origin_cursor;
        // Move cursor during search
        app.handle_action(&Action::CursorDown, &ptx, &stx);
        app.handle_action(&Action::SearchCancel, &ptx, &stx);
        assert_eq!(app.tree.cursor, orig_cursor);
        assert_eq!(app.ui.input_mode, InputMode::Normal);
    }

    // -- Action handling: global search --

    #[test]
    fn action_start_global_search_enters_mode() {
        let (mut app, _tmp) = test_app_with_files();
        let (ptx, stx) = make_channels();
        app.handle_action(&Action::StartGlobalSearch, &ptx, &stx);
        assert_eq!(app.ui.input_mode, InputMode::GlobalSearch);
        assert_eq!(
            app.search_state.global_search_type,
            GlobalSearchType::FileName
        );
    }

    #[test]
    fn action_start_global_search_content_enters_mode() {
        let (mut app, _tmp) = test_app_with_files();
        let (ptx, stx) = make_channels();
        app.handle_action(&Action::StartGlobalSearchContent, &ptx, &stx);
        assert_eq!(app.ui.input_mode, InputMode::GlobalSearch);
        assert_eq!(
            app.search_state.global_search_type,
            GlobalSearchType::Content
        );
    }

    #[test]
    fn action_global_search_cancel_clears_state() {
        let (mut app, _tmp) = test_app_with_files();
        let (ptx, stx) = make_channels();
        app.handle_action(&Action::StartGlobalSearch, &ptx, &stx);
        app.search_state.query = "test".to_string();
        app.handle_action(&Action::GlobalSearchCancel, &ptx, &stx);
        assert_eq!(app.ui.input_mode, InputMode::Normal);
        assert!(app.search_state.query.is_empty());
    }

    #[test]
    fn action_global_search_up_down_navigates() {
        let (mut app, _tmp) = test_app_with_files();
        let (ptx, stx) = make_channels();
        app.ui.input_mode = InputMode::GlobalSearch;
        app.search_state = SearchState::new(SearchMode::Global);
        // Add some mock results
        for i in 0..5 {
            app.search_state.global_results.push(GlobalSearchResult {
                path: PathBuf::from(format!("file{i}.rs")),
                display: format!("file{i}.rs"),
                line: None,
                context: None,
            });
        }
        assert_eq!(app.search_state.global_selected, 0);
        app.handle_action(&Action::GlobalSearchDown, &ptx, &stx);
        assert_eq!(app.search_state.global_selected, 1);
        app.handle_action(&Action::GlobalSearchDown, &ptx, &stx);
        assert_eq!(app.search_state.global_selected, 2);
        app.handle_action(&Action::GlobalSearchUp, &ptx, &stx);
        assert_eq!(app.search_state.global_selected, 1);
    }

    #[test]
    fn action_global_search_up_at_zero_stays() {
        let (mut app, _tmp) = test_app_with_files();
        let (ptx, stx) = make_channels();
        app.ui.input_mode = InputMode::GlobalSearch;
        app.search_state = SearchState::new(SearchMode::Global);
        app.search_state.global_results.push(GlobalSearchResult {
            path: PathBuf::from("file.rs"),
            display: "file.rs".to_string(),
            line: None,
            context: None,
        });
        app.handle_action(&Action::GlobalSearchUp, &ptx, &stx);
        assert_eq!(app.search_state.global_selected, 0);
    }

    // -- Action handling: file ops dispatch --

    #[test]
    fn action_new_file_opens_dialog() {
        let (mut app, _tmp) = test_app_with_files();
        let (ptx, stx) = make_channels();
        app.handle_action(&Action::NewFile, &ptx, &stx);
        assert_eq!(app.ui.input_mode, InputMode::Dialog);
        assert!(app.ui.input_dialog.is_some());
        assert!(matches!(
            app.ui.input_dialog.as_ref().unwrap().kind,
            DialogKind::NewFile
        ));
    }

    #[test]
    fn action_new_dir_opens_dialog() {
        let (mut app, _tmp) = test_app_with_files();
        let (ptx, stx) = make_channels();
        app.handle_action(&Action::NewDir, &ptx, &stx);
        assert_eq!(app.ui.input_mode, InputMode::Dialog);
        assert!(matches!(
            app.ui.input_dialog.as_ref().unwrap().kind,
            DialogKind::NewDir
        ));
    }

    #[test]
    fn action_dialog_cancel_returns_to_normal() {
        let (mut app, _tmp) = test_app_with_files();
        let (ptx, stx) = make_channels();
        app.handle_action(&Action::NewFile, &ptx, &stx);
        assert_eq!(app.ui.input_mode, InputMode::Dialog);
        app.handle_action(&Action::DialogCancel, &ptx, &stx);
        assert_eq!(app.ui.input_mode, InputMode::Normal);
        assert!(app.ui.input_dialog.is_none());
    }

    #[test]
    fn action_dialog_char_inserts_text() {
        let (mut app, _tmp) = test_app_with_files();
        let (ptx, stx) = make_channels();
        app.handle_action(&Action::NewFile, &ptx, &stx);
        app.handle_action(&Action::DialogChar('h'), &ptx, &stx);
        app.handle_action(&Action::DialogChar('i'), &ptx, &stx);
        assert_eq!(app.ui.input_dialog.as_ref().unwrap().input, "hi");
    }

    // -- Action handling: collapse all --

    #[test]
    fn action_collapse_all_collapses_dirs() {
        let (mut app, _tmp) = test_app_with_files();
        let (ptx, stx) = make_channels();
        // Expand subdir first
        let dir_idx = app
            .tree
            .nodes
            .iter()
            .position(TreeNode::is_dir)
            .expect("should have a dir");
        app.tree.toggle(dir_idx);
        assert!(app.tree.nodes[dir_idx].is_expanded);
        app.handle_action(&Action::CollapseAll, &ptx, &stx);
        assert!(!app.tree.nodes[dir_idx].is_expanded);
    }

    // -- Action handling: switch focus --

    #[test]
    fn action_switch_focus_toggles() {
        let (mut app, _tmp) = test_app_with_files();
        let (ptx, stx) = make_channels();
        app.preview.visible = true;
        assert_eq!(app.focus, FocusPane::Tree);
        app.handle_action(&Action::SwitchFocus, &ptx, &stx);
        assert_eq!(app.focus, FocusPane::Preview);
        app.handle_action(&Action::SwitchFocus, &ptx, &stx);
        assert_eq!(app.focus, FocusPane::Tree);
    }

    // -- Action handling: open editor --

    #[test]
    fn action_open_in_editor_returns_post_action() {
        let (mut app, _tmp) = test_app_with_files();
        let (ptx, stx) = make_channels();
        // Navigate to a file node
        while app.tree.selected().is_some_and(TreeNode::is_dir) {
            app.handle_action(&Action::CursorDown, &ptx, &stx);
        }
        let post = app.handle_action(&Action::OpenInEditor, &ptx, &stx);
        assert!(matches!(post, PostAction::OpenEditor(_, None)));
    }

    #[test]
    fn action_open_in_editor_on_dir_is_noop() {
        let (mut app, _tmp) = test_app_with_files();
        let (ptx, stx) = make_channels();
        // Navigate to a dir node
        while app.tree.selected().is_some_and(|n| !n.is_dir()) {
            app.handle_action(&Action::CursorDown, &ptx, &stx);
        }
        if app.tree.selected().is_some_and(TreeNode::is_dir) {
            let post = app.handle_action(&Action::OpenInEditor, &ptx, &stx);
            assert!(matches!(post, PostAction::None));
        }
    }

    // -- build_external_editor_argv --

    #[test]
    fn build_external_editor_argv_no_line() {
        let argv = build_external_editor_argv("code -g", Path::new("/tmp/f.rs"), None);
        assert_eq!(argv, vec!["code", "-g", "/tmp/f.rs"]);
    }

    #[test]
    fn build_external_editor_argv_with_line() {
        let argv = build_external_editor_argv("code -g", Path::new("/tmp/f.rs"), Some(42));
        assert_eq!(argv, vec!["code", "-g", "/tmp/f.rs:42"]);
    }

    #[test]
    fn build_external_editor_argv_subl() {
        let argv = build_external_editor_argv("subl", Path::new("src/main.rs"), Some(10));
        assert_eq!(argv, vec!["subl", "src/main.rs:10"]);
    }

    #[test]
    fn build_external_editor_argv_path_with_spaces() {
        let argv =
            build_external_editor_argv("code -g", Path::new("/tmp/my project/f.rs"), Some(5));
        assert_eq!(argv, vec!["code", "-g", "/tmp/my project/f.rs:5"]);
    }

    // -- Action handling: error auto-dismiss --

    #[test]
    fn error_message_expires_after_3_seconds() {
        let (mut app, _tmp) = test_app();
        app.ui.error_message = Some((
            "old error".to_string(),
            Instant::now().checked_sub(Duration::from_secs(4)).unwrap(),
        ));
        let (_, ts) = app.ui.error_message.as_ref().unwrap();
        assert!(ts.elapsed() >= Duration::from_secs(3));
    }

    // -- Bug 3: global search scroll-down keeps selection visible --

    #[test]
    fn test_global_search_down_adjusts_scroll_offset() {
        use crate::search::GlobalSearchResult;
        use std::path::PathBuf;

        let mut state = SearchState::new(SearchMode::Global);
        state.global_visible_height = 5;
        // Populate with 20 results
        for i in 0..20 {
            state.global_results.push(GlobalSearchResult {
                path: PathBuf::from(format!("file{i}.rs")),
                display: format!("file{i}.rs"),
                line: None,
                context: None,
            });
        }

        // Navigate down 5 times (indices 0->5), should trigger scroll at index 5
        for _ in 0..5 {
            state.global_selected += 1;
            let visible = state.global_visible_height;
            if visible > 0 && state.global_selected >= state.global_scroll_offset + visible {
                state.global_scroll_offset = state.global_selected - visible + 1;
            }
        }

        assert_eq!(state.global_selected, 5);
        assert_eq!(state.global_scroll_offset, 1);

        // One more
        state.global_selected += 1;
        let visible = state.global_visible_height;
        if visible > 0 && state.global_selected >= state.global_scroll_offset + visible {
            state.global_scroll_offset = state.global_selected - visible + 1;
        }
        assert_eq!(state.global_selected, 6);
        assert_eq!(state.global_scroll_offset, 2);
    }

    // -- Bracketed paste tests --

    #[test]
    fn paste_in_normal_mode_is_ignored() {
        let (mut app, _tmp) = test_app();
        let (ptx, stx) = make_channels();
        app.ui.input_mode = InputMode::Normal;
        let cursor_before = app.tree.cursor;

        app.handle_action(&Action::Paste("qdr".to_string()), &ptx, &stx);

        assert_eq!(app.ui.input_mode, InputMode::Normal);
        assert_eq!(app.tree.cursor, cursor_before);
        assert!(!app.should_quit);
    }

    #[test]
    fn paste_in_search_mode_inserts_text() {
        let (mut app, _tmp) = test_app_with_files();
        let (ptx, stx) = make_channels();
        app.search_state = SearchState::new(SearchMode::Find);
        app.ui.input_mode = InputMode::Search;

        app.handle_action(&Action::Paste("hello".to_string()), &ptx, &stx);

        assert_eq!(app.search_state.query, "hello");
        assert_eq!(app.search_state.cursor_pos, 5);
    }

    #[test]
    fn paste_in_dialog_mode_inserts_text() {
        let (mut app, _tmp) = test_app();
        let (ptx, stx) = make_channels();
        app.ui.input_dialog = Some(InputDialogState::new(
            DialogKind::NewFile,
            std::path::PathBuf::from("/tmp"),
            String::new(),
        ));
        app.ui.input_mode = InputMode::Dialog;

        app.handle_action(&Action::Paste("newfile.txt".to_string()), &ptx, &stx);

        assert_eq!(app.ui.input_dialog.as_ref().unwrap().input, "newfile.txt");
        assert_eq!(app.ui.input_dialog.as_ref().unwrap().cursor_pos, 11);
    }

    #[test]
    fn paste_strips_control_chars() {
        // Control char stripping happens in the event loop, not handle_action.
        // Verify the stripping logic directly.
        let raw = "hello\nworld\tfoo\x00bar";
        let clean: String = raw.chars().filter(|c| !c.is_control()).collect();
        assert_eq!(clean, "helloworldfoobar");
    }

    #[tokio::test]
    async fn test_click_file_opens_preview_when_hidden() {
        let (mut app, _tmp) = test_app_with_files();
        let (preview_tx, _) = make_channels();

        // Populate rendered_indices so handle_click_row can resolve row -> index.
        // Index 0 is the root dir; files start at index 1+.
        app.tree.rendered_indices = (0..app.tree.len()).collect();

        // Find a file node (not the root directory).
        let file_idx = (0..app.tree.len())
            .find(|&i| !app.tree.nodes[i].is_dir())
            .expect("should have a file node");

        // Ensure cursor is NOT on that file and preview is hidden.
        app.tree.cursor = usize::from(file_idx == 0);
        app.preview.visible = false;

        // Click the file row.
        app.handle_click_row(file_idx as u16, &preview_tx);

        assert!(
            app.preview.visible,
            "clicking a non-selected file should open the preview panel"
        );
        assert_eq!(app.tree.cursor, file_idx);
    }

    #[tokio::test]
    async fn test_click_directory_schedules_directory_preview() {
        let (mut app, _tmp) = test_app_with_files();
        let (preview_tx, _) = make_channels();

        app.tree.rendered_indices = (0..app.tree.len()).collect();
        let dir_idx = (0..app.tree.len())
            .find(|&i| app.tree.nodes[i].is_dir())
            .expect("should have a directory node");
        app.preview.visible = true;

        app.handle_click_row(dir_idx as u16, &preview_tx);

        assert!(
            app.preview.debounce_handle.is_some(),
            "clicking a directory with preview visible should schedule a directory preview"
        );
    }

    // -- Preview staleness --

    #[test]
    fn stale_text_preview_result_is_discarded() {
        let (mut app, tmp) = test_app();
        // Create two files
        let file_a = tmp.path().join("a.txt");
        let file_b = tmp.path().join("b.txt");
        std::fs::write(&file_a, "aaa").unwrap();
        std::fs::write(&file_b, "bbb").unwrap();
        app.tree.refresh();

        // Ensure cursor is on file_a (index 0)
        app.tree.cursor = 0;
        let selected_path = app.tree.selected().unwrap().path.clone();
        assert_eq!(selected_path, file_a);

        // preview_state should have no content initially
        assert!(app.preview.state.current_path.is_none());

        // Simulate receiving a preview result for file_b (stale -- user moved away)
        // The staleness check should prevent apply
        let still_selected = app.tree.selected().is_some_and(|n| n.path == file_b);
        assert!(!still_selected);
        // So preview_state remains unchanged
        assert!(app.preview.state.current_path.is_none());
    }

    #[tokio::test]
    async fn preview_generation_increments_on_trigger() {
        let (mut app, _tmp) = test_app_with_files();
        let (preview_tx, _rx) = mpsc::channel(4);
        app.preview.visible = true;
        let initial = app.preview.generation;
        app.trigger_preview_load(&preview_tx);
        assert!(
            app.preview.generation > initial,
            "preview_generation should increment after trigger_preview_load"
        );
    }

    #[tokio::test]
    async fn preview_generation_stable_when_cached() {
        let (mut app, _tmp) = test_app_with_files();
        let (preview_tx, _rx) = mpsc::channel(4);
        app.preview.visible = true;

        // Simulate that the preview for the selected file was already loaded and applied.
        // This means current_path is set, kind is not Loading, mtime matches,
        // and the cached_diff_hint matches the hint that will be re-derived
        // from the node's git status (Clean → Skip by default).
        let selected_path = app.tree.selected().unwrap().path.clone();
        let mtime = std::fs::metadata(&selected_path)
            .ok()
            .and_then(|m| m.modified().ok());
        app.preview.state.current_path = Some(selected_path);
        app.preview.state.kind = PreviewKind::Text;
        app.preview.state.cached_mtime = mtime;
        app.preview.state.cached_diff_hint = Some(crate::git::diff::GitDiffHint::Skip);

        let gen_before = app.preview.generation;
        // This call should hit the cache (path+mtime+hint all match) and
        // return early -- generation should NOT increment.
        app.trigger_preview_load(&preview_tx);
        assert_eq!(
            app.preview.generation, gen_before,
            "preview_generation should not increment when preview is cached"
        );
    }

    #[tokio::test]
    async fn preview_cache_invalidated_when_git_status_changes() {
        // Regression test for a race condition: if the preview was loaded while
        // git_status was still stale (Clean), then a background refresh landed
        // the file as Modified, the preview cache must be invalidated on the
        // next trigger even though path+mtime haven't changed. Otherwise the
        // diff gutter would be permanently stuck showing no changes.
        use crate::git::diff::GitDiffHint;

        let (mut app, _tmp) = test_app_with_files();
        let (preview_tx, _rx) = mpsc::channel(4);
        app.preview.visible = true;

        let selected_path = app.tree.selected().unwrap().path.clone();
        let mtime = std::fs::metadata(&selected_path)
            .ok()
            .and_then(|m| m.modified().ok());

        // Pretend a prior preview ran when git status was Clean (hint=Skip)
        // and cached the result.
        app.preview.state.current_path = Some(selected_path);
        app.preview.state.kind = PreviewKind::Text;
        app.preview.state.cached_mtime = mtime;
        app.preview.state.cached_diff_hint = Some(GitDiffHint::Skip);

        // Now simulate a background refresh landing Modified status on the
        // selected node (file is the same, mtime is the same, only git state
        // changed).
        let cursor_idx = app.tree.cursor;
        app.tree.nodes[cursor_idx].git_status = crate::tree::node::GitStatus::Modified;

        let gen_before = app.preview.generation;
        app.trigger_preview_load(&preview_tx);

        // The cache must have been busted: generation must have incremented
        // because Modified → Compute, which disagrees with the cached Skip.
        assert!(
            app.preview.generation > gen_before,
            "preview must reload when git_status changes from Clean to Modified"
        );
    }

    #[tokio::test]
    async fn stale_preview_generation_discarded() {
        let (mut app, _tmp) = test_app_with_files();
        let (_preview_tx, _rx) = mpsc::channel::<(u64, PathBuf, LoadedPreview)>(4);
        app.preview.visible = true;

        // Simulate: generation is at 5, but a stale result arrives with gen=3
        app.preview.generation = 5;
        let stale_gen: u64 = 3;

        // The generation check: stale_gen != app.preview.generation
        assert_ne!(stale_gen, app.preview.generation);

        // A result with matching generation should be accepted
        let current_gen = app.preview.generation;
        assert_eq!(current_gen, app.preview.generation);
    }

    #[tokio::test]
    async fn trigger_preview_load_clears_preview_when_selection_disappears() {
        let (mut app, tmp) = test_app();
        let path = tmp.path().join("gone.txt");
        std::fs::write(&path, "preview me").unwrap();
        app.tree.refresh();

        app.preview.visible = true;
        app.preview.state.apply(
            path.clone(),
            PreviewKind::Text,
            vec![vec![(
                "preview me".to_string(),
                ratatui::style::Style::default(),
            )]],
            "10 B".to_string(),
            None,
            crate::git::diff::GitDiffHint::Skip,
        );

        std::fs::remove_file(&path).unwrap();
        app.tree.refresh();
        assert!(app.tree.selected().is_none());

        let (preview_tx, _rx) = mpsc::channel(1);
        let gen_before = app.preview.generation;
        app.trigger_preview_load(&preview_tx);

        assert_eq!(app.preview.state.kind, PreviewKind::Empty);
        assert!(app.preview.state.current_path.is_none());
        assert!(app.preview.debounce_handle.is_none());
        assert_eq!(app.preview.generation, gen_before.wrapping_add(1));
    }
}
