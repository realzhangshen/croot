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
    None,
}

/// Map a keyboard event to an Action.
pub fn handle_key(key: KeyEvent) -> Action {
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
        KeyCode::Tab => Action::Toggle,

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
