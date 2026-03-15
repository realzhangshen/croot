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

/// Build the keybinding map using a two-phase algorithm:
/// 1. Actions with built-in defaults get their default key unless the user overrides or disables.
/// 2. Opt-in actions (no default) are added only if the user configures them.
///
/// User override semantics:
/// - `None` → use built-in default (if any)
/// - `Some("")` → disabled (no key bound)
/// - `Some("key")` → use that key
pub fn build_keybinding_map(config: &KeybindingsConfig) -> KeybindingMap {
    let mut map = KeybindingMap::new();

    // Phase 1: Actions with built-in defaults
    let defaults: &[(Action, &str, &Option<String>)] = &[
        (Action::CursorUp, "Up", &config.cursor_up),
        (Action::CursorDown, "Down", &config.cursor_down),
        (Action::CursorLeft, "Left", &config.cursor_left),
        (Action::CursorRight, "Right", &config.cursor_right),
        (Action::GotoTop, "Home", &config.goto_top),
        (Action::GotoBottom, "End", &config.goto_bottom),
        (Action::StartFind, "/", &config.search),
        (Action::StartFilter, "f", &config.filter),
        (Action::StartGlobalSearch, "s", &config.global_search),
        (
            Action::StartGlobalSearchContent,
            "S",
            &config.global_search_content,
        ),
        (Action::ToggleRender, "m", &config.toggle_render),
    ];

    for (action, default_key, user_override) in defaults {
        let key_str = match user_override {
            None => Some(*default_key),
            Some(s) if s.is_empty() => None,
            Some(s) => Some(s.as_str()),
        };
        if let Some(key_str) = key_str {
            if let Some((code, modifiers)) = parse_key(key_str) {
                let binding = KeyBinding { code, modifiers };
                if let Some(existing) = map.get(&binding) {
                    eprintln!(
                        "croot: warning: key '{key_str}' is bound to both {existing:?} and {action:?}; last wins"
                    );
                }
                map.insert(binding, action.clone());
            }
        }
    }

    // Phase 2: Opt-in actions (no built-in default)
    let opt_ins: &[(&Option<String>, Action)] = &[
        (&config.quit, Action::Quit),
        (&config.toggle, Action::Toggle),
        (&config.refresh, Action::Refresh),
        (&config.new_file, Action::NewFile),
        (&config.new_dir, Action::NewDir),
        (&config.rename, Action::RenameNode),
        (&config.delete, Action::DeleteNode),
        (&config.toggle_preview, Action::TogglePreview),
        (&config.open_in_editor, Action::OpenInEditor),
        (&config.open_externally, Action::OpenExternally),
        (&config.collapse_all, Action::CollapseAll),
        (&config.branch_picker, Action::OpenBranchPicker),
        (&config.enter, Action::EnterKey),
    ];

    for (opt, action) in opt_ins {
        if let Some(ref s) = opt {
            if let Some((code, modifiers)) = parse_key(s) {
                let binding = KeyBinding { code, modifiers };
                if let Some(existing) = map.get(&binding) {
                    eprintln!(
                        "croot: warning: key '{s}' is bound to both {existing:?} and {action:?}; last wins"
                    );
                }
                map.insert(binding, action.clone());
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

    // Hardcoded: Esc clears preview selection
    if key.code == KeyCode::Esc {
        if preview_has_selection {
            return Action::ClearSelection;
        }
        // Tree selection clearing is handled at the app layer
        // (handle_key doesn't know tree state, so Action::None + Esc
        //  is checked in handle_action)
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
/// Checks user keybindings first, then falls back to hard-coded defaults.
pub fn handle_key_menu(key: KeyEvent, keybindings: &KeybindingMap) -> Action {
    // Check user keybindings for navigation actions
    let binding = KeyBinding {
        code: key.code,
        modifiers: key.modifiers,
    };
    if let Some(action) = keybindings.get(&binding) {
        match action {
            Action::Quit => return Action::MenuClose,
            Action::CursorDown => return Action::MenuDown,
            Action::CursorUp => return Action::MenuUp,
            _ => {}
        }
    }
    // Hard-coded fallbacks
    match key.code {
        KeyCode::Esc => Action::MenuClose,
        KeyCode::Up => Action::MenuUp,
        KeyCode::Down => Action::MenuDown,
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

#[cfg(test)]
mod tests {
    use super::*;

    fn make_key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    // ── Bug 7: menu keybindings ──────────────────────────────────────

    #[test]
    fn handle_key_menu_uses_custom_quit_binding() {
        let mut map = KeybindingMap::new();
        // Map 'x' to Quit
        map.insert(
            KeyBinding {
                code: KeyCode::Char('x'),
                modifiers: KeyModifiers::NONE,
            },
            Action::Quit,
        );
        assert_eq!(
            handle_key_menu(make_key(KeyCode::Char('x')), &map),
            Action::MenuClose
        );
    }

    #[test]
    fn handle_key_menu_uses_custom_nav_bindings() {
        let mut map = KeybindingMap::new();
        map.insert(
            KeyBinding {
                code: KeyCode::Char('n'),
                modifiers: KeyModifiers::NONE,
            },
            Action::CursorDown,
        );
        map.insert(
            KeyBinding {
                code: KeyCode::Char('p'),
                modifiers: KeyModifiers::NONE,
            },
            Action::CursorUp,
        );
        assert_eq!(
            handle_key_menu(make_key(KeyCode::Char('n')), &map),
            Action::MenuDown
        );
        assert_eq!(
            handle_key_menu(make_key(KeyCode::Char('p')), &map),
            Action::MenuUp
        );
    }

    #[test]
    fn handle_key_menu_falls_back_to_defaults() {
        let map = KeybindingMap::new(); // empty
        assert_eq!(
            handle_key_menu(make_key(KeyCode::Esc), &map),
            Action::MenuClose
        );
        assert_eq!(handle_key_menu(make_key(KeyCode::Up), &map), Action::MenuUp);
        assert_eq!(
            handle_key_menu(make_key(KeyCode::Down), &map),
            Action::MenuDown
        );
        assert_eq!(
            handle_key_menu(make_key(KeyCode::Enter), &map),
            Action::MenuSelect(MenuAction::CopyPath)
        );
    }
}
