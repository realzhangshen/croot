use std::path::PathBuf;
use std::time::Duration;

use crossterm::event::{
    DisableMouseCapture, EnableMouseCapture, Event, EventStream, KeyboardEnhancementFlags,
    PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags,
};
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen};
use futures::StreamExt;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::Style,
    widgets::{StatefulWidget, Widget},
    Terminal,
};
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

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
use crate::preview::state::{PreviewKind, PreviewState};
use crate::render::colors;
use crate::render::context_menu::{ContextMenuState, ContextMenuWidget, MenuAction};
use crate::render::global_search::GlobalSearchOverlay;
use crate::render::input_dialog::{DialogKind, InputDialogState, InputDialogWidget};
use crate::render::picker::{PickerState, PickerWidget};
use crate::render::preview_view::PreviewView;
use crate::render::search_bar::{
    do_match, do_match_positions, GlobalSearchResult, GlobalSearchType, SearchBar, SearchMode,
    SearchState,
};
use crate::render::status_bar::{HyperlinkRegion, StatusBar};
use crate::render::tree_view::TreeView;
use crate::tree::forest::FileTree;

/// Signal from action handling that requires terminal-level processing.
pub enum PostAction {
    None,
    OpenEditor(PathBuf),
}

#[allow(clippy::struct_excessive_bools)]
pub struct App {
    pub tree: FileTree,
    pub git: Option<GitState>,
    pub cmux: Option<CmuxBridge>,
    pub config: Config,
    pub root: PathBuf,
    pub should_quit: bool,
    tree_area_y: u16,
    tree_area_height: u16,
    // Preview panel state
    pub preview_state: PreviewState,
    pub preview_visible: bool,
    pub focus: FocusPane,
    preview_debounce_handle: Option<JoinHandle<()>>,
    preview_area_x: Option<u16>,
    preview_layout: Option<PreviewLayout>,
    preview_content_width: u16,
    dragging_separator: bool,
    main_area_width: u16,
    hover_row: Option<usize>,
    // UI overlay state
    input_mode: InputMode,
    context_menu: Option<ContextMenuState>,
    input_dialog: Option<InputDialogState>,
    picker_state: Option<PickerState>,
    // Search state
    search_state: SearchState,
    /// Handle for the current async global search task.
    global_search_handle: Option<JoinHandle<()>>,
    // Hyperlink regions for post-render OSC 8 emission
    hyperlink_regions: Vec<HyperlinkRegion>,
    // Whether Kitty keyboard enhancement protocol is active
    enhanced_keyboard: bool,
    // Double-click detection for tree rows
    click_tracker: ClickTracker,
    // User-configured keybindings
    keybinding_map: KeybindingMap,
    // Whether mouse capture is enabled
    mouse_enabled: bool,
    // Status/search bar y-coordinates for mouse routing
    status_bar_y: u16,
    search_bar_y: Option<u16>,
    // Status bar branch click region: (x_start, x_end)
    status_bar_branch_region: Option<(u16, u16)>,
}

impl App {
    pub fn new(root: PathBuf, enhanced_keyboard: bool, config: Config) -> anyhow::Result<Self> {
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

        Ok(Self {
            tree,
            git,
            cmux,
            config,
            root,
            should_quit: false,
            tree_area_y: 0,
            tree_area_height: 0,
            preview_state: {
                let mut ps = PreviewState::new();
                ps.render_markdown = render_markdown;
                ps
            },
            preview_visible,
            focus: FocusPane::Tree,
            preview_debounce_handle: None,
            preview_area_x: None,
            preview_layout: None,
            preview_content_width: 80,
            dragging_separator: false,
            main_area_width: 0,
            hover_row: None,
            input_mode: InputMode::Normal,
            context_menu: None,
            input_dialog: None,
            picker_state: None,
            search_state: SearchState::new(SearchMode::Find),
            global_search_handle: None,
            hyperlink_regions: Vec::new(),
            enhanced_keyboard,
            click_tracker: ClickTracker::new(),
            keybinding_map,
            mouse_enabled,
            status_bar_y: 0,
            search_bar_y: None,
            status_bar_branch_region: None,
        })
    }

    pub async fn run<B: ratatui::backend::Backend>(
        &mut self,
        terminal: &mut Terminal<B>,
    ) -> anyhow::Result<()>
    where
        B::Error: Send + Sync + 'static,
    {
        let mut reader = EventStream::new();

        // Set up file watcher with 100ms debounce
        let (fs_tx, mut fs_rx) = mpsc::channel::<()>(1);
        let _watcher = crate::watcher::setup_watcher(&self.root, fs_tx);
        let mut watcher_active = true;

        // Channel for receiving loaded preview results
        let (preview_tx, mut preview_rx) = mpsc::channel::<(PathBuf, LoadedPreview)>(4);

        // Channel for receiving global search results
        let (search_tx, mut search_rx) =
            mpsc::channel::<(u64, Vec<GlobalSearchResult>, Option<String>)>(4);

        // Trigger initial preview load if auto_preview is on
        if self.preview_visible {
            self.trigger_preview_load(&preview_tx);
        }

        let mut post_action = PostAction::None;

        loop {
            terminal.draw(|frame| self.draw(frame))?;
            self.emit_osc8_hyperlinks()?;

            tokio::select! {
                event = reader.next() => {
                    match event {
                        Some(Ok(Event::Key(key))) => {
                            let action = match self.input_mode {
                                InputMode::Normal => {
                                    let has_selection = self.preview_state.selection.is_active();
                                    let action = handle_key(key, self.preview_visible, has_selection, &self.keybinding_map);
                                    if self.focus == FocusPane::Preview {
                                        match action {
                                            Action::ScrollUp(n) => Action::PreviewScrollUp(n),
                                            Action::ScrollDown(n) => Action::PreviewScrollDown(n),
                                            a => a,
                                        }
                                    } else {
                                        action
                                    }
                                }
                                InputMode::ContextMenu => handle_key_menu(key),
                                InputMode::Dialog => handle_key_dialog(key),
                                InputMode::Search => handle_key_search(key),
                                InputMode::Picker => handle_key_picker(key),
                                InputMode::GlobalSearch => handle_key_global_search(key),
                            };
                            post_action = self.handle_action(&action, &preview_tx, &search_tx);
                        }
                        Some(Ok(Event::Mouse(mouse))) if self.mouse_enabled => {
                            use crossterm::event::{MouseButton, MouseEventKind};

                            if self.input_mode == InputMode::ContextMenu {
                                post_action = self.handle_context_menu_mouse(mouse);
                            } else if self.input_mode == InputMode::Picker {
                                post_action = self.handle_picker_mouse(mouse);
                            } else if self.input_mode == InputMode::Dialog {
                                // R5: Click outside dialog cancels it
                                post_action = self.handle_dialog_mouse(mouse);
                            } else if self.input_mode == InputMode::GlobalSearch {
                                post_action = self.handle_global_search_mouse(mouse);
                            } else {
                                // Route by area priority: status > search > tree/preview
                                let is_left_down = matches!(mouse.kind, MouseEventKind::Down(MouseButton::Left));

                                if mouse.row == self.status_bar_y && is_left_down {
                                    post_action = self.handle_status_bar_click(mouse.column, &preview_tx);
                                } else if self.search_bar_y.is_some_and(|y| mouse.row == y) && is_left_down {
                                    post_action = self.handle_search_bar_click(mouse.column, &preview_tx);
                                } else {
                                    let action = handle_mouse(mouse, self.tree_area_y, self.tree_area_height, self.preview_area_x, &mut self.click_tracker);
                                    post_action = self.handle_action(&action, &preview_tx, &search_tx);
                                }
                            }
                        }
                        Some(Ok(Event::Resize(_, _))) => {
                            self.context_menu = None;
                            self.picker_state = None;
                            self.input_mode = InputMode::Normal;
                            if self.preview_visible {
                                self.trigger_preview_load(&preview_tx);
                            }
                        }
                        Some(Err(_)) | None => break,
                        _ => {}
                    }
                }
                result = fs_rx.recv(), if watcher_active => {
                    if result.is_none() {
                        watcher_active = false;
                        continue;
                    }
                    self.tree.refresh();
                    if let Some(ref mut git) = self.git {
                        git.refresh();
                    }
                    self.reapply_git();
                    self.refresh_search_state();
                    if self.preview_visible {
                        self.trigger_preview_load(&preview_tx);
                    }
                }
                result = search_rx.recv() => {
                    if let Some((id, results, error)) = result {
                        if id == self.search_state.request_id {
                            self.search_state.global_results = results;
                            self.search_state.global_error = error;
                            self.search_state.global_loading = false;
                            self.search_state.global_selected = 0;
                            self.search_state.global_scroll_offset = 0;
                        }
                    }
                }
                result = preview_rx.recv() => {
                    if let Some((path, loaded)) = result {
                        self.preview_state.apply(path, loaded.kind, loaded.content, loaded.file_info);
                    }
                }
            }

            // Process post-actions that require terminal access
            if let PostAction::OpenEditor(path) =
                std::mem::replace(&mut post_action, PostAction::None)
            {
                self.open_editor_suspend(terminal, &path)?;
                // Refresh tree, git, preview after editor exits
                self.tree.refresh();
                if let Some(ref mut git) = self.git {
                    git.refresh();
                }
                self.reapply_git();
                if self.preview_visible {
                    self.trigger_preview_load(&preview_tx);
                }
                // Recreate event stream to flush stale buffered events
                reader = EventStream::new();
            }

            if self.should_quit {
                break;
            }
        }

        Ok(())
    }

