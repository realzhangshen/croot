use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Actions that can be triggered by user input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    Quit,
    CursorUp,
    CursorDown,
    CursorLeft,
    CursorRight,
    Toggle,
    Open,
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
    None,
}

/// Map a keyboard event to an Action.
///
/// `preview_visible`: whether the preview panel is currently shown.
/// When preview is hidden, Tab still acts as Toggle for backward compat.
/// `preview_has_selection`: whether there is an active text selection in the preview.
pub fn handle_key(key: KeyEvent, preview_visible: bool, preview_has_selection: bool) -> Action {
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

        // y: copy selection if one exists (vim-style yank)
        KeyCode::Char('y') if preview_has_selection => Action::CopySelection,

        // Esc: clear selection if one exists
        KeyCode::Esc if preview_has_selection => Action::ClearSelection,

        // Quit
        KeyCode::Char('q') => Action::Quit,

        // Navigation
        KeyCode::Char('k') | KeyCode::Up => Action::CursorUp,
        KeyCode::Char('j') | KeyCode::Down => Action::CursorDown,
        KeyCode::Char('h') | KeyCode::Left => Action::CursorLeft,
        KeyCode::Char('l') | KeyCode::Right => Action::CursorRight,

        // Toggle expand/collapse
        KeyCode::Char(' ') => Action::Toggle,
        KeyCode::Enter => Action::Open,

        // Tab: switch focus when preview is visible, otherwise toggle
        KeyCode::Tab => {
            if preview_visible {
                Action::SwitchFocus
            } else {
                Action::Toggle
            }
        }

        // Preview toggle
        KeyCode::Char('p') => Action::TogglePreview,

        // Toggle Markdown render mode
        KeyCode::Char('m') => Action::ToggleRender,

        // Refresh
        KeyCode::Char('r') => Action::Refresh,

        // Page navigation
        KeyCode::PageUp => Action::ScrollUp(10),
        KeyCode::PageDown => Action::ScrollDown(10),
        KeyCode::Char('g') => Action::GotoTop,
        KeyCode::Char('G') => Action::GotoBottom,

        _ => Action::None,
    }
}
