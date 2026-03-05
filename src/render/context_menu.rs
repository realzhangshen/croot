use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::Widget,
};

use super::colors;

/// An item in the context menu.
#[derive(Debug, Clone)]
pub struct MenuItem {
    pub label: String,
    pub action: MenuAction,
}

/// Actions triggered by context menu selections.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MenuAction {
    OpenEditor,
    CmuxPreview,
    CopyPath,
    CopyAbsPath,
    RevealInFinder,
    NewFile,
    NewDir,
    Rename,
    Delete,
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
        Self {
            x,
            y,
            node_idx,
            selected: 0,
            items: vec![
                MenuItem { label: "Open in Editor".into(), action: MenuAction::OpenEditor },
                MenuItem { label: "Preview (cmux)".into(), action: MenuAction::CmuxPreview },
                MenuItem { label: "Copy Path".into(), action: MenuAction::CopyPath },
                MenuItem { label: "Copy Absolute Path".into(), action: MenuAction::CopyAbsPath },
                MenuItem { label: "Reveal in Finder".into(), action: MenuAction::RevealInFinder },
                MenuItem { label: "────────────────".into(), action: MenuAction::CopyPath }, // separator (inert)
                MenuItem { label: "Rename".into(), action: MenuAction::Rename },
                MenuItem { label: "Delete".into(), action: MenuAction::Delete },
            ],
        }
    }

    pub fn new_for_dir(x: u16, y: u16, node_idx: usize) -> Self {
        Self {
            x,
            y,
            node_idx,
            selected: 0,
            items: vec![
                MenuItem { label: "New File".into(), action: MenuAction::NewFile },
                MenuItem { label: "New Directory".into(), action: MenuAction::NewDir },
                MenuItem { label: "Copy Path".into(), action: MenuAction::CopyPath },
                MenuItem { label: "Copy Absolute Path".into(), action: MenuAction::CopyAbsPath },
                MenuItem { label: "Reveal in Finder".into(), action: MenuAction::RevealInFinder },
                MenuItem { label: "────────────────".into(), action: MenuAction::CopyPath }, // separator
                MenuItem { label: "Rename".into(), action: MenuAction::Rename },
                MenuItem { label: "Delete".into(), action: MenuAction::Delete },
            ],
        }
    }

    pub fn move_up(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
            // Skip separator
            if self.items[self.selected].label.starts_with('─') && self.selected > 0 {
                self.selected -= 1;
            }
        }
    }

    pub fn move_down(&mut self) {
        if self.selected + 1 < self.items.len() {
            self.selected += 1;
            // Skip separator
            if self.items[self.selected].label.starts_with('─')
                && self.selected + 1 < self.items.len()
            {
                self.selected += 1;
            }
        }
    }

    pub fn selected_action(&self) -> &MenuAction {
        &self.items[self.selected].action
    }

    /// Return the menu rect, clamped to fit within the terminal area.
    pub fn menu_rect(&self, terminal_width: u16, terminal_height: u16) -> Rect {
        let width = self.items.iter().map(|i| i.label.len()).max().unwrap_or(10) as u16 + 4;
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
    pub fn row_to_item(&self, row: u16, terminal_width: u16, terminal_height: u16) -> Option<usize> {
        let rect = self.menu_rect(terminal_width, terminal_height);
        if row <= rect.y || row >= rect.y + rect.height - 1 {
            return None; // border rows
        }
        let idx = (row - rect.y - 1) as usize;
        if idx < self.items.len() && !self.items[idx].label.starts_with('─') {
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
        let menu_rect = self.state.menu_rect(area.x + area.width, area.y + area.height);

        let border_style = Style::default().fg(colors::TREE_LINE);
        let bg = colors::STATUS_BAR_BG;
        let normal_style = Style::default().fg(Color::Reset).bg(bg);
        let selected_style = Style::default()
            .fg(Color::White)
            .bg(colors::SELECTED_BG)
            .add_modifier(Modifier::BOLD);
        let separator_style = Style::default().fg(colors::TREE_LINE).bg(bg);
        let delete_style = Style::default().fg(Color::Red).bg(bg);

        // Fill background
        for y in menu_rect.y..menu_rect.y + menu_rect.height {
            for x in menu_rect.x..menu_rect.x + menu_rect.width {
                if let Some(cell) = buf.cell_mut((x, y)) {
                    cell.set_style(Style::default().bg(bg));
                    cell.set_symbol(" ");
                }
            }
        }

        // Top border
        if let Some(cell) = buf.cell_mut((menu_rect.x, menu_rect.y)) {
            cell.set_symbol("┌");
            cell.set_style(border_style.bg(bg));
        }
        for x in (menu_rect.x + 1)..(menu_rect.x + menu_rect.width - 1) {
            if let Some(cell) = buf.cell_mut((x, menu_rect.y)) {
                cell.set_symbol("─");
                cell.set_style(border_style.bg(bg));
            }
        }
        if menu_rect.width > 1 {
            if let Some(cell) = buf.cell_mut((menu_rect.x + menu_rect.width - 1, menu_rect.y)) {
                cell.set_symbol("┐");
                cell.set_style(border_style.bg(bg));
            }
        }

        // Bottom border
        let bottom_y = menu_rect.y + menu_rect.height - 1;
        if let Some(cell) = buf.cell_mut((menu_rect.x, bottom_y)) {
            cell.set_symbol("└");
            cell.set_style(border_style.bg(bg));
        }
        for x in (menu_rect.x + 1)..(menu_rect.x + menu_rect.width - 1) {
            if let Some(cell) = buf.cell_mut((x, bottom_y)) {
                cell.set_symbol("─");
                cell.set_style(border_style.bg(bg));
            }
        }
        if menu_rect.width > 1 {
            if let Some(cell) = buf.cell_mut((menu_rect.x + menu_rect.width - 1, bottom_y)) {
                cell.set_symbol("┘");
                cell.set_style(border_style.bg(bg));
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
                cell.set_style(border_style.bg(bg));
            }
            // Right border
            if let Some(cell) = buf.cell_mut((menu_rect.x + menu_rect.width - 1, y)) {
                cell.set_symbol("│");
                cell.set_style(border_style.bg(bg));
            }

            let is_separator = item.label.starts_with('─');
            let is_selected = i == self.state.selected && !is_separator;
            let is_delete = item.action == MenuAction::Delete;

            let style = if is_separator {
                separator_style
            } else if is_selected {
                selected_style
            } else if is_delete {
                delete_style
            } else {
                normal_style
            };

            // Fill row with style
            if is_selected {
                for x in (menu_rect.x + 1)..(menu_rect.x + menu_rect.width - 1) {
                    if let Some(cell) = buf.cell_mut((x, y)) {
                        cell.set_style(selected_style);
                    }
                }
            }

            // Render item text
            let text = format!(" {} ", item.label);
            let content_width = (menu_rect.width - 2) as usize;
            let display = if text.len() > content_width {
                &text[..content_width]
            } else {
                &text
            };
            buf.set_string(menu_rect.x + 1, y, display, style);
        }
    }
}
