use std::collections::HashMap;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::config::{parse_key, KeybindingsConfig};
use crate::render::context_menu::MenuAction;

/// Actions that can be triggered by user input.
/// Many variants are only constructed at runtime via toolbar/context-menu/mouse — not from
/// keyboard shortcuts — so the compiler reports them as "never constructed". They are used.
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub enum Action {
    Quit,
    CursorUp,
    CursorDown,
    CursorLeft,
    CursorRight,
    Toggle,
    Refresh,
    ScrollUp(u16),
    ScrollDown(u16),
    GotoTop,
    GotoBottom,
    ClickRow(u16),
    TogglePreview,
    SwitchFocus,
    PreviewScrollUp(u16),
    PreviewScrollDown(u16),
    /// Begin a text selection at screen (col, row).
    SelectionStart(u16, u16),
    /// Extend a text selection to screen (col, row).
    SelectionUpdate(u16, u16),
    /// Copy the current selection to the system clipboard.
    CopySelection,
    /// Clear the current selection.
    ClearSelection,
    /// Toggle rendered/raw preview for Markdown files.
    ToggleRender,
    /// Begin dragging the separator between tree and preview panes.
    SeparatorDragStart,
    /// Mouse drag update at screen (col, row) — app routes based on drag state.
    DragUpdate(u16, u16),
    /// Mouse hover at screen (col, row) for tree row highlighting.
    Hover(u16, u16),
    /// Right-click context menu at screen (col, row).
    RightClick(u16, u16),
    /// Execute a context menu action.
    MenuSelect(MenuAction),
    /// Close the context menu.
    MenuClose,
    /// Navigate context menu up.
    MenuUp,
    /// Navigate context menu down.
    MenuDown,
    /// File operation: new file in current dir.
    NewFile,
    /// File operation: new directory in current dir.
    NewDir,
    /// File operation: rename current node.
    RenameNode,
    /// File operation: delete current node.
    DeleteNode,
    /// Dialog input: user typed a character.
    DialogChar(char),
    /// Dialog input: backspace.
    DialogBackspace,
    /// Dialog input: confirm.
    DialogConfirm,
    /// Dialog input: cancel.
    DialogCancel,
    /// Dialog input: move cursor left.
    DialogLeft,
    /// Dialog input: move cursor right.
    DialogRight,
    /// Start Find mode (jump to match, no filtering).
    StartFind,
    /// Start Filter mode (reduce tree to matches + ancestors).
    StartFilter,
    /// Search input: typed a character.
    SearchChar(char),
    /// Search input: backspace.
    SearchBackspace,
    /// Search input: confirm search.
    SearchConfirm,
    /// Search input: cancel search.
    SearchCancel,
    /// Search input: move cursor.
    SearchLeft,
    SearchRight,
    /// Navigate to next/prev match in search results.
    SearchNext,
    SearchPrev,
    /// Open the selected file in $EDITOR.
    OpenInEditor,
    /// Open the selected file externally (fire-and-forget).
    OpenExternally,
    /// Collapse all expanded directories.
    CollapseAll,
    /// Focus the search bar without clearing the existing query.
    FocusSearch,
    /// Double-click on a tree row.
    DoubleClick(u16),
    /// Enter key: open file in editor or toggle directory.
    EnterKey,
    /// Open the branch picker overlay.
    OpenBranchPicker,
    /// Picker input: user typed a character.
    PickerChar(char),
    /// Picker input: backspace.
    PickerBackspace,
    /// Picker input: confirm selection.
    PickerConfirm,
    /// Picker input: cancel.
    PickerCancel,
    /// Picker navigation: move up.
    PickerUp,
    /// Picker navigation: move down.
    PickerDown,
    /// Start global search for file names (fd).
    StartGlobalSearch,
    /// Start global search for file contents (rg).
    StartGlobalSearchContent,
    /// Global search input: typed a character.
    GlobalSearchChar(char),
    /// Global search input: backspace.
    GlobalSearchBackspace,
    /// Global search: confirm selection (navigate to result).
    GlobalSearchConfirm,
    /// Global search: cancel.
    GlobalSearchCancel,
    /// Global search: move selection up.
    GlobalSearchUp,
    /// Global search: move selection down.
    GlobalSearchDown,
    None,
}

/// App input mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    Normal,
    ContextMenu,
    Dialog,
    Search,
    Picker,
    GlobalSearch,
}

/// A key binding: a key code plus modifiers.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct KeyBinding {
    pub code: KeyCode,
    pub modifiers: KeyModifiers,
}

/// Maps key bindings to actions.
pub type KeybindingMap = HashMap<KeyBinding, Action>;

/// Build the keybinding map from user config. Only keys the user has set will be active.
pub fn build_keybinding_map(config: &KeybindingsConfig) -> KeybindingMap {
    let mut map = KeybindingMap::new();

    let entries: &[(&Option<String>, Action)] = &[
        (&config.quit, Action::Quit),
        (&config.cursor_up, Action::CursorUp),
        (&config.cursor_down, Action::CursorDown),
        (&config.cursor_left, Action::CursorLeft),
        (&config.cursor_right, Action::CursorRight),
        (&config.toggle, Action::Toggle),
        (&config.refresh, Action::Refresh),
        (&config.new_file, Action::NewFile),
        (&config.new_dir, Action::NewDir),
        (&config.rename, Action::RenameNode),
        (&config.delete, Action::DeleteNode),
        (&config.toggle_preview, Action::TogglePreview),
        (&config.toggle_render, Action::ToggleRender),
        (&config.open_in_editor, Action::OpenInEditor),
        (&config.open_externally, Action::OpenExternally),
        (&config.collapse_all, Action::CollapseAll),
        (&config.search, Action::StartFind),
        (&config.filter, Action::StartFilter),
        (&config.goto_top, Action::GotoTop),
        (&config.goto_bottom, Action::GotoBottom),
        (&config.branch_picker, Action::OpenBranchPicker),
        (&config.enter, Action::EnterKey),
        (&config.global_search, Action::StartGlobalSearch),
        (
            &config.global_search_content,
            Action::StartGlobalSearchContent,
        ),
    ];

    for (opt, action) in entries {
        if let Some(ref s) = opt {
            if let Some((code, modifiers)) = parse_key(s) {
                map.insert(KeyBinding { code, modifiers }, action.clone());
            }
        }
    }

    map
}

