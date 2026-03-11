use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

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
    /// Toggle multi-select on current node.
    ToggleSelect,
    /// Clear multi-selection.
    ClearSelect,
    /// Delete all selected nodes.
    DeleteSelected,
    /// Start search mode.
    StartSearch,
    /// Search input: typed a character.
    SearchChar(char),
    /// Search input: backspace.
    SearchBackspace,
    /// Search input: confirm search (keep filter, return to normal).
    SearchConfirm,
    /// Search input: cancel search (clear filter).
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
}

/// Map a keyboard event to an Action in Normal mode.
///
/// Pure mouse interaction mode: only Ctrl+C (quit/copy) and Esc (clear selection) are kept.
/// All other operations are available via toolbar, right-click context menu, and clickable UI.
pub fn handle_key(key: KeyEvent, _preview_visible: bool, preview_has_selection: bool) -> Action {
    match key.code {
        // Ctrl+C or Super+C (Command+C via Kitty keyboard protocol): copy or quit
        KeyCode::Char('c')
            if key.modifiers.contains(KeyModifiers::CONTROL)
                || key.modifiers.contains(KeyModifiers::SUPER) =>
        {
            if preview_has_selection {
                Action::CopySelection
            } else {
                Action::Quit
            }
        }

        // Esc: clear selection if one exists
        KeyCode::Esc if preview_has_selection => Action::ClearSelection,

        _ => Action::None,
    }
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