    fn emit_osc8_hyperlinks(&self) -> anyhow::Result<()> {
        use std::io::Write;
        let mut stdout = std::io::stdout();
        for region in &self.hyperlink_regions {
            crossterm::queue!(stdout, crossterm::cursor::MoveTo(region.x, region.y))?;
            crossterm::queue!(
                stdout,
                crossterm::style::SetAttribute(crossterm::style::Attribute::Reverse)
            )?;
            write!(
                stdout,
                "\x1b]8;;{}\x07{}\x1b]8;;\x07",
                region.url, region.text
            )?;
            crossterm::queue!(
                stdout,
                crossterm::style::SetAttribute(crossterm::style::Attribute::Reset)
            )?;
        }
        stdout.flush()?;
        Ok(())
    }

    fn draw(&mut self, frame: &mut ratatui::Frame) {
        let size = frame.area();

        let show_search_bar = self.input_mode == InputMode::Search
            || (!self.search_state.is_empty() && self.search_state.mode != SearchMode::Global);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(if show_search_bar {
                vec![
                    Constraint::Min(1),
                    Constraint::Length(1),
                    Constraint::Length(1),
                ]
            } else {
                vec![Constraint::Min(1), Constraint::Length(1)]
            })
            .split(size);

        let main_area = chunks[0];
        let status_area = chunks[1];

        // Store status/search bar y for mouse routing
        self.status_bar_y = status_area.y;
        self.search_bar_y = if show_search_bar {
            Some(chunks[2].y)
        } else {
            None
        };

        let content_area = main_area;
        self.tree_area_y = content_area.y;
        self.main_area_width = main_area.width;

        if self.preview_visible && content_area.width > 20 {
            // Split horizontally: tree | separator | preview
            let ratio = self.config.preview.split_ratio.clamp(0.2, 0.8);
            let tree_width = (f32::from(content_area.width) * (1.0 - ratio)) as u16;
            let separator_width: u16 = 1;
            let preview_width = content_area
                .width
                .saturating_sub(tree_width + separator_width);

            let tree_area = ratatui::layout::Rect {
                x: content_area.x,
                y: content_area.y,
                width: tree_width,
                height: content_area.height,
            };
            let separator_area = ratatui::layout::Rect {
                x: content_area.x + tree_width,
                y: content_area.y,
                width: separator_width,
                height: content_area.height,
            };
            let preview_area = ratatui::layout::Rect {
                x: content_area.x + tree_width + separator_width,
                y: content_area.y,
                width: preview_width,
                height: content_area.height,
            };

            self.tree_area_height = tree_area.height;

            TreeView {
                config: &self.config.tree,
                hover_row: self.hover_row,
                filter_indices: if self.search_state.mode == SearchMode::Filter
                    && !self.search_state.visible_indices.is_empty()
                {
                    &self.search_state.visible_indices
                } else {
                    &[]
                },
                highlight_indices: if self.search_state.mode == SearchMode::Find
                    && !self.search_state.match_indices.is_empty()
                {
                    &self.search_state.match_indices
                } else {
                    &[]
                },
                highlight_char_positions: &self.search_state.match_char_positions,
            }
            .render(tree_area, frame.buffer_mut(), &mut self.tree);

            let sep_style = Style::default().fg(colors::TREE_LINE);
            for y in separator_area.y..separator_area.y + separator_area.height {
                frame
                    .buffer_mut()
                    .set_string(separator_area.x, y, "│", sep_style);
            }

            self.preview_content_width = preview_width;
            self.preview_area_x = Some(preview_area.x);

            let content_area_y = preview_area.y + 1;
            let content_area_height = preview_area.height.saturating_sub(1);
            let gutter_width = if self.config.preview.show_line_numbers
                && self.preview_state.kind == PreviewKind::Text
            {
                let digits = if self.preview_state.total_lines == 0 {
                    1
                } else {
                    (self.preview_state.total_lines as f64).log10().floor() as u16 + 1
                };
                digits + 1
            } else {
                0
            };
            self.preview_layout = Some(PreviewLayout {
                x: preview_area.x + gutter_width,
                y: content_area_y,
                height: content_area_height,
            });

            PreviewView {
                config: &self.config.preview,
                focused: self.focus == FocusPane::Preview,
            }
            .render(preview_area, frame.buffer_mut(), &mut self.preview_state);
        } else {
            self.preview_area_x = None;
            self.preview_layout = None;
            self.tree_area_height = content_area.height;

            TreeView {
                config: &self.config.tree,
                hover_row: self.hover_row,
                filter_indices: if self.search_state.mode == SearchMode::Filter
                    && !self.search_state.visible_indices.is_empty()
                {
                    &self.search_state.visible_indices
                } else {
                    &[]
                },
                highlight_indices: if self.search_state.mode == SearchMode::Find
                    && !self.search_state.match_indices.is_empty()
                {
                    &self.search_state.match_indices
                } else {
                    &[]
                },
                highlight_char_positions: &self.search_state.match_char_positions,
            }
            .render(content_area, frame.buffer_mut(), &mut self.tree);
        }

        let root_name = self.root.file_name().map_or_else(
            || self.root.to_string_lossy().into_owned(),
            |n| n.to_string_lossy().into_owned(),
        );
        let root_path = self.root.to_string_lossy().into_owned();

        let selected_rel = self.tree.selected().and_then(|n| {
            if n.is_dir() {
                None
            } else {
                n.path
                    .strip_prefix(&self.root)
                    .ok()
                    .map(|p| p.to_string_lossy().into_owned())
            }
        });
        let selected_abs = self.tree.selected().and_then(|n| {
            if n.is_dir() {
                None
            } else {
                Some(n.path.to_string_lossy().into_owned())
            }
        });

        let file_count = self.tree.file_count;
        let dir_count = self.tree.dir_count;
        let branch = self
            .git
            .as_ref()
            .and_then(|g| g.branch())
            .map(std::string::ToString::to_string);
        let cmux_indicator = if self.cmux.is_some() {
            Some("cmux")
        } else {
            None
        };

        let status_bar = StatusBar {
            branch: branch.as_deref(),
            file_count,
            dir_count,
            root_name: &root_name,
            root_path: &root_path,
            cmux_status: cmux_indicator,
            selected_path: selected_rel.as_deref(),
            selected_abs_path: selected_abs.as_deref(),
        };
        // Track branch click region for mouse routing
        self.status_bar_branch_region = branch.as_ref().map(|b| {
            // Branch is rendered as "  {branch} │ " starting at col 0
            let end = (2 + b.len() + 1) as u16; // "  branch "
            (0, end)
        });

        self.hyperlink_regions = status_bar.hyperlink_regions(status_area);
        status_bar.render(status_area, frame.buffer_mut());

        // Search bar (shown when in search mode or filter is active)
        if show_search_bar {
            let search_area = chunks[2];
            let search_bar = SearchBar {
                state: &self.search_state,
                show_close_button: true,
            };
            search_bar.render(search_area, frame.buffer_mut());
        }

        // Render overlays (context menu / input dialog)
        if let Some(ref menu) = self.context_menu {
            let widget = ContextMenuWidget { state: menu };
            widget.render(size, frame.buffer_mut());
        }

        if let Some(ref dialog) = self.input_dialog {
            let widget = InputDialogWidget { state: dialog };
            widget.render(size, frame.buffer_mut());
        }

        if let Some(ref mut picker) = self.picker_state {
            PickerWidget::render_mut(picker, size, frame.buffer_mut());
        }

        // Global search overlay
        if self.input_mode == InputMode::GlobalSearch {
            let overlay = GlobalSearchOverlay {
                state: &self.search_state,
            };
            overlay.render(size, frame.buffer_mut());
        }
    }

