use std::collections::HashMap;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::config::{parse_key, KeybindingsConfig};
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
    /// Mouse button released — clears drag state.
    DragEnd,
    /// Mouse hover at screen (col, row) for tree row highlighting.
    Hover(u16, u16),
    /// Right-click context menu at screen (col, row).
    RightClick(u16, u16),
    /// Execute a context menu action.
    MenuSelect,
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
    /// Global search: confirm selection (open in editor).
    GlobalSearchConfirm,
    /// Global search: navigate to file in tree without opening editor.
    GlobalSearchGoto,
    /// Global search: cancel.
    GlobalSearchCancel,
    /// Global search: move selection up.
    GlobalSearchUp,
    /// Global search: move selection down.
    GlobalSearchDown,
    /// Bracketed paste: complete pasted text (control chars already stripped).
    Paste(String),
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
    // (intentionally not collapsed — Esc without selection falls through to keybinding lookup)
    #[allow(clippy::collapsible_if)]
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

    // Uppercase char = Shift was pressed. Normalize terminal variations:
    // - Kitty may send Char('S') + SHIFT
    // - Legacy may send Char('S') + NONE (no SHIFT flag)
    // Canonicalize to lowercase and try Shift binding first, then non-Shift fallback.
    if let KeyCode::Char(ch) = key.code {
        if ch.is_ascii_uppercase() {
            let lower = ch.to_ascii_lowercase();
            let other_mods = key.modifiers - KeyModifiers::SHIFT;

            // Try Shift binding first (e.g., "S" is stored as Char('s') + SHIFT)
            let with_shift = KeyBinding {
                code: KeyCode::Char(lower),
                modifiers: other_mods | KeyModifiers::SHIFT,
            };
            if let Some(action) = keybindings.get(&with_shift) {
                return action.clone();
            }

            // Fall back to non-Shift binding (e.g., only "s" is bound)
            let without_shift = KeyBinding {
                code: KeyCode::Char(lower),
                modifiers: other_mods,
            };
            if let Some(action) = keybindings.get(&without_shift) {
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
        KeyCode::Enter => Action::MenuSelect,
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
        KeyCode::Tab => Action::GlobalSearchGoto,
        KeyCode::Backspace => Action::GlobalSearchBackspace,
        KeyCode::Up | KeyCode::BackTab => Action::GlobalSearchUp,
        KeyCode::Down => Action::GlobalSearchDown,
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
            Action::MenuSelect
        );
    }

    // ── Normal mode: handle_key ─────────────────────────────────────────

    fn make_key_mod(code: KeyCode, modifiers: KeyModifiers) -> KeyEvent {
        KeyEvent::new(code, modifiers)
    }

    #[test]
    fn ctrl_c_quits_without_selection() {
        let map = KeybindingMap::new();
        let action = handle_key(
            make_key_mod(KeyCode::Char('c'), KeyModifiers::CONTROL),
            false,
            false, // no selection
            &map,
        );
        assert_eq!(action, Action::Quit);
    }

    #[test]
    fn ctrl_c_copies_with_active_selection() {
        let map = KeybindingMap::new();
        let action = handle_key(
            make_key_mod(KeyCode::Char('c'), KeyModifiers::CONTROL),
            true,
            true, // has selection
            &map,
        );
        assert_eq!(action, Action::CopySelection);
    }

    #[test]
    fn super_c_copies_with_active_selection() {
        let map = KeybindingMap::new();
        let action = handle_key(
            make_key_mod(KeyCode::Char('c'), KeyModifiers::SUPER),
            true,
            true,
            &map,
        );
        assert_eq!(action, Action::CopySelection);
    }

    #[test]
    fn esc_clears_selection_when_active() {
        let map = KeybindingMap::new();
        let action = handle_key(make_key(KeyCode::Esc), true, true, &map);
        assert_eq!(action, Action::ClearSelection);
    }

    #[test]
    fn esc_returns_none_without_selection() {
        let map = KeybindingMap::new();
        let action = handle_key(make_key(KeyCode::Esc), true, false, &map);
        assert_eq!(action, Action::None);
    }

    #[test]
    fn keybinding_map_lookup_works() {
        let mut map = KeybindingMap::new();
        map.insert(
            KeyBinding {
                code: KeyCode::Char('j'),
                modifiers: KeyModifiers::NONE,
            },
            Action::CursorDown,
        );
        let action = handle_key(make_key(KeyCode::Char('j')), false, false, &map);
        assert_eq!(action, Action::CursorDown);
    }

    #[test]
    fn uppercase_normalization_prefers_shift_binding() {
        // Map has both: Char('s') + NONE → Search, Char('s') + SHIFT → SearchContent
        // (this is how parse_key("s") and parse_key("S") store them)
        let mut map = KeybindingMap::new();
        map.insert(
            KeyBinding {
                code: KeyCode::Char('s'),
                modifiers: KeyModifiers::NONE,
            },
            Action::StartGlobalSearch,
        );
        map.insert(
            KeyBinding {
                code: KeyCode::Char('s'),
                modifiers: KeyModifiers::SHIFT,
            },
            Action::StartGlobalSearchContent,
        );

        // Some terminals: Char('S') + SHIFT → should match Char('s') + SHIFT → content
        let action = handle_key(
            make_key_mod(KeyCode::Char('S'), KeyModifiers::SHIFT),
            false,
            false,
            &map,
        );
        assert_eq!(action, Action::StartGlobalSearchContent);

        // Legacy terminal: Char('S') + NONE → should also match Char('s') + SHIFT → content
        let action = handle_key(make_key(KeyCode::Char('S')), false, false, &map);
        assert_eq!(action, Action::StartGlobalSearchContent);
    }

    #[test]
    fn uppercase_normalization_falls_back_to_lowercase_binding() {
        // Only lowercase 's' bound, no uppercase 'S' binding
        let mut map = KeybindingMap::new();
        map.insert(
            KeyBinding {
                code: KeyCode::Char('s'),
                modifiers: KeyModifiers::NONE,
            },
            Action::StartGlobalSearch,
        );

        // Char('S') + SHIFT → no Char('s') + SHIFT binding, falls back to Char('s') + NONE
        let action = handle_key(
            make_key_mod(KeyCode::Char('S'), KeyModifiers::SHIFT),
            false,
            false,
            &map,
        );
        assert_eq!(action, Action::StartGlobalSearch);

        // Char('S') + NONE → same fallback
        let action = handle_key(make_key(KeyCode::Char('S')), false, false, &map);
        assert_eq!(action, Action::StartGlobalSearch);
    }

    #[test]
    fn uppercase_normalization_via_default_config() {
        // End-to-end: build_keybinding_map with default config, then handle_key
        let config = KeybindingsConfig::default();
        let map = build_keybinding_map(&config);

        // Char('S') + SHIFT → should reach StartGlobalSearchContent
        let action = handle_key(
            make_key_mod(KeyCode::Char('S'), KeyModifiers::SHIFT),
            false,
            false,
            &map,
        );
        assert_eq!(action, Action::StartGlobalSearchContent);

        // Char('S') + NONE (legacy terminal) → should also reach StartGlobalSearchContent
        let action = handle_key(make_key(KeyCode::Char('S')), false, false, &map);
        assert_eq!(action, Action::StartGlobalSearchContent);
    }

    #[test]
    fn unbound_key_returns_none() {
        let map = KeybindingMap::new();
        let action = handle_key(make_key(KeyCode::Char('z')), false, false, &map);
        assert_eq!(action, Action::None);
    }

    // ── Search mode ─────────────────────────────────────────────────────

    #[test]
    fn search_esc_cancels() {
        assert_eq!(
            handle_key_search(make_key(KeyCode::Esc)),
            Action::SearchCancel
        );
    }

    #[test]
    fn search_enter_confirms() {
        assert_eq!(
            handle_key_search(make_key(KeyCode::Enter)),
            Action::SearchConfirm
        );
    }

    #[test]
    fn search_tab_goes_next() {
        assert_eq!(
            handle_key_search(make_key(KeyCode::Tab)),
            Action::SearchNext
        );
    }

    #[test]
    fn search_backtab_goes_prev() {
        assert_eq!(
            handle_key_search(make_key(KeyCode::BackTab)),
            Action::SearchPrev
        );
    }

    #[test]
    fn search_char_input() {
        assert_eq!(
            handle_key_search(make_key(KeyCode::Char('a'))),
            Action::SearchChar('a')
        );
    }

    #[test]
    fn search_backspace() {
        assert_eq!(
            handle_key_search(make_key(KeyCode::Backspace)),
            Action::SearchBackspace
        );
    }

    #[test]
    fn search_cursor_movement() {
        assert_eq!(
            handle_key_search(make_key(KeyCode::Left)),
            Action::SearchLeft
        );
        assert_eq!(
            handle_key_search(make_key(KeyCode::Right)),
            Action::SearchRight
        );
    }

    // ── Dialog mode ─────────────────────────────────────────────────────

    #[test]
    fn dialog_esc_cancels() {
        assert_eq!(
            handle_key_dialog(make_key(KeyCode::Esc)),
            Action::DialogCancel
        );
    }

    #[test]
    fn dialog_enter_confirms() {
        assert_eq!(
            handle_key_dialog(make_key(KeyCode::Enter)),
            Action::DialogConfirm
        );
    }

    #[test]
    fn dialog_char_input() {
        assert_eq!(
            handle_key_dialog(make_key(KeyCode::Char('x'))),
            Action::DialogChar('x')
        );
    }

    #[test]
    fn dialog_cursor_movement() {
        assert_eq!(
            handle_key_dialog(make_key(KeyCode::Left)),
            Action::DialogLeft
        );
        assert_eq!(
            handle_key_dialog(make_key(KeyCode::Right)),
            Action::DialogRight
        );
    }

    // ── Picker mode ─────────────────────────────────────────────────────

    #[test]
    fn picker_navigation() {
        assert_eq!(handle_key_picker(make_key(KeyCode::Up)), Action::PickerUp);
        assert_eq!(
            handle_key_picker(make_key(KeyCode::Down)),
            Action::PickerDown
        );
        assert_eq!(
            handle_key_picker(make_key(KeyCode::Tab)),
            Action::PickerDown
        );
        assert_eq!(
            handle_key_picker(make_key(KeyCode::BackTab)),
            Action::PickerUp
        );
    }

    #[test]
    fn picker_confirm_cancel() {
        assert_eq!(
            handle_key_picker(make_key(KeyCode::Enter)),
            Action::PickerConfirm
        );
        assert_eq!(
            handle_key_picker(make_key(KeyCode::Esc)),
            Action::PickerCancel
        );
    }

    // ── Global search mode ──────────────────────────────────────────────

    #[test]
    fn global_search_navigation() {
        assert_eq!(
            handle_key_global_search(make_key(KeyCode::Up)),
            Action::GlobalSearchUp
        );
        assert_eq!(
            handle_key_global_search(make_key(KeyCode::Down)),
            Action::GlobalSearchDown
        );
    }

    #[test]
    fn global_search_confirm_cancel() {
        assert_eq!(
            handle_key_global_search(make_key(KeyCode::Enter)),
            Action::GlobalSearchConfirm
        );
        assert_eq!(
            handle_key_global_search(make_key(KeyCode::Esc)),
            Action::GlobalSearchCancel
        );
    }

    #[test]
    fn global_search_char_input() {
        assert_eq!(
            handle_key_global_search(make_key(KeyCode::Char('t'))),
            Action::GlobalSearchChar('t')
        );
    }

    // ── build_keybinding_map ────────────────────────────────────────────

    #[test]
    fn default_keybindings_include_arrows_and_search() {
        let config = KeybindingsConfig::default();
        let map = build_keybinding_map(&config);
        // Arrow keys should be mapped by default
        assert_eq!(
            map.get(&KeyBinding {
                code: KeyCode::Up,
                modifiers: KeyModifiers::NONE,
            }),
            Some(&Action::CursorUp)
        );
        assert_eq!(
            map.get(&KeyBinding {
                code: KeyCode::Down,
                modifiers: KeyModifiers::NONE,
            }),
            Some(&Action::CursorDown)
        );
        // "/" should start find
        assert_eq!(
            map.get(&KeyBinding {
                code: KeyCode::Char('/'),
                modifiers: KeyModifiers::NONE,
            }),
            Some(&Action::StartFind)
        );
    }

    #[test]
    fn user_override_replaces_default() {
        let config = KeybindingsConfig {
            search: Some("?".to_string()), // override "/" with "?"
            ..Default::default()
        };
        let map = build_keybinding_map(&config);
        // "/" should no longer be bound
        assert_eq!(
            map.get(&KeyBinding {
                code: KeyCode::Char('/'),
                modifiers: KeyModifiers::NONE,
            }),
            None
        );
        // "?" should now start find
        assert_eq!(
            map.get(&KeyBinding {
                code: KeyCode::Char('?'),
                modifiers: KeyModifiers::NONE,
            }),
            Some(&Action::StartFind)
        );
    }

    #[test]
    fn empty_string_disables_binding() {
        let config = KeybindingsConfig {
            search: Some(String::new()), // disable search key
            ..Default::default()
        };
        let map = build_keybinding_map(&config);
        assert_eq!(
            map.get(&KeyBinding {
                code: KeyCode::Char('/'),
                modifiers: KeyModifiers::NONE,
            }),
            None
        );
    }

    #[test]
    fn opt_in_binding_added_when_configured() {
        let config = KeybindingsConfig {
            quit: Some("q".to_string()),
            ..Default::default()
        };
        let map = build_keybinding_map(&config);
        assert_eq!(
            map.get(&KeyBinding {
                code: KeyCode::Char('q'),
                modifiers: KeyModifiers::NONE,
            }),
            Some(&Action::Quit)
        );
    }

    // ── Modifier key handling ────────────────────────────────────────────

    #[test]
    fn ctrl_modifier_keybinding() {
        let config = KeybindingsConfig {
            quit: Some("Ctrl+q".to_string()),
            ..Default::default()
        };
        let map = build_keybinding_map(&config);
        assert_eq!(
            map.get(&KeyBinding {
                code: KeyCode::Char('q'),
                modifiers: KeyModifiers::CONTROL,
            }),
            Some(&Action::Quit)
        );
    }

    #[test]
    fn alt_modifier_keybinding() {
        let config = KeybindingsConfig {
            toggle_preview: Some("Alt+p".to_string()),
            ..Default::default()
        };
        let map = build_keybinding_map(&config);
        assert_eq!(
            map.get(&KeyBinding {
                code: KeyCode::Char('p'),
                modifiers: KeyModifiers::ALT,
            }),
            Some(&Action::TogglePreview)
        );
    }

    #[test]
    fn function_key_binding() {
        let config = KeybindingsConfig {
            refresh: Some("F5".to_string()),
            ..Default::default()
        };
        let map = build_keybinding_map(&config);
        assert_eq!(
            map.get(&KeyBinding {
                code: KeyCode::F(5),
                modifiers: KeyModifiers::NONE,
            }),
            Some(&Action::Refresh)
        );
    }

    // ── Conflict resolution ──────────────────────────────────────────────

    #[test]
    fn last_binding_wins_on_conflict() {
        let config = KeybindingsConfig {
            refresh: Some("r".to_string()),
            rename: Some("r".to_string()), // same key — last in opt-in list wins
            ..Default::default()
        };
        let map = build_keybinding_map(&config);
        let binding = KeyBinding {
            code: KeyCode::Char('r'),
            modifiers: KeyModifiers::NONE,
        };
        // Last in opt-in list wins (rename comes after refresh)
        assert_eq!(map.get(&binding), Some(&Action::RenameNode));
    }

    // ── Multiple opt-in bindings ─────────────────────────────────────────

    #[test]
    fn multiple_opt_in_bindings_coexist() {
        let config = KeybindingsConfig {
            new_file: Some("a".to_string()),
            new_dir: Some("A".to_string()),
            delete: Some("D".to_string()),
            ..Default::default()
        };
        let map = build_keybinding_map(&config);
        assert_eq!(
            map.get(&KeyBinding {
                code: KeyCode::Char('a'),
                modifiers: KeyModifiers::NONE,
            }),
            Some(&Action::NewFile)
        );
        // "A" parses to Char('a') + SHIFT
        assert_eq!(
            map.get(&KeyBinding {
                code: KeyCode::Char('a'),
                modifiers: KeyModifiers::SHIFT,
            }),
            Some(&Action::NewDir)
        );
        assert_eq!(
            map.get(&KeyBinding {
                code: KeyCode::Char('d'),
                modifiers: KeyModifiers::SHIFT,
            }),
            Some(&Action::DeleteNode)
        );
    }

    // ── handle_key with special keys ─────────────────────────────────────

    #[test]
    fn handle_key_enter_binding() {
        let config = KeybindingsConfig {
            enter: Some("Enter".to_string()),
            ..Default::default()
        };
        let map = build_keybinding_map(&config);
        let action = handle_key(make_key(KeyCode::Enter), false, false, &map);
        assert_eq!(action, Action::EnterKey);
    }

    #[test]
    fn handle_key_space_binding() {
        let config = KeybindingsConfig {
            toggle: Some("Space".to_string()),
            ..Default::default()
        };
        let map = build_keybinding_map(&config);
        let action = handle_key(make_key(KeyCode::Char(' ')), false, false, &map);
        assert_eq!(action, Action::Toggle);
    }

    // ── Global search mode key mapping ───────────────────────────────────

    #[test]
    fn global_search_backspace() {
        assert_eq!(
            handle_key_global_search(make_key(KeyCode::Backspace)),
            Action::GlobalSearchBackspace
        );
    }

    #[test]
    fn global_search_tab_goes_to_file() {
        assert_eq!(
            handle_key_global_search(make_key(KeyCode::Tab)),
            Action::GlobalSearchGoto
        );
    }

    #[test]
    fn global_search_backtab_goes_up() {
        assert_eq!(
            handle_key_global_search(make_key(KeyCode::BackTab)),
            Action::GlobalSearchUp
        );
    }

    #[test]
    fn global_search_unknown_key_returns_none() {
        assert_eq!(
            handle_key_global_search(make_key(KeyCode::F(12))),
            Action::None
        );
    }

    // ── Dialog mode completeness ─────────────────────────────────────────

    #[test]
    fn dialog_backspace() {
        assert_eq!(
            handle_key_dialog(make_key(KeyCode::Backspace)),
            Action::DialogBackspace
        );
    }

    #[test]
    fn dialog_unknown_key_returns_none() {
        assert_eq!(handle_key_dialog(make_key(KeyCode::F(12))), Action::None);
    }

    // ── Picker mode completeness ─────────────────────────────────────────

    #[test]
    fn picker_char_input() {
        assert_eq!(
            handle_key_picker(make_key(KeyCode::Char('a'))),
            Action::PickerChar('a')
        );
    }

    #[test]
    fn picker_backspace() {
        assert_eq!(
            handle_key_picker(make_key(KeyCode::Backspace)),
            Action::PickerBackspace
        );
    }

    #[test]
    fn picker_unknown_key_returns_none() {
        assert_eq!(handle_key_picker(make_key(KeyCode::F(12))), Action::None);
    }
}