/// Map a keyboard event to an Action in Normal mode.
///
/// Hardcoded: Ctrl+C (quit/copy) and Esc (clear selection).
/// All other shortcuts come from the user-configured keybinding map.
pub fn handle_key(
    key: KeyEvent,
    _preview_visible: bool,
    preview_has_selection: bool,
    keybindings: &KeybindingMap,
) -> Action {
    // Hardcoded: Ctrl+C / Super+C (copy or quit)
    if matches!(key.code, KeyCode::Char('c'))
        && (key.modifiers.contains(KeyModifiers::CONTROL)
            || key.modifiers.contains(KeyModifiers::SUPER))
    {
        return if preview_has_selection {
            Action::CopySelection
        } else {
            Action::Quit
        };
    }

    // Hardcoded: Esc clears selection
    if key.code == KeyCode::Esc && preview_has_selection {
        return Action::ClearSelection;
    }

    // Look up user-configured keybindings
    // Try exact modifiers first
    let binding = KeyBinding {
        code: key.code,
        modifiers: key.modifiers,
    };
    if let Some(action) = keybindings.get(&binding) {
        return action.clone();
    }

    // For uppercase chars, terminals may send Char('A') with SHIFT modifier (Kitty protocol)
    // or Char('A') without SHIFT (legacy). Normalize by stripping SHIFT for letter chars.
    if let KeyCode::Char(ch) = key.code {
        if ch.is_ascii_uppercase() && key.modifiers.contains(KeyModifiers::SHIFT) {
            let stripped = KeyBinding {
                code: KeyCode::Char(ch.to_ascii_lowercase()),
                modifiers: key.modifiers - KeyModifiers::SHIFT,
            };
            if let Some(action) = keybindings.get(&stripped) {
                return action.clone();
            }
        }
    }

    Action::None
}

/// Map a keyboard event in context menu mode.
pub fn handle_key_menu(key: KeyEvent) -> Action {
    match key.code {
        KeyCode::Esc | KeyCode::Char('q') => Action::MenuClose,
        KeyCode::Up | KeyCode::Char('k') => Action::MenuUp,
        KeyCode::Down | KeyCode::Char('j') => Action::MenuDown,
        KeyCode::Enter => Action::MenuSelect(MenuAction::CopyPath), // placeholder, app resolves
        _ => Action::None,
    }
}

/// Map a keyboard event in search mode.
pub fn handle_key_search(key: KeyEvent) -> Action {
    match key.code {
        KeyCode::Esc => Action::SearchCancel,
        KeyCode::Enter => Action::SearchConfirm,
        KeyCode::Backspace => Action::SearchBackspace,
        KeyCode::Left => Action::SearchLeft,
        KeyCode::Right => Action::SearchRight,
        KeyCode::Tab | KeyCode::Down => Action::SearchNext,
        KeyCode::BackTab | KeyCode::Up => Action::SearchPrev,
        KeyCode::Char(c) => Action::SearchChar(c),
        _ => Action::None,
    }
}

/// Map a keyboard event in picker mode.
pub fn handle_key_picker(key: KeyEvent) -> Action {
    match key.code {
        KeyCode::Esc => Action::PickerCancel,
        KeyCode::Enter => Action::PickerConfirm,
        KeyCode::Backspace => Action::PickerBackspace,
        KeyCode::Up | KeyCode::BackTab => Action::PickerUp,
        KeyCode::Down | KeyCode::Tab => Action::PickerDown,
        KeyCode::Char(c) => Action::PickerChar(c),
        _ => Action::None,
    }
}

/// Map a keyboard event in dialog mode.
pub fn handle_key_dialog(key: KeyEvent) -> Action {
    match key.code {
        KeyCode::Esc => Action::DialogCancel,
        KeyCode::Enter => Action::DialogConfirm,
        KeyCode::Backspace => Action::DialogBackspace,
        KeyCode::Left => Action::DialogLeft,
        KeyCode::Right => Action::DialogRight,
        KeyCode::Char(c) => Action::DialogChar(c),
        _ => Action::None,
    }
}

/// Map a keyboard event in global search mode.
pub fn handle_key_global_search(key: KeyEvent) -> Action {
    match key.code {
        KeyCode::Esc => Action::GlobalSearchCancel,
        KeyCode::Enter => Action::GlobalSearchConfirm,
        KeyCode::Backspace => Action::GlobalSearchBackspace,
        KeyCode::Up | KeyCode::BackTab => Action::GlobalSearchUp,
        KeyCode::Down | KeyCode::Tab => Action::GlobalSearchDown,
        KeyCode::Char(c) => Action::GlobalSearchChar(c),
        _ => Action::None,
    }
}