    fn handle_action(
        &mut self,
        action: &Action,
        preview_tx: &mpsc::Sender<(PathBuf, LoadedPreview)>,
        search_tx: &mpsc::Sender<(u64, Vec<GlobalSearchResult>, Option<String>)>,
    ) -> PostAction {
        let mut post = PostAction::None;
        match *action {
            Action::Quit => {
                if self.input_mode == InputMode::Normal {
                    self.should_quit = true;
                } else {
                    self.input_mode = InputMode::Normal;
                    self.context_menu = None;
                    self.input_dialog = None;
                    self.picker_state = None;
                }
            }

            // Tree actions
            Action::CursorUp
            | Action::CursorDown
            | Action::CursorLeft
            | Action::CursorRight
            | Action::Toggle
            | Action::Refresh
            | Action::ScrollUp(_)
            | Action::ScrollDown(_)
            | Action::GotoTop
            | Action::GotoBottom => {
                self.handle_tree_action(action);
            }

            // Preview actions
            Action::PreviewScrollUp(_) | Action::PreviewScrollDown(_) | Action::SwitchFocus => {
                self.handle_preview_action(action);
            }
            Action::TogglePreview => {
                self.preview_visible = !self.preview_visible;
                if self.preview_visible {
                    self.trigger_preview_load(preview_tx);
                } else {
                    self.focus = FocusPane::Tree;
                }
            }
            Action::ToggleRender => {
                self.preview_state.render_markdown = !self.preview_state.render_markdown;
                self.preview_state.cached_mtime = None;
                if self.preview_visible {
                    self.trigger_preview_load(preview_tx);
                }
            }

            // Separator drag
            Action::SeparatorDragStart => {
                self.dragging_separator = true;
            }
            Action::DragUpdate(col, row) => {
                if self.dragging_separator {
                    if self.main_area_width > 0 {
                        let ratio = 1.0 - (f32::from(col) / f32::from(self.main_area_width));
                        self.config.preview.split_ratio = ratio.clamp(0.2, 0.8);
                    }
                } else if self.preview_area_x.is_some_and(|px| col >= px) {
                    self.handle_selection_action(&Action::SelectionUpdate(col, row));
                }
            }

            // Selection actions
            Action::SelectionStart(_, _)
            | Action::SelectionUpdate(_, _)
            | Action::CopySelection
            | Action::ClearSelection => {
                self.dragging_separator = false;
                self.handle_selection_action(action);
            }

            // Click routing
            Action::ClickRow(row) => {
                self.dragging_separator = false;
                self.handle_click_row(row, preview_tx);
            }

            Action::Hover(col, row) => {
                self.update_hover(col, row);
            }

            // Right-click context menu
            Action::RightClick(col, row) => {
                self.open_context_menu(col, row);
            }

            // Context menu actions
            Action::MenuClose => {
                self.context_menu = None;
                self.input_mode = InputMode::Normal;
            }
            Action::MenuUp => {
                if let Some(ref mut menu) = self.context_menu {
                    menu.move_up();
                }
            }
            Action::MenuDown => {
                if let Some(ref mut menu) = self.context_menu {
                    menu.move_down();
                }
            }
            Action::MenuSelect(ref _placeholder) => {
                // Resolve actual action from selected menu item
                if let Some(menu) = self.context_menu.take() {
                    let menu_action = menu.selected_action().clone();
                    self.input_mode = InputMode::Normal;
                    post = self.execute_menu_action(&menu_action, menu.node_idx, preview_tx);
                }
            }
            // File operations (keyboard shortcuts)
            Action::NewFile => self.start_new_file(),
            Action::NewDir => self.start_new_dir(),
            Action::RenameNode => self.start_rename(),
            Action::DeleteNode => self.start_delete(),

            // Dialog actions
            Action::DialogChar(ch) => {
                if let Some(ref mut dialog) = self.input_dialog {
                    dialog.insert_char(ch);
                }
            }
            Action::DialogBackspace => {
                if let Some(ref mut dialog) = self.input_dialog {
                    dialog.delete_char();
                }
            }
            Action::DialogLeft => {
                if let Some(ref mut dialog) = self.input_dialog {
                    dialog.move_left();
                }
            }
            Action::DialogRight => {
                if let Some(ref mut dialog) = self.input_dialog {
                    dialog.move_right();
                }
            }
            Action::DialogConfirm => {
                self.confirm_dialog();
            }
            Action::DialogCancel => {
                self.input_dialog = None;
                self.input_mode = InputMode::Normal;
            }

            // Search actions — Find mode
            Action::StartFind => {
                self.search_state = SearchState::new(SearchMode::Find);
                self.search_state.origin_cursor = self.tree.cursor;
                self.search_state.origin_scroll_offset = self.tree.scroll_offset;
                self.input_mode = InputMode::Search;
            }
            // Search actions — Filter mode
            Action::StartFilter => {
                self.search_state = SearchState::new(SearchMode::Filter);
                self.search_state.origin_cursor = self.tree.cursor;
                self.search_state.origin_scroll_offset = self.tree.scroll_offset;
                self.input_mode = InputMode::Search;
            }
            Action::SearchChar(ch) => {
                self.search_state.insert_char(ch);
                match self.search_state.mode {
                    SearchMode::Find => self.update_find_matches(),
                    SearchMode::Filter => self.update_filter_view(),
                    SearchMode::Global => {}
                }
            }
            Action::SearchBackspace => {
                self.search_state.delete_char();
                match self.search_state.mode {
                    SearchMode::Find => self.update_find_matches(),
                    SearchMode::Filter => self.update_filter_view(),
                    SearchMode::Global => {}
                }
            }
            Action::SearchLeft => {
                self.search_state.move_left();
            }
            Action::SearchRight => {
                self.search_state.move_right();
            }
            Action::SearchConfirm => {
                match self.search_state.mode {
                    SearchMode::Find => {
                        // Exit search, clear highlights, cursor stays
                        self.input_mode = InputMode::Normal;
                        self.search_state.clear();
                    }
                    SearchMode::Filter => {
                        // Exit input but keep filter active
                        self.input_mode = InputMode::Normal;
                    }
                    SearchMode::Global => {}
                }
            }
            Action::SearchCancel => {
                // Restore cursor and clear all search state
                self.tree.cursor = self.search_state.origin_cursor;
                self.tree.scroll_offset = self.search_state.origin_scroll_offset;
                self.input_mode = InputMode::Normal;
                self.search_state.clear();
            }
            Action::SearchNext => {
                self.search_navigate_next();
            }
            Action::SearchPrev => {
                self.search_navigate_prev();
            }
            // Global search actions
            Action::StartGlobalSearch => {
                self.search_state = SearchState::new(SearchMode::Global);
                self.search_state.global_search_type = GlobalSearchType::FileName;
                self.input_mode = InputMode::GlobalSearch;
            }
            Action::StartGlobalSearchContent => {
                self.search_state = SearchState::new(SearchMode::Global);
                self.search_state.global_search_type = GlobalSearchType::Content;
                self.input_mode = InputMode::GlobalSearch;
            }
            Action::GlobalSearchChar(ch) => {
                self.search_state.insert_char(ch);
                self.spawn_global_search(search_tx);
            }
            Action::GlobalSearchBackspace => {
                self.search_state.delete_char();
                if self.search_state.query.is_empty() {
                    self.search_state.global_results.clear();
                    self.search_state.global_error = None;
                    self.search_state.global_loading = false;
                } else {
                    self.spawn_global_search(search_tx);
                }
            }
            Action::GlobalSearchUp => {
                if self.search_state.global_selected > 0 {
                    self.search_state.global_selected -= 1;
                    if self.search_state.global_selected < self.search_state.global_scroll_offset {
                        self.search_state.global_scroll_offset = self.search_state.global_selected;
                    }
                }
            }
            Action::GlobalSearchDown => {
                if !self.search_state.global_results.is_empty()
                    && self.search_state.global_selected + 1
                        < self.search_state.global_results.len()
                {
                    self.search_state.global_selected += 1;
                }
            }
            Action::GlobalSearchConfirm => {
                if let Some(result) = self
                    .search_state
                    .global_results
                    .get(self.search_state.global_selected)
                    .cloned()
                {
                    self.input_mode = InputMode::Normal;
                    self.search_state.clear();
                    let path = result.path;
                    self.tree.navigate_to_path(&path);
                    self.reapply_git();
                }
            }
            Action::GlobalSearchCancel => {
                self.search_state.clear();
                self.input_mode = InputMode::Normal;
                if let Some(handle) = self.global_search_handle.take() {
                    handle.abort();
                }
            }

            // Open file in editor
            Action::OpenInEditor => {
                if let Some(node) = self.tree.selected() {
                    if !node.is_dir() {
                        post = PostAction::OpenEditor(node.path.clone());
                    }
                }
            }
            Action::OpenExternally => {
                if let Some(node) = self.tree.selected() {
                    if !node.is_dir() {
                        let path = node.path.clone();
                        self.open_externally(&path);
                    }
                }
            }
            // Collapse all directories
            Action::CollapseAll => {
                self.tree.collapse_all();
                self.reapply_git();
                self.refresh_search_state();
                if self.preview_visible {
                    self.trigger_preview_load(preview_tx);
                }
            }
            // Focus search bar without clearing query
            Action::FocusSearch => {
                self.input_mode = InputMode::Search;
            }

            Action::DoubleClick(row) => {
                // Cursor already set by the first ClickRow. For files, open externally.
                // For directories, do nothing — the first click already toggled.
                let row_idx = row as usize;
                if row_idx < self.tree.rendered_indices.len() {
                    let idx = self.tree.rendered_indices[row_idx];
                    if idx < self.tree.len() && !self.tree.nodes[idx].is_dir() {
                        let path = self.tree.nodes[idx].path.clone();
                        self.open_externally(&path);
                    }
                }
            }
            // Branch picker
            Action::OpenBranchPicker => {
                if self.git.is_some() {
                    self.open_branch_picker();
                }
            }
            Action::PickerChar(ch) => {
                if let Some(ref mut picker) = self.picker_state {
                    picker.insert_char(ch);
                }
            }
            Action::PickerBackspace => {
                if let Some(ref mut picker) = self.picker_state {
                    picker.delete_char();
                }
            }
            Action::PickerUp => {
                if let Some(ref mut picker) = self.picker_state {
                    picker.move_up();
                }
            }
            Action::PickerDown => {
                if let Some(ref mut picker) = self.picker_state {
                    picker.move_down();
                }
            }
            Action::PickerConfirm => {
                self.confirm_picker();
            }
            Action::PickerCancel => {
                self.picker_state = None;
                self.input_mode = InputMode::Normal;
            }

            Action::EnterKey => {
                let selected_is_dir = self
                    .tree
                    .selected()
                    .is_some_and(crate::tree::node::TreeNode::is_dir);
                if selected_is_dir {
                    let idx = self.tree.cursor;
                    self.tree.toggle(idx);
                    self.reapply_git();
                    self.refresh_search_state();
                } else if let Some(path) = self.tree.selected().map(|n| n.path.clone()) {
                    post = PostAction::OpenEditor(path);
                }
            }

            Action::None => {}
        }
        post
    }

