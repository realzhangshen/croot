use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};
use unicode_width::UnicodeWidthStr;

use super::colors;

/// An item in the context menu.
#[derive(Debug, Clone)]
pub struct MenuItem {
    pub label: String,
    pub action: MenuAction,
}

/// Actions triggered by context menu selections.
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub enum MenuAction {
    OpenInEditor,
    OpenExternally,
    CopyPath,
    CopyAbsPath,
    RevealInFinder,
    NewFile,
    NewDir,
    Rename,
    Delete,
    TogglePreview,
    Refresh,
    CollapseAll,
    StartFind,
    /// Inert separator — no action should be triggered.
    Separator,
}

/// State for the visible context menu.
#[derive(Debug, Clone)]
pub struct ContextMenuState {
    /// Screen position where the menu was triggered.
    pub x: u16,
    pub y: u16,
    /// Index of the tree node the menu was opened on.
    pub node_idx: usize,
    /// Currently highlighted menu item.
    pub selected: usize,
    /// Menu items.
    pub items: Vec<MenuItem>,
}

impl ContextMenuState {
    pub fn new_for_file(x: u16, y: u16, node_idx: usize) -> Self {
        let items = vec![
            MenuItem {
                label: "Open in Editor".into(),
                action: MenuAction::OpenInEditor,
            },
            MenuItem {
                label: "Open Externally".into(),
                action: MenuAction::OpenExternally,
            },
            MenuItem {
                label: "─".into(),
                action: MenuAction::Separator,
            }, // separator (inert)
            MenuItem {
                label: "Copy Relative Path".into(),
                action: MenuAction::CopyPath,
            },
            MenuItem {
                label: "Copy Absolute Path".into(),
                action: MenuAction::CopyAbsPath,
            },
            MenuItem {
                label: "Reveal in Finder".into(),
                action: MenuAction::RevealInFinder,
            },
            MenuItem {
                label: "─".into(),
                action: MenuAction::Separator,
            }, // separator (inert)
            MenuItem {
                label: "Rename".into(),
                action: MenuAction::Rename,
            },
            MenuItem {
                label: "Delete".into(),
                action: MenuAction::Delete,
            },
        ];
        Self {
            x,
            y,
            node_idx,
            selected: 0,
            items,
        }
    }

    pub fn new_for_workspace(x: u16, y: u16, node_idx: usize) -> Self {
        Self {
            x,
            y,
            node_idx,
            selected: 0,
            items: vec![
                MenuItem {
                    label: "New File".into(),
                    action: MenuAction::NewFile,
                },
                MenuItem {
                    label: "New Directory".into(),
                    action: MenuAction::NewDir,
                },
                MenuItem {
                    label: "─".into(),
                    action: MenuAction::Separator,
                },
                MenuItem {
                    label: "Refresh".into(),
                    action: MenuAction::Refresh,
                },
                MenuItem {
                    label: "Collapse All".into(),
                    action: MenuAction::CollapseAll,
                },
                MenuItem {
                    label: "Toggle Preview".into(),
                    action: MenuAction::TogglePreview,
                },
                MenuItem {
                    label: "Find".into(),
                    action: MenuAction::StartFind,
                },
            ],
        }
    }

    pub fn new_for_dir(x: u16, y: u16, node_idx: usize) -> Self {
        let items = vec![
            MenuItem {
                label: "New File".into(),
                action: MenuAction::NewFile,
            },
            MenuItem {
                label: "New Directory".into(),
                action: MenuAction::NewDir,
            },
            MenuItem {
                label: "─".into(),
                action: MenuAction::Separator,
            },
            MenuItem {
                label: "Collapse All".into(),
                action: MenuAction::CollapseAll,
            },
            MenuItem {
                label: "Toggle Preview".into(),
                action: MenuAction::TogglePreview,
            },
            MenuItem {
                label: "─".into(),
                action: MenuAction::Separator,
            },
            MenuItem {
                label: "Copy Relative Path".into(),
                action: MenuAction::CopyPath,
            },
            MenuItem {
                label: "Copy Absolute Path".into(),
                action: MenuAction::CopyAbsPath,
            },
            MenuItem {
                label: "Reveal in Finder".into(),
                action: MenuAction::RevealInFinder,
            },
            MenuItem {
                label: "─".into(),
                action: MenuAction::Separator,
            }, // separator
            MenuItem {
                label: "Rename".into(),
                action: MenuAction::Rename,
            },
            MenuItem {
                label: "Delete".into(),
                action: MenuAction::Delete,
            },
        ];
        Self {
            x,
            y,
            node_idx,
            selected: 0,
            items,
        }
    }

