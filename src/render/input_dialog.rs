use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::Widget,
};

use super::colors;

/// The kind of dialog being shown.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DialogKind {
    NewFile,
    NewDir,
    Rename,
    ConfirmDelete,
}

impl DialogKind {
    fn title(&self) -> &'static str {
        match self {
            Self::NewFile => "New File",
            Self::NewDir => "New Directory",
            Self::Rename => "Rename",
            Self::ConfirmDelete => "Confirm Delete",
        }
    }
}

/// State for the input dialog overlay.
#[derive(Debug, Clone)]
pub struct InputDialogState {
    pub kind: DialogKind,
    pub input: String,
    pub cursor_pos: usize,
    /// Context: the path being acted upon (e.g. parent dir for new, file for rename/delete).
    pub context_path: std::path::PathBuf,
    /// Display name of the target (for delete confirmation).
    pub target_name: String,
}

impl InputDialogState {
    pub fn new(kind: DialogKind, context_path: std::path::PathBuf, target_name: String) -> Self {
        let input = if kind == DialogKind::Rename {
            target_name.clone()
        } else {
            String::new()
        };
        let cursor_pos = input.len();
        Self {
            kind,
            input,
            cursor_pos,
            context_path,
            target_name,
        }
    }

    pub fn insert_char(&mut self, ch: char) {
        self.input.insert(self.cursor_pos, ch);
        self.cursor_pos += ch.len_utf8();
    }

    pub fn delete_char(&mut self) {
        if self.cursor_pos > 0 {
            let prev = self.input[..self.cursor_pos]
                .chars()
                .last()
                .map_or(0, char::len_utf8);
            self.cursor_pos -= prev;
            self.input.remove(self.cursor_pos);
        }
    }

    pub fn move_left(&mut self) {
        if self.cursor_pos > 0 {
            let prev = self.input[..self.cursor_pos]
                .chars()
                .last()
                .map_or(0, char::len_utf8);
            self.cursor_pos -= prev;
        }
    }

    pub fn move_right(&mut self) {
        if self.cursor_pos < self.input.len() {
            let next = self.input[self.cursor_pos..]
                .chars()
                .next()
                .map_or(0, char::len_utf8);
            self.cursor_pos += next;
        }
    }
}

pub struct InputDialogWidget<'a> {
    pub state: &'a InputDialogState,
}

impl Widget for InputDialogWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let dialog_width = 50u16.min(area.width.saturating_sub(4));
        let dialog_height = if self.state.kind == DialogKind::ConfirmDelete {
            6
        } else {
            5
        };

        let x = area.x + (area.width.saturating_sub(dialog_width)) / 2;
        let y = area.y + (area.height.saturating_sub(dialog_height)) / 2;

        let dialog_rect = Rect::new(x, y, dialog_width, dialog_height);

        let border_style = Style::default().fg(Color::Cyan);
        let bg = colors::STATUS_BAR_BG;
        let text_style = Style::default().fg(Color::Reset).bg(bg);
        let title_style = Style::default()
            .fg(Color::Cyan)
            .bg(bg)
            .add_modifier(Modifier::BOLD);

        // Fill background
        for dy in 0..dialog_rect.height {
            for dx in 0..dialog_rect.width {
                if let Some(cell) = buf.cell_mut((dialog_rect.x + dx, dialog_rect.y + dy)) {
                    cell.set_style(Style::default().bg(bg));
                    cell.set_symbol(" ");
                }
            }
        }

        // Draw border
        draw_border(buf, dialog_rect, border_style.bg(bg));

        // Title
        let title = self.state.kind.title();
        let title_x = dialog_rect.x + (dialog_rect.width.saturating_sub(title.len() as u16 + 2)) / 2;
        buf.set_string(title_x, dialog_rect.y, format!(" {title} "), title_style);

        if self.state.kind == DialogKind::ConfirmDelete {
            // Show confirmation message
            let msg = format!("Delete '{}'?", self.state.target_name);
            let msg_x = dialog_rect.x + 2;
            buf.set_string(msg_x, dialog_rect.y + 2, &msg, text_style);

            let hint = "[Enter] confirm  [Esc] cancel";
            let hint_x = dialog_rect.x + 2;
            buf.set_string(
                hint_x,
                dialog_rect.y + 3,
                hint,
                Style::default().fg(Color::DarkGray).bg(bg),
            );
        } else {
            // Input field
            let input_y = dialog_rect.y + 2;
            let input_x = dialog_rect.x + 2;
            let input_width = dialog_rect.width.saturating_sub(4) as usize;

            // Draw input background
            for dx in 0..input_width {
                if let Some(cell) = buf.cell_mut((input_x + dx as u16, input_y)) {
                    cell.set_style(Style::default().bg(Color::Black));
                    cell.set_symbol(" ");
                }
            }

            // Draw input text
            let display_text = if self.state.input.len() > input_width {
                &self.state.input[self.state.input.len() - input_width..]
            } else {
                &self.state.input
            };
            buf.set_string(
                input_x,
                input_y,
                display_text,
                Style::default().fg(Color::White).bg(Color::Black),
            );

            // Draw cursor
            let cursor_display_pos = if self.state.input.len() > input_width {
                input_width
            } else {
                self.state.cursor_pos
            };
            if let Some(cell) = buf.cell_mut((input_x + cursor_display_pos as u16, input_y)) {
                cell.set_style(
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::White),
                );
            }

            // Hint
            let hint = "[Enter] confirm  [Esc] cancel";
            buf.set_string(
                dialog_rect.x + 2,
                dialog_rect.y + 3,
                hint,
                Style::default().fg(Color::DarkGray).bg(bg),
            );
        }
    }
}

fn draw_border(buf: &mut Buffer, rect: Rect, style: Style) {
    // Corners
    if let Some(cell) = buf.cell_mut((rect.x, rect.y)) {
        cell.set_symbol("╭");
        cell.set_style(style);
    }
    if let Some(cell) = buf.cell_mut((rect.x + rect.width - 1, rect.y)) {
        cell.set_symbol("╮");
        cell.set_style(style);
    }
    if let Some(cell) = buf.cell_mut((rect.x, rect.y + rect.height - 1)) {
        cell.set_symbol("╰");
        cell.set_style(style);
    }
    if let Some(cell) = buf.cell_mut((rect.x + rect.width - 1, rect.y + rect.height - 1)) {
        cell.set_symbol("╯");
        cell.set_style(style);
    }

    // Horizontal edges
    for x in (rect.x + 1)..(rect.x + rect.width - 1) {
        if let Some(cell) = buf.cell_mut((x, rect.y)) {
            cell.set_symbol("─");
            cell.set_style(style);
        }
        if let Some(cell) = buf.cell_mut((x, rect.y + rect.height - 1)) {
            cell.set_symbol("─");
            cell.set_style(style);
        }
    }

    // Vertical edges
    for y in (rect.y + 1)..(rect.y + rect.height - 1) {
        if let Some(cell) = buf.cell_mut((rect.x, y)) {
            cell.set_symbol("│");
            cell.set_style(style);
        }
        if let Some(cell) = buf.cell_mut((rect.x + rect.width - 1, y)) {
            cell.set_symbol("│");
            cell.set_style(style);
        }
    }
}