    fn handle_tree_action(&mut self, action: &Action) {
        match action {
            Action::CursorUp => {
                if self.focus == FocusPane::Preview {
                    self.preview_state.scroll_up(1);
                } else {
                    self.tree.cursor_up();
                }
            }
            Action::CursorDown => {
                if self.focus == FocusPane::Preview {
                    self.preview_state.scroll_down(1);
                } else {
                    self.tree.cursor_down();
                }
            }
            Action::CursorLeft => {
                if self.focus == FocusPane::Tree {
                    self.tree.cursor_left();
                }
            }
            Action::CursorRight => {
                if self.focus == FocusPane::Tree {
                    self.tree.cursor_right();
                    self.reapply_git();
                    self.refresh_search_state();
                }
            }
            Action::Toggle => {
                let idx = self.tree.cursor;
                self.tree.toggle(idx);
                self.reapply_git();
                self.refresh_search_state();
            }
            Action::Refresh => {
                self.tree.refresh();
                if let Some(ref mut git) = self.git {
                    git.refresh();
                }
                self.reapply_git();
                self.refresh_search_state();
            }
            Action::ScrollUp(n) => {
                for _ in 0..*n {
                    self.tree.cursor_up();
                }
            }
            Action::ScrollDown(n) => {
                for _ in 0..*n {
                    self.tree.cursor_down();
                }
            }
            Action::GotoTop => {
                if self.focus == FocusPane::Preview {
                    self.preview_state.scroll_offset = 0;
                } else {
                    self.tree.cursor = 0;
                }
            }
            Action::GotoBottom => {
                if self.focus == FocusPane::Preview {
                    self.preview_state.scroll_offset =
                        self.preview_state.total_lines.saturating_sub(1);
                } else if !self.tree.is_empty() {
                    self.tree.cursor = self.tree.len() - 1;
                }
            }
            _ => {}
        }
    }

    fn handle_preview_action(&mut self, action: &Action) {
        match action {
            Action::PreviewScrollUp(n) => self.preview_state.scroll_up(*n as usize),
            Action::PreviewScrollDown(n) => self.preview_state.scroll_down(*n as usize),
            Action::SwitchFocus => {
                self.focus = match self.focus {
                    FocusPane::Tree => FocusPane::Preview,
                    FocusPane::Preview => FocusPane::Tree,
                };
            }
            _ => {}
        }
    }

    fn handle_selection_action(&mut self, action: &Action) {
        match action {
            Action::SelectionStart(col, row) => {
                self.focus = FocusPane::Preview;
                if let Some(pos) = self.screen_to_content(*col, *row) {
                    self.preview_state.selection.anchor = Some(pos);
                    self.preview_state.selection.cursor = Some(pos);
                } else {
                    self.preview_state.selection.clear();
                }
            }
            Action::SelectionUpdate(col, row) => {
                if self.preview_state.selection.anchor.is_some() {
                    if let Some(pos) = self.screen_to_content(*col, *row) {
                        self.preview_state.selection.cursor = Some(pos);
                    }
                }
            }
            Action::CopySelection => {
                if let Some(text) = self.preview_state.extract_selected_text() {
                    if let Ok(mut clipboard) = arboard::Clipboard::new() {
                        let _ = clipboard.set_text(text);
                    }
                }
                self.preview_state.selection.clear();
            }
            Action::ClearSelection => {
                self.preview_state.selection.clear();
            }
            _ => {}
        }
    }

    fn handle_click_row(&mut self, row: u16, preview_tx: &mpsc::Sender<(PathBuf, LoadedPreview)>) {
        self.focus = FocusPane::Tree;
        self.preview_state.selection.clear();
        let row_idx = row as usize;
        let idx = if row_idx < self.tree.rendered_indices.len() {
            self.tree.rendered_indices[row_idx]
        } else {
            return;
        };
        if idx < self.tree.len() {
            let already_selected = self.tree.cursor == idx;
            self.tree.cursor = idx;
            if self.tree.nodes[idx].is_dir() {
                self.tree.toggle(idx);
                self.reapply_git();
            } else if already_selected && self.preview_visible {
                self.preview_visible = false;
                self.focus = FocusPane::Tree;
            } else {
                self.preview_visible = true;
                self.trigger_preview_load(preview_tx);
            }
        }
    }

    /// Schedule a debounced preview load for the currently selected file.
    fn trigger_preview_load(&mut self, preview_tx: &mpsc::Sender<(PathBuf, LoadedPreview)>) {
        let Some(node) = self.tree.selected() else {
            return;
        };

        if node.is_dir() {
            self.preview_state.clear();
            return;
        }

        let path = node.path.clone();

        if self.preview_state.current_path.as_ref() == Some(&path)
            && self.preview_state.kind != PreviewKind::Loading
        {
            let current_mtime = std::fs::metadata(&path)
                .ok()
                .and_then(|m| m.modified().ok());
            if current_mtime == self.preview_state.cached_mtime {
                return;
            }
        }

        if let Some(handle) = self.preview_debounce_handle.take() {
            handle.abort();
        }

        self.preview_state.kind = PreviewKind::Loading;

        let tx = preview_tx.clone();
        let delay = Duration::from_millis(self.config.preview.preview_delay_ms);
        let max_file_size_kb = self.config.preview.max_file_size_kb;
        let syntax_highlight = self.config.preview.syntax_highlight;
        let render_markdown = self.preview_state.render_markdown;
        let preview_width = self.preview_content_width as usize;

        self.preview_debounce_handle = Some(tokio::spawn(async move {
            tokio::time::sleep(delay).await;

            let path_for_send = path.clone();
            let loaded = tokio::task::spawn_blocking(move || {
                load_preview(
                    &path,
                    max_file_size_kb,
                    syntax_highlight,
                    render_markdown,
                    preview_width,
                )
            })
            .await;

            if let Ok(loaded) = loaded {
                let _ = tx.send((path_for_send, loaded)).await;
            }
        }));
    }

    fn update_hover(&mut self, col: u16, row: u16) {
        if self.preview_area_x.is_some_and(|px| col >= px) {
            self.hover_row = None;
            return;
        }
        if row >= self.tree_area_y && row < self.tree_area_y + self.tree_area_height {
            let relative_row = (row - self.tree_area_y) as usize;
            if relative_row < self.tree.rendered_indices.len() {
                self.hover_row = Some(relative_row);
            } else {
                self.hover_row = None;
            }
        } else {
            self.hover_row = None;
        }
    }

    // ── Context menu ────────────────────────────────────────────────────

    fn open_context_menu(&mut self, col: u16, row: u16) {
        // Exclude preview pane and separator
        if self
            .preview_area_x
            .is_some_and(|px| col >= px.saturating_sub(1))
        {
            return;
        }
        if row < self.tree_area_y || row >= self.tree_area_y + self.tree_area_height {
            return;
        }
        let relative_row = (row - self.tree_area_y) as usize;
        let menu = if relative_row >= self.tree.rendered_indices.len() {
            // Empty space below tree items → workspace root menu
            ContextMenuState::new_for_workspace(col, row, self.tree.len())
        } else {
            let node_idx = self.tree.rendered_indices[relative_row];
            if node_idx >= self.tree.len() {
                return;
            }
            self.tree.cursor = node_idx;
            if self.tree.nodes[node_idx].is_dir() {
                ContextMenuState::new_for_dir(col, row, node_idx)
            } else {
                ContextMenuState::new_for_file(col, row, node_idx)
            }
        };

        self.context_menu = Some(menu);
        self.input_mode = InputMode::ContextMenu;
    }