    pub fn move_up(&mut self) {
        while self.selected > 0 {
            self.selected -= 1;
            if self.items[self.selected].action != MenuAction::Separator {
                break;
            }
        }
    }

    pub fn move_down(&mut self) {
        while self.selected + 1 < self.items.len() {
            self.selected += 1;
            if self.items[self.selected].action != MenuAction::Separator {
                break;
            }
        }
    }

    pub fn selected_action(&self) -> Option<&MenuAction> {
        let item = self.items.get(self.selected)?;
        if item.action == MenuAction::Separator {
            None
        } else {
            Some(&item.action)
        }
    }

    /// Return the menu rect, clamped to fit within the terminal area.
    pub fn menu_rect(&self, terminal_width: u16, terminal_height: u16) -> Rect {
        let width = self
            .items
            .iter()
            .map(|i| i.label.width())
            .max()
            .unwrap_or(10) as u16
            + 4;
        let height = self.items.len() as u16 + 2; // +2 for border

        let x = if self.x + width > terminal_width {
            terminal_width.saturating_sub(width)
        } else {
            self.x
        };
        let y = if self.y + height > terminal_height {
            terminal_height.saturating_sub(height)
        } else {
            self.y
        };

        Rect::new(x, y, width.min(terminal_width), height.min(terminal_height))
    }

    /// Check if a screen position (col, row) is inside the menu.
    pub fn contains(&self, col: u16, row: u16, terminal_width: u16, terminal_height: u16) -> bool {
        let rect = self.menu_rect(terminal_width, terminal_height);
        col >= rect.x && col < rect.x + rect.width && row >= rect.y && row < rect.y + rect.height
    }

    /// Convert a screen row to a menu item index (if valid).
    pub fn row_to_item(
        &self,
        row: u16,
        terminal_width: u16,
        terminal_height: u16,
    ) -> Option<usize> {
        let rect = self.menu_rect(terminal_width, terminal_height);
        if row <= rect.y || row >= rect.y + rect.height - 1 {
            return None; // border rows
        }
        let idx = (row - rect.y - 1) as usize;
        if idx < self.items.len() && self.items[idx].action != MenuAction::Separator {
            Some(idx)
        } else {
            None
        }
    }
}

pub struct ContextMenuWidget<'a> {
    pub state: &'a ContextMenuState,
}

