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
    FocusPreview,
    PreviewScrollUp(u16),
    PreviewScrollDown(u16),
    None,
}

/// Map a keyboard event to an Action.
///
/// `preview_visible`: whether the preview panel is currently shown.
/// When preview is hidden, Tab still acts as Toggle for backward compat.
pub fn handle_key(key: KeyEvent, preview_visible: bool) -> Action {
    match key.code {
        // Quit
        KeyCode::Char('q') => Action::Quit,
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => Action::Quit,

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