    fn handle_context_menu_mouse(&mut self, mouse: crossterm::event::MouseEvent) -> PostAction {
        use crossterm::event::{MouseButton, MouseEventKind};

        match mouse.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                if let Some(ref menu) = self.context_menu {
                    let tw = self.main_area_width;
                    let th = self.tree_area_y + self.tree_area_height + 1;
                    if menu.contains(mouse.column, mouse.row, tw, th) {
                        if let Some(idx) = menu.row_to_item(mouse.row, tw, th) {
                            let menu_action = menu.items[idx].action.clone();
                            let node_idx = menu.node_idx;
                            self.context_menu = None;
                            self.input_mode = InputMode::Normal;
                            return self.execute_menu_action_sync(&menu_action, node_idx);
                        }
                    } else {
                        self.context_menu = None;
                        self.input_mode = InputMode::Normal;
                    }
                }
            }
            MouseEventKind::Moved => {
                if let Some(ref mut menu) = self.context_menu {
                    let tw = self.main_area_width;
                    let th = self.tree_area_y + self.tree_area_height + 1;
                    if let Some(idx) = menu.row_to_item(mouse.row, tw, th) {
                        menu.selected = idx;
                    }
                }
            }
            _ => {
                // Any other click closes the menu
                if matches!(mouse.kind, MouseEventKind::Down(_)) {
                    self.context_menu = None;
                    self.input_mode = InputMode::Normal;
                }
            }
        }
        PostAction::None
    }

    fn handle_picker_mouse(&mut self, mouse: crossterm::event::MouseEvent) -> PostAction {
        use crossterm::event::{MouseButton, MouseEventKind};

        let (cols, rows) = crossterm::terminal::size().unwrap_or((80, 24));
        let area = ratatui::layout::Rect::new(0, 0, cols, rows);

        let layout = match self.picker_state.as_ref().and_then(|p| p.layout(area)) {
            Some(l) => l,
            None => return PostAction::None,
        };

        let dialog = layout.dialog_rect;
        let inside = mouse.column >= dialog.x
            && mouse.column < dialog.x + dialog.width
            && mouse.row >= dialog.y
            && mouse.row < dialog.y + dialog.height;

        match mouse.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                if inside {
                    if let Some(picker) = self.picker_state.as_mut() {
                        if let Some(idx) = picker.row_to_filtered_idx(&layout, mouse.row) {
                            picker.selected = idx;
                            self.confirm_picker();
                        }
                    }
                } else {
                    // Click outside closes picker
                    self.picker_state = None;
                    self.input_mode = InputMode::Normal;
                }
            }
            MouseEventKind::Moved => {
                if inside {
                    if let Some(picker) = self.picker_state.as_mut() {
                        if let Some(idx) = picker.row_to_filtered_idx(&layout, mouse.row) {
                            picker.selected = idx;
                        }
                    }
                }
            }
            MouseEventKind::ScrollUp => {
                if inside {
                    if let Some(picker) = self.picker_state.as_mut() {
                        picker.move_up();
                    }
                }
            }
            MouseEventKind::ScrollDown => {
                if inside {
                    if let Some(picker) = self.picker_state.as_mut() {
                        picker.move_down();
                    }
                }
            }
            _ => {
                if matches!(mouse.kind, MouseEventKind::Down(_)) {
                    self.picker_state = None;
                    self.input_mode = InputMode::Normal;
                }
            }
        }
        PostAction::None
    }

    // SYNC: keep in sync with execute_menu_action_sync
    fn execute_menu_action(
        &mut self,
        action: &MenuAction,
        node_idx: usize,
        preview_tx: &mpsc::Sender<(PathBuf, LoadedPreview)>,
    ) -> PostAction {
        match action {
            MenuAction::OpenInEditor => {
                if let Some(node) = self.tree.nodes.get(node_idx) {
                    if !node.is_dir() {
                        return PostAction::OpenEditor(node.path.clone());
                    }
                }
            }
            MenuAction::CopyPath => {
                if let Some(node) = self.tree.nodes.get(node_idx) {
                    let rel = node
                        .path
                        .strip_prefix(&self.root)
                        .unwrap_or(&node.path)
                        .to_string_lossy()
                        .into_owned();
                    if let Ok(mut clipboard) = arboard::Clipboard::new() {
                        let _ = clipboard.set_text(rel);
                    }
                }
            }
            MenuAction::CopyAbsPath => {
                if let Some(node) = self.tree.nodes.get(node_idx) {
                    let abs = node.path.to_string_lossy().into_owned();
                    if let Ok(mut clipboard) = arboard::Clipboard::new() {
                        let _ = clipboard.set_text(abs);
                    }
                }
            }
            MenuAction::OpenExternally => {
                if let Some(node) = self.tree.nodes.get(node_idx) {
                    if !node.is_dir() {
                        self.open_externally(&node.path.clone());
                    }
                }
            }
            MenuAction::RevealInFinder => {
                if let Some(node) = self.tree.nodes.get(node_idx) {
                    let _ = std::process::Command::new("open")
                        .arg("-R")
                        .arg(&node.path)
                        .spawn();
                }
            }
            MenuAction::NewFile => self.start_new_file_at(node_idx),
            MenuAction::NewDir => self.start_new_dir_at(node_idx),
            MenuAction::Rename => self.start_rename_at(node_idx),
            MenuAction::Delete => self.start_delete_at(node_idx),
            MenuAction::TogglePreview => {
                self.preview_visible = !self.preview_visible;
                if self.preview_visible {
                    self.trigger_preview_load(preview_tx);
                } else {
                    self.focus = FocusPane::Tree;
                }
            }
            MenuAction::Refresh => {
                self.tree.refresh();
                if let Some(ref mut git) = self.git {
                    git.refresh();
                }
                self.reapply_git();
                self.refresh_search_state();
            }
            MenuAction::CollapseAll => {
                self.tree.collapse_all();
                self.reapply_git();
                self.refresh_search_state();
            }
            MenuAction::StartFind => {
                self.search_state = SearchState::new(SearchMode::Find);
                self.search_state.origin_cursor = self.tree.cursor;
                self.search_state.origin_scroll_offset = self.tree.scroll_offset;
                self.input_mode = InputMode::Search;
            }
        }

        // Refresh preview after menu actions that modify files
        if matches!(
            action,
            MenuAction::NewFile | MenuAction::NewDir | MenuAction::Rename | MenuAction::Delete
        ) {
            // Refresh handled in confirm_dialog
        } else if self.preview_visible {
            self.trigger_preview_load(preview_tx);
        }
        PostAction::None
    }

    // SYNC: keep in sync with execute_menu_action
    fn execute_menu_action_sync(&mut self, action: &MenuAction, node_idx: usize) -> PostAction {
        match action {
            MenuAction::OpenInEditor => {
                if let Some(node) = self.tree.nodes.get(node_idx) {
                    if !node.is_dir() {
                        return PostAction::OpenEditor(node.path.clone());
                    }
                }
            }
            MenuAction::CopyPath => {
                if let Some(node) = self.tree.nodes.get(node_idx) {
                    let rel = node
                        .path
                        .strip_prefix(&self.root)
                        .unwrap_or(&node.path)
                        .to_string_lossy()
                        .into_owned();
                    if let Ok(mut clipboard) = arboard::Clipboard::new() {
                        let _ = clipboard.set_text(rel);
                    }
                }
            }
            MenuAction::CopyAbsPath => {
                if let Some(node) = self.tree.nodes.get(node_idx) {
                    let abs = node.path.to_string_lossy().into_owned();
                    if let Ok(mut clipboard) = arboard::Clipboard::new() {
                        let _ = clipboard.set_text(abs);
                    }
                }
            }
            MenuAction::OpenExternally => {
                if let Some(node) = self.tree.nodes.get(node_idx) {
                    if !node.is_dir() {
                        self.open_externally(&node.path.clone());
                    }
                }
            }
            MenuAction::RevealInFinder => {
                if let Some(node) = self.tree.nodes.get(node_idx) {
                    let _ = std::process::Command::new("open")
                        .arg("-R")
                        .arg(&node.path)
                        .spawn();
                }
            }
            MenuAction::NewFile => self.start_new_file_at(node_idx),
            MenuAction::NewDir => self.start_new_dir_at(node_idx),
            MenuAction::Rename => self.start_rename_at(node_idx),
            MenuAction::Delete => self.start_delete_at(node_idx),
            MenuAction::TogglePreview => {
                self.preview_visible = !self.preview_visible;
                if !self.preview_visible {
                    self.focus = FocusPane::Tree;
                }
            }
            MenuAction::Refresh => {
                self.tree.refresh();
                if let Some(ref mut git) = self.git {
                    git.refresh();
                }
                self.reapply_git();
                self.refresh_search_state();
            }
            MenuAction::CollapseAll => {
                self.tree.collapse_all();
                self.reapply_git();
                self.refresh_search_state();
            }
            MenuAction::StartFind => {
                self.search_state = SearchState::new(SearchMode::Find);
                self.search_state.origin_cursor = self.tree.cursor;
                self.search_state.origin_scroll_offset = self.tree.scroll_offset;
                self.input_mode = InputMode::Search;
            }
        }
        PostAction::None
    }

    // ── Dialog mouse (R5: click outside dismisses) ───────────────────────

    fn handle_dialog_mouse(&mut self, mouse: crossterm::event::MouseEvent) -> PostAction {
        use crossterm::event::{MouseButton, MouseEventKind};

        if !matches!(mouse.kind, MouseEventKind::Down(MouseButton::Left)) {
            return PostAction::None;
        }

        // Check if click is outside the dialog area
        let (cols, rows) = crossterm::terminal::size().unwrap_or((80, 24));
        let area = ratatui::layout::Rect::new(0, 0, cols, rows);

        if let Some(ref dialog) = self.input_dialog {
            let dialog_width = 50u16.min(area.width.saturating_sub(4));
            let dialog_height = if matches!(dialog.kind, DialogKind::ConfirmDelete) {
                6
            } else {
                5
            };
            let dx = (area.width.saturating_sub(dialog_width)) / 2;
            let dy = (area.height.saturating_sub(dialog_height)) / 2;

            let inside = mouse.column >= dx
                && mouse.column < dx + dialog_width
                && mouse.row >= dy
                && mouse.row < dy + dialog_height;

            if inside {
                let (confirm_rect, cancel_rect) = dialog.button_positions(area);
                if mouse.column >= confirm_rect.x
                    && mouse.column < confirm_rect.x + confirm_rect.width
                    && mouse.row == confirm_rect.y
                {
                    self.confirm_dialog();
                    return PostAction::None;
                }
                if mouse.column >= cancel_rect.x
                    && mouse.column < cancel_rect.x + cancel_rect.width
                    && mouse.row == cancel_rect.y
                {
                    self.input_dialog = None;
                    self.input_mode = InputMode::Normal;
                    return PostAction::None;
                }
            } else {
                self.input_dialog = None;
                self.input_mode = InputMode::Normal;
            }
        }
        PostAction::None
    }

    // ── Status bar click ────────────────────────────────────────────────

    fn handle_status_bar_click(
        &mut self,
        col: u16,
        _preview_tx: &mpsc::Sender<(PathBuf, LoadedPreview)>,
    ) -> PostAction {
        // Check if click is on the branch name region
        if let Some((start, end)) = self.status_bar_branch_region {
            if col >= start && col < end && self.git.is_some() {
                self.open_branch_picker();
            }
        }
        PostAction::None
    }

    // ── Search bar click ────────────────────────────────────────────────

    fn handle_search_bar_click(
        &mut self,
        col: u16,
        _preview_tx: &mpsc::Sender<(PathBuf, LoadedPreview)>,
    ) -> PostAction {
        use crate::render::search_bar::SearchBar;

        // Check if click is on the [×] close button
        if let Some(search_y) = self.search_bar_y {
            let (_, rows) = crossterm::terminal::size().unwrap_or((80, 24));
            let area_width = self.main_area_width;
            let close_x = SearchBar::close_button_x(0, area_width);
            if col >= close_x {
                // Close button clicked → cancel search
                self.input_mode = InputMode::Normal;
                self.search_state.clear();
                let _ = (search_y, rows);
                return PostAction::None;
            }
        }

        // Click elsewhere on search bar → focus search (preserve query)
        self.input_mode = InputMode::Search;
        PostAction::None
    }

    // ── Open externally ─────────────────────────────────────────────────

    fn resolve_open_command(&self, path: &std::path::Path) -> String {
        let file_name = path
            .file_name()
            .map(|n| n.to_string_lossy())
            .unwrap_or_default();
        for rule in &self.config.open.rules {
            if let Ok(glob) = globset::Glob::new(&rule.pattern) {
                let matcher = glob.compile_matcher();
                if matcher.is_match(file_name.as_ref()) {
                    return rule.command.clone();
                }
            }
        }
        self.config.open.default.clone()
    }

    fn open_externally(&self, path: &std::path::Path) {
        let command_str = self.resolve_open_command(path);
        let parts = match shell_words::split(&command_str) {
            Ok(p) if !p.is_empty() => p,
            _ => return,
        };
        let (cmd, args) = parts.split_first().unwrap();
        let _ = std::process::Command::new(cmd)
            .args(args)
            .arg(path)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn();
    }

    // ── File operations ─────────────────────────────────────────────────

    fn start_new_file(&mut self) {
        let dir = self.current_dir();
        self.input_dialog = Some(InputDialogState::new(
            DialogKind::NewFile,
            dir,
            String::new(),
        ));
        self.input_mode = InputMode::Dialog;
    }

    fn start_new_dir(&mut self) {
        let dir = self.current_dir();
        self.input_dialog = Some(InputDialogState::new(
            DialogKind::NewDir,
            dir,
            String::new(),
        ));
        self.input_mode = InputMode::Dialog;
    }

    fn start_rename(&mut self) {
        if let Some(node) = self.tree.selected() {
            let name = node.name.clone();
            let path = node.path.clone();
            self.input_dialog = Some(InputDialogState::new(DialogKind::Rename, path, name));
            self.input_mode = InputMode::Dialog;
        }
    }

    fn start_delete(&mut self) {
        if let Some(node) = self.tree.selected() {
            let name = node.name.clone();
            let path = node.path.clone();
            self.input_dialog = Some(InputDialogState::new(DialogKind::ConfirmDelete, path, name));
            self.input_mode = InputMode::Dialog;
        }
    }

    fn start_new_file_at(&mut self, node_idx: usize) {
        let dir = self.dir_for_node(node_idx);
        self.input_dialog = Some(InputDialogState::new(
            DialogKind::NewFile,
            dir,
            String::new(),
        ));
        self.input_mode = InputMode::Dialog;
    }

    fn start_new_dir_at(&mut self, node_idx: usize) {
        let dir = self.dir_for_node(node_idx);
        self.input_dialog = Some(InputDialogState::new(
            DialogKind::NewDir,
            dir,
            String::new(),
        ));
        self.input_mode = InputMode::Dialog;
    }

    fn start_rename_at(&mut self, node_idx: usize) {
        if let Some(node) = self.tree.nodes.get(node_idx) {
            let name = node.name.clone();
            let path = node.path.clone();
            self.input_dialog = Some(InputDialogState::new(DialogKind::Rename, path, name));
            self.input_mode = InputMode::Dialog;
        }
    }

    fn start_delete_at(&mut self, node_idx: usize) {
        if let Some(node) = self.tree.nodes.get(node_idx) {
            let name = node.name.clone();
            let path = node.path.clone();
            self.input_dialog = Some(InputDialogState::new(DialogKind::ConfirmDelete, path, name));
            self.input_mode = InputMode::Dialog;
        }
    }

    fn confirm_dialog(&mut self) {
        let Some(dialog) = self.input_dialog.take() else {
            return;
        };
        self.input_mode = InputMode::Normal;

        match dialog.kind {
            DialogKind::NewFile => {
                if !dialog.input.is_empty() {
                    let new_path = dialog.context_path.join(&dialog.input);
                    // Create parent dirs if needed
                    if let Some(parent) = new_path.parent() {
                        let _ = std::fs::create_dir_all(parent);
                    }
                    let _ = std::fs::File::create(&new_path);
                }
            }
            DialogKind::NewDir => {
                if !dialog.input.is_empty() {
                    let new_path = dialog.context_path.join(&dialog.input);
                    let _ = std::fs::create_dir_all(&new_path);
                }
            }
            DialogKind::Rename => {
                if !dialog.input.is_empty() && dialog.input != dialog.target_name {
                    if let Some(parent) = dialog.context_path.parent() {
                        let new_path = parent.join(&dialog.input);
                        let _ = std::fs::rename(&dialog.context_path, &new_path);
                    }
                }
            }
            // R1: ConfirmDelete always deletes only the context_path node
            DialogKind::ConfirmDelete => {
                let path = &dialog.context_path;
                if path.is_dir() {
                    let _ = std::fs::remove_dir_all(path);
                } else {
                    let _ = std::fs::remove_file(path);
                }
            }
        }

        // Refresh tree after any file operation
        self.tree.refresh();
        if let Some(ref mut git) = self.git {
            git.refresh();
        }
        self.reapply_git();
    }

    /// Get the directory context for the currently selected node.
    fn current_dir(&self) -> PathBuf {
        if let Some(node) = self.tree.selected() {
            if node.is_dir() {
                node.path.clone()
            } else {
                node.path.parent().unwrap_or(&self.root).to_path_buf()
            }
        } else {
            self.root.clone()
        }
    }

    /// Get the directory for a given node (node itself if dir, or its parent).
    fn dir_for_node(&self, node_idx: usize) -> PathBuf {
        if let Some(node) = self.tree.nodes.get(node_idx) {
            if node.is_dir() {
                node.path.clone()
            } else {
                node.path.parent().unwrap_or(&self.root).to_path_buf()
            }
        } else {
            self.root.clone()
        }
    }

    // ── Search ───────────────────────────────────────────────────────────

    /// Re-compute search state after structural changes (expand/collapse/refresh).
    fn refresh_search_state(&mut self) {
        if self.search_state.query.is_empty() {
            self.search_state.match_indices.clear();
            self.search_state.visible_indices.clear();
            self.search_state.current_match = 0;
            return;
        }
        match self.search_state.mode {
            SearchMode::Find => self.update_find_matches(),
            SearchMode::Filter => self.update_filter_view(),
            SearchMode::Global => {} // global search doesn't depend on tree structure
        }
    }

    /// Find mode: compute `match_indices` (highlight only, no filtering).
    fn update_find_matches(&mut self) {
        // ALWAYS clear positions — node indices shift on expand/collapse
        self.search_state.match_char_positions.clear();

        if self.search_state.query.is_empty() {
            self.search_state.match_indices.clear();
            self.search_state.current_match = 0;
            return;
        }

        let (query, match_mode) = self.search_state.effective_query();
        let re = self.search_state.compiled_regex.take();
        let displayable = self.tree.build_displayable_indices();

        let mut matches = Vec::new();
        for idx in displayable {
            let display_name = self.tree.compact_display_name_for(idx);
            if let Some(positions) =
                do_match_positions(match_mode, &query, re.as_ref(), &display_name)
            {
                matches.push(idx);
                self.search_state
                    .match_char_positions
                    .insert(idx, positions);
            } else {
                // Fall back to path match (no character positions — renderer uses FullName)
                let rel_path = self.tree.nodes[idx]
                    .path
                    .strip_prefix(&self.root)
                    .unwrap_or(&self.tree.nodes[idx].path)
                    .to_string_lossy()
                    .into_owned();
                if do_match(match_mode, &query, re.as_ref(), &rel_path) {
                    matches.push(idx);
                }
            }
        }

        self.search_state.compiled_regex = re;
        self.search_state.match_indices = matches;

        // Jump cursor to the closest match to origin_cursor
        if self.search_state.match_indices.is_empty() {
            self.search_state.current_match = 0;
        } else {
            let origin = self.search_state.origin_cursor;
            #[allow(clippy::cast_possible_wrap)]
            let closest = self
                .search_state
                .match_indices
                .iter()
                .enumerate()
                .min_by_key(|(_, &idx)| (idx as isize - origin as isize).unsigned_abs())
                .map_or(0, |(i, _)| i);
            self.search_state.current_match = closest;
            self.tree.cursor = self.search_state.match_indices[closest];
        }
    }

    /// Filter mode: compute `match_indices` and `visible_indices` (matches + ancestors).
    fn update_filter_view(&mut self) {
        if self.search_state.query.is_empty() {
            self.search_state.match_indices.clear();
            self.search_state.visible_indices.clear();
            self.search_state.current_match = 0;
            return;
        }

        let (query, match_mode) = self.search_state.effective_query();
        let re = self.search_state.compiled_regex.take();
        let displayable = self.tree.build_displayable_indices();

        // Step 1: find matching displayable nodes
        let mut matches = Vec::new();
        for idx in displayable {
            let target_name = self.tree.compact_display_name_for(idx);
            let rel_path = self.tree.nodes[idx]
                .path
                .strip_prefix(&self.root)
                .unwrap_or(&self.tree.nodes[idx].path)
                .to_string_lossy()
                .into_owned();
            if do_match(match_mode, &query, re.as_ref(), &rel_path)
                || do_match(match_mode, &query, re.as_ref(), &target_name)
            {
                matches.push(idx);
            }
        }

        self.search_state.compiled_regex = re;

        // Step 2: collect ancestors for each match
        let mut visible_set = std::collections::HashSet::new();
        for &match_idx in &matches {
            visible_set.insert(match_idx);
            // Walk up the tree to find ancestors (nodes with decreasing depth)
            let match_depth = self.tree.nodes[match_idx].depth;
            if match_depth > 0 {
                let mut target_depth = match_depth - 1;
                for i in (0..match_idx).rev() {
                    if self.tree.nodes[i].depth == target_depth {
                        visible_set.insert(i);
                        if target_depth == 0 {
                            break;
                        }
                        target_depth -= 1;
                    }
                }
            }
        }

        // Step 3: sort visible set
        let mut visible: Vec<usize> = visible_set.into_iter().collect();
        visible.sort_unstable();

        self.search_state.match_indices = matches;
        self.search_state.visible_indices = visible;

        // Move cursor to first match if not already on one
        if !self.search_state.match_indices.is_empty()
            && !self.search_state.match_indices.contains(&self.tree.cursor)
        {
            self.tree.cursor = self.search_state.match_indices[0];
            self.search_state.current_match = 0;
        }
    }

    fn search_navigate_next(&mut self) {
        if self.search_state.match_indices.is_empty() {
            return;
        }
        let len = self.search_state.match_indices.len();
        let next = (self.search_state.current_match + 1) % len;
        self.search_state.current_match = next;
        self.tree.cursor = self.search_state.match_indices[next];
    }

    fn search_navigate_prev(&mut self) {
        if self.search_state.match_indices.is_empty() {
            return;
        }
        let len = self.search_state.match_indices.len();
        let prev = if self.search_state.current_match == 0 {
            len - 1
        } else {
            self.search_state.current_match - 1
        };
        self.search_state.current_match = prev;
        self.tree.cursor = self.search_state.match_indices[prev];
    }

    /// Spawn an async global search (fd or rg) with debounce.
    fn spawn_global_search(
        &mut self,
        search_tx: &mpsc::Sender<(u64, Vec<GlobalSearchResult>, Option<String>)>,
    ) {
        // Abort previous search
        if let Some(handle) = self.global_search_handle.take() {
            handle.abort();
        }

        if self.search_state.query.is_empty() {
            return;
        }

        self.search_state.request_id += 1;
        self.search_state.global_loading = true;
        let id = self.search_state.request_id;
        let query = self.search_state.query.clone();
        let search_type = self.search_state.global_search_type;
        let root = self.root.clone();
        let fd_cmd = self.config.search.fd_command.clone();
        let rg_cmd = self.config.search.rg_command.clone();
        let max_results = self.config.search.max_results;
        let tx = search_tx.clone();

        self.global_search_handle = Some(tokio::spawn(async move {
            // Debounce: wait 200ms before executing
            tokio::time::sleep(std::time::Duration::from_millis(200)).await;

            let output = match search_type {
                GlobalSearchType::FileName => {
                    tokio::process::Command::new(&fd_cmd)
                        .args(["--type", "f", "--color", "never", &query])
                        .current_dir(&root)
                        .output()
                        .await
                }
                GlobalSearchType::Content => {
                    tokio::process::Command::new(&rg_cmd)
                        .args([
                            "--line-number",
                            "--no-heading",
                            "--color",
                            "never",
                            "--max-count",
                            "1",
                            &query,
                        ])
                        .current_dir(&root)
                        .output()
                        .await
                }
            };

            match output {
                Ok(out) => {
                    let stdout = String::from_utf8_lossy(&out.stdout);
                    let mut results = Vec::new();

                    for line in stdout.lines().take(max_results) {
                        if line.is_empty() {
                            continue;
                        }
                        match search_type {
                            GlobalSearchType::FileName => {
                                let path = root.join(line);
                                results.push(GlobalSearchResult {
                                    path,
                                    display: line.to_string(),
                                    line: None,
                                    context: None,
                                });
                            }
                            GlobalSearchType::Content => {
                                // Format: path:line:content
                                let mut parts = line.splitn(3, ':');
                                let file = parts.next().unwrap_or("");
                                let line_num: Option<usize> =
                                    parts.next().and_then(|s| s.parse().ok());
                                let context = parts.next().map(std::string::ToString::to_string);
                                let path = root.join(file);
                                results.push(GlobalSearchResult {
                                    path,
                                    display: file.to_string(),
                                    line: line_num,
                                    context,
                                });
                            }
                        }
                    }

                    let error = if !out.status.success() && results.is_empty() {
                        let stderr = String::from_utf8_lossy(&out.stderr);
                        if stderr.contains("not found")
                            || stderr.contains("No such file")
                            || out.status.code() == Some(127)
                        {
                            let cmd_name = match search_type {
                                GlobalSearchType::FileName => &fd_cmd,
                                GlobalSearchType::Content => &rg_cmd,
                            };
                            Some(format!("{cmd_name} not found"))
                        } else {
                            None
                        }
                    } else {
                        None
                    };

                    let _ = tx.send((id, results, error)).await;
                }
                Err(e) => {
                    let cmd_name = match search_type {
                        GlobalSearchType::FileName => &fd_cmd,
                        GlobalSearchType::Content => &rg_cmd,
                    };
                    let _ = tx
                        .send((id, Vec::new(), Some(format!("{cmd_name}: {e}"))))
                        .await;
                }
            }
        }));
    }

    // ── Utility ─────────────────────────────────────────────────────────

    fn screen_to_content(
        &self,
        screen_col: u16,
        screen_row: u16,
    ) -> Option<crate::preview::state::ContentPos> {
        let pl = self.preview_layout?;
        layout::screen_to_content(pl, self.preview_state.scroll_offset, screen_col, screen_row)
    }

    fn reapply_git(&mut self) {
        if let Some(ref git) = self.git {
            git.apply_to_nodes(&mut self.tree.nodes);
        }
    }

    // ── Branch picker ──────────────────────────────────────────────────

    fn open_branch_picker(&mut self) {
        let Some(ref git) = self.git else { return };
        let branches = crate::git::branches::list_branches(git.repo_root());
        self.picker_state = Some(PickerState::new_branch(&branches));
        self.input_mode = InputMode::Picker;
    }

    fn confirm_picker(&mut self) {
        let Some(picker) = self.picker_state.take() else {
            return;
        };

        let Some(item) = picker.selected_item().cloned() else {
            self.input_mode = InputMode::Normal;
            return;
        };

        // Don't switch to the already-current branch
        if item.is_current {
            self.input_mode = InputMode::Normal;
            return;
        }

        let Some(ref git) = self.git else {
            self.input_mode = InputMode::Normal;
            return;
        };

        // For remote branches like "origin/feature", use --track to resolve
        // multi-remote ambiguity (e.g. both origin/foo and upstream/foo exist).
        let result = if item.is_remote {
            // Try to create a local tracking branch from the specific remote ref.
            // If the local branch already exists, git will error — that's fine,
            // the user can select the local branch directly instead.
            std::process::Command::new("git")
                .arg("-C")
                .arg(git.repo_root())
                .arg("switch")
                .arg("--track")
                .arg(&item.data)
                .output()
        } else {
            std::process::Command::new("git")
                .arg("-C")
                .arg(git.repo_root())
                .arg("switch")
                .arg(&item.data)
                .output()
        };

        match result {
            Ok(output) if output.status.success() => {
                self.input_mode = InputMode::Normal;
                // Refresh tree and git state
                self.tree.refresh();
                if let Some(ref mut git) = self.git {
                    git.refresh();
                }
                self.reapply_git();
            }
            Ok(output) => {
                // Show error from stderr
                let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
                let branches = crate::git::branches::list_branches(git.repo_root());
                let mut restored_picker = PickerState::new_branch(&branches);
                restored_picker.error_message = Some(stderr);
                self.picker_state = Some(restored_picker);
                // Stay in picker mode so user can Esc
            }
            Err(e) => {
                let branches = crate::git::branches::list_branches(git.repo_root());
                let mut restored_picker = PickerState::new_branch(&branches);
                restored_picker.error_message = Some(format!("git: {e}"));
                self.picker_state = Some(restored_picker);
            }
        }
    }

    // ── Editor ─────────────────────────────────────────────────────────

    /// Resolve the editor command: config → $VISUAL → $EDITOR → "vi".
    fn resolve_editor(&self) -> String {
        crate::config::resolve_editor(&self.config)
    }

    /// Suspend the terminal, spawn the editor, then resume.
    fn open_editor_suspend<B: ratatui::backend::Backend>(
        &self,
        terminal: &mut Terminal<B>,
        path: &std::path::Path,
    ) -> anyhow::Result<()>
    where
        B::Error: Send + Sync + 'static,
    {
        // Leave alternate screen
        let mut stdout = std::io::stdout();
        if self.enhanced_keyboard {
            let _ = crossterm::execute!(stdout, PopKeyboardEnhancementFlags);
        }
        if self.mouse_enabled {
            let _ = crossterm::execute!(stdout, LeaveAlternateScreen, DisableMouseCapture);
        } else {
            let _ = crossterm::execute!(stdout, LeaveAlternateScreen);
        }
        let _ = crossterm::terminal::disable_raw_mode();

        // Resolve editor and split into command + args (e.g. "code --wait")
        let editor_str = self.resolve_editor();
        let mut parts = editor_str.split_whitespace();
        let cmd = parts.next().unwrap_or("vi");
        let status = std::process::Command::new(cmd)
            .args(parts)
            .arg(path)
            .status();

        if let Err(e) = status {
            eprintln!("Failed to open editor '{editor_str}': {e}");
            std::thread::sleep(Duration::from_secs(2));
        }

        // Restore terminal
        let _ = crossterm::terminal::enable_raw_mode();
        let mut stdout = std::io::stdout();
        if self.mouse_enabled {
            let _ = crossterm::execute!(stdout, EnterAlternateScreen, EnableMouseCapture);
        } else {
            let _ = crossterm::execute!(stdout, EnterAlternateScreen);
        }
        if self.enhanced_keyboard {
            let _ = crossterm::execute!(
                stdout,
                PushKeyboardEnhancementFlags(KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES)
            );
        }
        terminal.clear()?;

        Ok(())
    }

    /// Handle mouse events while in GlobalSearch mode.
    /// Only left-clicks are meaningful; all other mouse events (moves, scrolls) are ignored.
    fn handle_global_search_mouse(&mut self, mouse: crossterm::event::MouseEvent) -> PostAction {
        use crossterm::event::{MouseButton, MouseEventKind};

        // Only respond to left-click; ignore hover, scroll, drag, etc.
        if !matches!(mouse.kind, MouseEventKind::Down(MouseButton::Left)) {
            return PostAction::None;
        }

        // Compute overlay rect (same formula as GlobalSearchOverlay::render)
        let (cols, rows) = crossterm::terminal::size().unwrap_or((80, 24));
        let area = ratatui::layout::Rect::new(0, 0, cols, rows);
        let width = (area.width * 3 / 5)
            .max(40)
            .min(area.width.saturating_sub(4));
        let height = (area.height * 3 / 5)
            .max(10)
            .min(area.height.saturating_sub(4));
        let x = area.x + (area.width.saturating_sub(width)) / 2;
        let y = area.y + (area.height.saturating_sub(height)) / 2;

        let inside = mouse.column >= x
            && mouse.column < x + width
            && mouse.row >= y
            && mouse.row < y + height;

        if !inside {
            // Click outside overlay → cancel
            self.search_state.clear();
            self.input_mode = InputMode::Normal;
            if let Some(handle) = self.global_search_handle.take() {
                handle.abort();
            }
            return PostAction::None;
        }

        // Click inside the results area → select + confirm that result
        let results_y = y + 3;
        let results_end_y = y + height.saturating_sub(2);
        if mouse.row >= results_y && mouse.row < results_end_y {
            let scroll = self.search_state.global_scroll_offset;
            let clicked_index = scroll + (mouse.row - results_y) as usize;
            if clicked_index < self.search_state.global_results.len() {
                self.search_state.global_selected = clicked_index;
                // Confirm the selection
                if let Some(result) = self
                    .search_state
                    .global_results
                    .get(self.search_state.global_selected)
                    .cloned()
                {
                    self.input_mode = InputMode::Normal;
                    self.search_state.clear();
                    let path = result.path;
                    self.tree.navigate_to_path(&path);
                    self.reapply_git();
                }
            }
        }

        // Click on input area or border → no-op
        PostAction::None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::render::search_bar::{GlobalSearchType, SearchMode, SearchState};
    use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};

    /// Helper to create a minimal App rooted in a temp directory.
    fn test_app() -> App {
        let dir = std::env::temp_dir().join("croot_test_app");
        let _ = std::fs::create_dir_all(&dir);
        App::new(dir, false, Config::default()).expect("test app creation")
    }

    #[test]
    fn test_global_search_mouse_move_does_not_cancel() {
        let mut app = test_app();
        // Enter GlobalSearch mode
        app.input_mode = InputMode::GlobalSearch;
        app.search_state = SearchState::new(SearchMode::Global);
        app.search_state.global_search_type = GlobalSearchType::FileName;

        // Simulate a mouse move event
        let mouse = MouseEvent {
            kind: MouseEventKind::Moved,
            column: 10,
            row: 10,
            modifiers: crossterm::event::KeyModifiers::NONE,
        };
        app.handle_global_search_mouse(mouse);

        // Mode should still be GlobalSearch
        assert_eq!(app.input_mode, InputMode::GlobalSearch);
    }

    #[test]
    fn test_global_search_scroll_does_not_cancel() {
        let mut app = test_app();
        app.input_mode = InputMode::GlobalSearch;
        app.search_state = SearchState::new(SearchMode::Global);

        let mouse = MouseEvent {
            kind: MouseEventKind::ScrollUp,
            column: 10,
            row: 10,
            modifiers: crossterm::event::KeyModifiers::NONE,
        };
        app.handle_global_search_mouse(mouse);

        assert_eq!(app.input_mode, InputMode::GlobalSearch);
    }

    #[test]
    fn test_global_search_click_outside_cancels() {
        let mut app = test_app();
        app.input_mode = InputMode::GlobalSearch;
        app.search_state = SearchState::new(SearchMode::Global);
        app.search_state.global_search_type = GlobalSearchType::FileName;

        // Click at (0, 0) — guaranteed outside the centered overlay
        let mouse = MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: 0,
            row: 0,
            modifiers: crossterm::event::KeyModifiers::NONE,
        };
        app.handle_global_search_mouse(mouse);

        assert_eq!(app.input_mode, InputMode::Normal);
    }
}