impl Widget for ContextMenuWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let menu_rect = self
            .state
            .menu_rect(area.x + area.width, area.y + area.height);

        let base = colors::popup_base();
        let border_style = colors::popup_border();
        let normal_style = base;
        let selected_style = colors::popup_selected();
        let selected_danger_style = colors::popup_selected_danger();
        let separator_style = colors::popup_border();

        // Fill background with REVERSED base
        colors::clear_region(buf, menu_rect, base);

        // Top border
        if let Some(cell) = buf.cell_mut((menu_rect.x, menu_rect.y)) {
            cell.set_symbol("┌");
            cell.set_style(border_style);
        }
        for x in (menu_rect.x + 1)..(menu_rect.x + menu_rect.width - 1) {
            if let Some(cell) = buf.cell_mut((x, menu_rect.y)) {
                cell.set_symbol("─");
                cell.set_style(border_style);
            }
        }
        if menu_rect.width > 1 {
            if let Some(cell) = buf.cell_mut((menu_rect.x + menu_rect.width - 1, menu_rect.y)) {
                cell.set_symbol("┐");
                cell.set_style(border_style);
            }
        }

        // Bottom border
        let bottom_y = menu_rect.y + menu_rect.height - 1;
        if let Some(cell) = buf.cell_mut((menu_rect.x, bottom_y)) {
            cell.set_symbol("└");
            cell.set_style(border_style);
        }
        for x in (menu_rect.x + 1)..(menu_rect.x + menu_rect.width - 1) {
            if let Some(cell) = buf.cell_mut((x, bottom_y)) {
                cell.set_symbol("─");
                cell.set_style(border_style);
            }
        }
        if menu_rect.width > 1 {
            if let Some(cell) = buf.cell_mut((menu_rect.x + menu_rect.width - 1, bottom_y)) {
                cell.set_symbol("┘");
                cell.set_style(border_style);
            }
        }

        // Side borders and menu items
        for (i, item) in self.state.items.iter().enumerate() {
            let y = menu_rect.y + 1 + i as u16;
            if y >= menu_rect.y + menu_rect.height - 1 {
                break;
            }

            // Left border
            if let Some(cell) = buf.cell_mut((menu_rect.x, y)) {
                cell.set_symbol("│");
                cell.set_style(border_style);
            }
            // Right border
            if let Some(cell) = buf.cell_mut((menu_rect.x + menu_rect.width - 1, y)) {
                cell.set_symbol("│");
                cell.set_style(border_style);
            }

            let is_separator = item.action == MenuAction::Separator;
            let is_selected = i == self.state.selected && !is_separator;
            let is_delete = item.action == MenuAction::Delete;

            let style = if is_separator {
                separator_style
            } else if is_selected && is_delete {
                selected_danger_style
            } else if is_selected {
                selected_style
            } else {
                normal_style
            };

            // Fill row with style
            if is_selected {
                for x in (menu_rect.x + 1)..(menu_rect.x + menu_rect.width - 1) {
                    if let Some(cell) = buf.cell_mut((x, y)) {
                        cell.set_style(style);
                    }
                }
            }

            // Render item text
            if is_separator {
                let content_width = menu_rect.width.saturating_sub(2) as usize;
                let separator_line: String = "─".repeat(content_width);
                buf.set_string(menu_rect.x + 1, y, &separator_line, style);
            } else {
                let text = format!(" {} ", item.label);
                let content_width = menu_rect.width.saturating_sub(2) as usize;
                let display = super::status_bar::truncate_to_display_width(&text, content_width);
                buf.set_string(menu_rect.x + 1, y, &display, style);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::style::{Color, Modifier};

    /// Render a context menu into a buffer and return it for inspection.
    fn render_menu(state: &ContextMenuState) -> ratatui::buffer::Buffer {
        let area = ratatui::layout::Rect::new(0, 0, 40, 20);
        let mut buf = ratatui::buffer::Buffer::empty(area);
        let widget = ContextMenuWidget { state };
        widget.render(area, &mut buf);
        buf
    }

    #[test]
    fn normal_item_has_popup_bg_fg() {
        let state = ContextMenuState::new_for_file(0, 0, 0);
        let buf = render_menu(&state);
        let rect = state.menu_rect(40, 20);
        // Item at index 1 (not selected when selected==0) — check a cell in that row
        let y = rect.y + 2; // second item row (index 1)
        let x = rect.x + 2;
        let cell = buf.cell((x, y)).unwrap();
        // popup_base() uses REVERSED with default (Reset) fg/bg
        assert_eq!(
            cell.bg,
            Color::Reset,
            "normal menu item bg should be Reset (REVERSED)"
        );
        assert_eq!(
            cell.fg,
            Color::Reset,
            "normal menu item fg should be Reset (REVERSED)"
        );
        assert!(
            cell.modifier.contains(Modifier::REVERSED),
            "normal menu item should have REVERSED"
        );
    }

    #[test]
    fn workspace_menu_has_new_file_and_dir() {
        let state = ContextMenuState::new_for_workspace(0, 0, 999);
        assert_eq!(state.items.len(), 7);
        assert_eq!(state.items[0].action, MenuAction::NewFile);
        assert_eq!(state.items[1].action, MenuAction::NewDir);
        assert_eq!(state.items[2].action, MenuAction::Separator);
        assert_eq!(state.items[3].action, MenuAction::Refresh);
        assert_eq!(state.items[4].action, MenuAction::CollapseAll);
        assert_eq!(state.items[5].action, MenuAction::TogglePreview);
        assert_eq!(state.items[6].action, MenuAction::StartFind);
    }

    #[test]
    fn selected_item_has_blue_bg_white_fg() {
        let state = ContextMenuState::new_for_file(0, 0, 0);
        let buf = render_menu(&state);
        let rect = state.menu_rect(40, 20);
        // Item at index 0 is selected — check a cell in that row
        let y = rect.y + 1;
        let x = rect.x + 2;
        let cell = buf.cell((x, y)).unwrap();
        assert_eq!(
            cell.bg,
            Color::Blue,
            "selected menu item should have Blue bg"
        );
        assert_eq!(
            cell.fg,
            Color::White,
            "selected menu item should have White fg"
        );
        assert!(
            cell.modifier.contains(Modifier::BOLD),
            "selected menu item should have BOLD, got {:?}",
            cell.modifier
        );
    }

    #[test]
    fn unselected_delete_has_no_red() {
        let state = ContextMenuState::new_for_file(0, 0, 0);
        let buf = render_menu(&state);
        let rect = state.menu_rect(40, 20);
        // Find the Delete item index
        let delete_idx = state
            .items
            .iter()
            .position(|i| i.action == MenuAction::Delete)
            .unwrap();
        // selected==0, so Delete is not selected
        assert_ne!(state.selected, delete_idx);
        let y = rect.y + 1 + delete_idx as u16;
        let x = rect.x + 2;
        let cell = buf.cell((x, y)).unwrap();
        assert_ne!(
            cell.fg,
            ratatui::style::Color::Red,
            "unselected Delete should not have red fg"
        );
        assert_ne!(
            cell.bg,
            ratatui::style::Color::Red,
            "unselected Delete should not have red bg"
        );
    }

    #[test]
    fn no_color_bleed_from_underlying_content() {
        let state = ContextMenuState::new_for_file(0, 0, 0);
        let area = ratatui::layout::Rect::new(0, 0, 40, 20);
        let mut buf = ratatui::buffer::Buffer::empty(area);

        // Pre-fill entire buffer with colored content (simulating syntax highlighting)
        for row in 0..area.height {
            for col in 0..area.width {
                if let Some(cell) = buf.cell_mut((col, row)) {
                    cell.set_symbol("X");
                    cell.set_style(
                        ratatui::style::Style::default()
                            .fg(ratatui::style::Color::Red)
                            .bg(ratatui::style::Color::Green),
                    );
                }
            }
        }

        // Render menu on top
        let widget = ContextMenuWidget { state: &state };
        widget.render(area, &mut buf);

        let rect = state.menu_rect(40, 20);
        // Check interior cells (skip borders) for stale colors.
        // clear_region() resets cells before applying REVERSED, so no cell
        // should retain the pre-filled Red fg or Green bg.
        for row in (rect.y + 1)..(rect.y + rect.height - 1) {
            for col in (rect.x + 1)..(rect.x + rect.width - 1) {
                let cell = buf.cell((col, row)).unwrap();
                assert_ne!(
                    cell.fg,
                    ratatui::style::Color::Red,
                    "stale Red fg at ({col},{row})"
                );
                assert_ne!(
                    cell.bg,
                    ratatui::style::Color::Green,
                    "stale Green bg at ({col},{row})"
                );
                assert_ne!(
                    cell.fg,
                    ratatui::style::Color::Green,
                    "stale Green fg at ({col},{row})"
                );
            }
        }
    }

    #[test]
    fn copy_path_label_says_relative() {
        let file_state = ContextMenuState::new_for_file(0, 0, 0);
        let copy_item = file_state
            .items
            .iter()
            .find(|i| i.action == MenuAction::CopyPath)
            .expect("file menu should have a CopyPath item");
        assert_eq!(copy_item.label, "Copy Relative Path");

        let dir_state = ContextMenuState::new_for_dir(0, 0, 0);
        let copy_item = dir_state
            .items
            .iter()
            .find(|i| i.action == MenuAction::CopyPath)
            .expect("dir menu should have a CopyPath item");
        assert_eq!(copy_item.label, "Copy Relative Path");
    }

    #[test]
    fn selected_delete_has_red_bg() {
        let mut state = ContextMenuState::new_for_file(0, 0, 0);
        let delete_idx = state
            .items
            .iter()
            .position(|i| i.action == MenuAction::Delete)
            .unwrap();
        state.selected = delete_idx;
        let buf = render_menu(&state);
        let rect = state.menu_rect(40, 20);
        let y = rect.y + 1 + delete_idx as u16;
        let x = rect.x + 2;
        let cell = buf.cell((x, y)).unwrap();
        assert_eq!(
            cell.bg,
            colors::popup_selected_danger_bg(),
            "selected Delete should have Red bg"
        );
        assert_eq!(
            cell.fg,
            colors::popup_fg(),
            "selected Delete should use POPUP_FG"
        );
        assert!(
            cell.modifier.contains(Modifier::BOLD),
            "selected Delete should have BOLD"
        );
        assert!(
            !cell.modifier.contains(Modifier::REVERSED),
            "selected Delete should NOT have REVERSED"
        );
    }

    #[test]
    fn move_down_skips_separator_in_workspace_menu() {
        let mut state = ContextMenuState::new_for_workspace(0, 0, 0);
        assert_eq!(state.selected, 0); // NewFile
        state.move_down(); // → NewDir (1)
        assert_eq!(state.items[state.selected].action, MenuAction::NewDir);
        state.move_down(); // → skip Separator (2) → Refresh (3)
        assert_eq!(state.items[state.selected].action, MenuAction::Refresh);
    }

    #[test]
    fn move_up_skips_separator_in_workspace_menu() {
        let mut state = ContextMenuState::new_for_workspace(0, 0, 0);
        state.selected = 3; // Refresh
        state.move_up(); // → skip Separator (2) → NewDir (1)
        assert_eq!(state.items[state.selected].action, MenuAction::NewDir);
    }
}
