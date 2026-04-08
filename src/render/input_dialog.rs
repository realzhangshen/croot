use ratatui::{buffer::Buffer, layout::Rect, style::Modifier, widgets::Widget};

use unicode_width::UnicodeWidthStr;

use super::colors;
use super::popup::draw_border;
pub use crate::file_ops::DialogKind;

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
    /// Whether to move to trash (true) or permanently delete (false).
    pub use_trash: bool,
}

/// Compute the centered dialog rect for an input dialog of the given kind.
/// Shared between render and mouse-hit-test to avoid layout drift.
pub fn input_dialog_rect(area: Rect, kind: &DialogKind) -> Rect {
    let dialog_width = 50u16.min(area.width.saturating_sub(4));
    let dialog_height = if matches!(kind, DialogKind::ConfirmDelete) {
        6
    } else {
        5
    };
    let x = area.x + (area.width.saturating_sub(dialog_width)) / 2;
    let y = area.y + (area.height.saturating_sub(dialog_height)) / 2;
    Rect::new(x, y, dialog_width, dialog_height)
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
            use_trash: false,
        }
    }

    /// Dialog for creating a new file inside `dir`.
    pub fn for_new_file(dir: std::path::PathBuf) -> Self {
        Self::new(DialogKind::NewFile, dir, String::new())
    }

    /// Dialog for creating a new directory inside `dir`.
    pub fn for_new_dir(dir: std::path::PathBuf) -> Self {
        Self::new(DialogKind::NewDir, dir, String::new())
    }

    /// Dialog for renaming a node (pre-fills the input with the existing name).
    pub fn for_rename(path: std::path::PathBuf, name: String) -> Self {
        Self::new(DialogKind::Rename, path, name)
    }

    /// Confirmation dialog for deleting a node. `use_trash` picks the
    /// trash-vs-permanent copy in the rendered message.
    pub fn for_delete(path: std::path::PathBuf, name: String, use_trash: bool) -> Self {
        let mut dialog = Self::new(DialogKind::ConfirmDelete, path, name);
        dialog.use_trash = use_trash;
        dialog
    }

    pub fn insert_char(&mut self, ch: char) {
        self.input.insert(self.cursor_pos, ch);
        self.cursor_pos += ch.len_utf8();
    }

    pub fn insert_str(&mut self, s: &str) {
        self.input.insert_str(self.cursor_pos, s);
        self.cursor_pos += s.len();
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

    /// Returns `(confirm_rect, cancel_rect)` in screen coordinates for the button row.
    pub fn button_positions(&self, area: Rect) -> (Rect, Rect) {
        let dialog = input_dialog_rect(area, &self.kind);
        let btn_y = dialog.y + 3;
        let confirm_x = dialog.x + 2;
        let confirm_w = 9; // "[Confirm]".len()
        let cancel_x = confirm_x + confirm_w + 2;
        let cancel_w = 8; // "[Cancel]".len()

        (
            Rect::new(confirm_x, btn_y, confirm_w, 1),
            Rect::new(cancel_x, btn_y, cancel_w, 1),
        )
    }
}

pub struct InputDialogWidget<'a> {
    pub state: &'a InputDialogState,
}

impl Widget for InputDialogWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let dialog_rect = input_dialog_rect(area, &self.state.kind);

        let base = colors::popup_base();
        let border_style = colors::popup_border();
        let text_style = base;
        let title_style = base.add_modifier(Modifier::BOLD);

        // Fill background with REVERSED base
        colors::clear_region(buf, dialog_rect, base);

        // Draw border
        draw_border(buf, dialog_rect, border_style);

        // Title
        let title = self.state.kind.title();
        let title_x = dialog_rect.x
            + (dialog_rect
                .width
                .saturating_sub(UnicodeWidthStr::width(title) as u16 + 2))
                / 2;
        buf.set_string(title_x, dialog_rect.y, format!(" {title} "), title_style);

        if matches!(self.state.kind, DialogKind::ConfirmDelete) {
            // Show confirmation message
            let msg = if self.state.use_trash {
                format!("Move '{}' to Trash?", self.state.target_name)
            } else {
                format!("Permanently delete '{}'?", self.state.target_name)
            };
            let msg_x = dialog_rect.x + 2;
            buf.set_string(msg_x, dialog_rect.y + 2, &msg, text_style);

            let btn_y = dialog_rect.y + 3;
            let confirm_x = dialog_rect.x + 2;
            buf.set_string(confirm_x, btn_y, "[Confirm]", colors::popup_selected());
            let cancel_x = confirm_x + 9 + 2;
            buf.set_string(cancel_x, btn_y, "[Cancel]", colors::popup_dim());
        } else {
            // Input field
            let input_y = dialog_rect.y + 2;
            let input_x = dialog_rect.x + 2;
            let input_width = dialog_rect.width.saturating_sub(4) as usize;

            // Draw input background (sunken field)
            let input_style = colors::popup_input();
            for dx in 0..input_width {
                if let Some(cell) = buf.cell_mut((input_x + dx as u16, input_y)) {
                    cell.reset();
                    cell.set_style(input_style);
                }
            }

            // Draw input text
            let display_text =
                super::text_util::truncate_start_to_display_width(&self.state.input, input_width);
            buf.set_string(input_x, input_y, &display_text, input_style);

            // Draw cursor (block cursor: swap fg/bg)
            let cursor_col =
                super::search_bar::cursor_byte_to_column(&self.state.input, self.state.cursor_pos);
            let input_display_width = UnicodeWidthStr::width(self.state.input.as_str());
            let cursor_display_pos = if input_display_width > input_width {
                // Text is scrolled: cursor is at the end of visible area offset by
                // how far the cursor is from the end of the text
                let dist_from_end = input_display_width - cursor_col;
                input_width.saturating_sub(dist_from_end)
            } else {
                cursor_col
            };
            if let Some(cell) = buf.cell_mut((input_x + cursor_display_pos as u16, input_y)) {
                cell.set_style(colors::popup_cursor());
            }

            // Buttons
            let btn_y = dialog_rect.y + 3;
            let confirm_x = dialog_rect.x + 2;
            buf.set_string(confirm_x, btn_y, "[Confirm]", colors::popup_selected());
            let cancel_x = confirm_x + 9 + 2;
            buf.set_string(cancel_x, btn_y, "[Cancel]", colors::popup_dim());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::style::{Color, Modifier};

    fn render_dialog(state: &InputDialogState) -> ratatui::buffer::Buffer {
        let area = ratatui::layout::Rect::new(0, 0, 60, 20);
        let mut buf = ratatui::buffer::Buffer::empty(area);
        let widget = InputDialogWidget { state };
        widget.render(area, &mut buf);
        buf
    }

    /// Helper to extract the rendered text from a buffer row.
    fn row_text(buf: &ratatui::buffer::Buffer, y: u16, x_start: u16, x_end: u16) -> String {
        (x_start..x_end)
            .filter_map(|x| buf.cell((x, y)).map(|c| c.symbol().to_string()))
            .collect::<String>()
    }

    // ── insert_str tests ──────────────────────────────────────────────

    #[test]
    fn insert_str_empty_is_noop() {
        let mut state = InputDialogState::new(
            DialogKind::NewFile,
            std::path::PathBuf::from("/tmp"),
            String::new(),
        );
        state.insert_str("");
        assert_eq!(state.input, "");
        assert_eq!(state.cursor_pos, 0);
    }

    #[test]
    fn insert_str_ascii_at_end() {
        let mut state = InputDialogState::new(
            DialogKind::NewFile,
            std::path::PathBuf::from("/tmp"),
            String::new(),
        );
        state.insert_str("hello.txt");
        assert_eq!(state.input, "hello.txt");
        assert_eq!(state.cursor_pos, 9);
    }

    #[test]
    fn insert_str_mid_cursor() {
        let mut state = InputDialogState::new(
            DialogKind::Rename,
            std::path::PathBuf::from("/tmp"),
            "ac".to_string(),
        );
        state.cursor_pos = 1; // between 'a' and 'c'
        state.insert_str("b");
        assert_eq!(state.input, "abc");
        assert_eq!(state.cursor_pos, 2);
    }

    #[test]
    fn insert_str_multibyte_utf8() {
        let mut state = InputDialogState::new(
            DialogKind::NewFile,
            std::path::PathBuf::from("/tmp"),
            String::new(),
        );
        state.insert_str("日本語");
        assert_eq!(state.input, "日本語");
        assert_eq!(state.cursor_pos, 9); // 3 × 3 bytes
    }

    #[test]
    fn delete_dialog_shows_trash_message_when_use_trash() {
        let mut state = InputDialogState::new(
            DialogKind::ConfirmDelete,
            std::path::PathBuf::from("/tmp"),
            "foo.txt".to_string(),
        );
        state.use_trash = true;
        let buf = render_dialog(&state);
        // Dialog centered: x=5, y=7 (height=6), message at y+2=9
        let text = row_text(&buf, 9, 5, 55);
        assert!(
            text.contains("Move 'foo.txt' to Trash?"),
            "Expected trash message, got: {text}"
        );
    }

    #[test]
    fn delete_dialog_shows_permanent_message_when_no_trash() {
        let mut state = InputDialogState::new(
            DialogKind::ConfirmDelete,
            std::path::PathBuf::from("/tmp"),
            "foo.txt".to_string(),
        );
        state.use_trash = false;
        let buf = render_dialog(&state);
        let text = row_text(&buf, 9, 5, 55);
        assert!(
            text.contains("Permanently delete 'foo.txt'?"),
            "Expected permanent delete message, got: {text}"
        );
    }

    #[test]
    fn dialog_container_has_popup_bg() {
        let state = InputDialogState::new(
            DialogKind::NewFile,
            std::path::PathBuf::from("/tmp"),
            String::new(),
        );
        let buf = render_dialog(&state);
        // Check a cell inside the dialog fill (row between title and input)
        // Dialog at (5, 7, 50, 5): y=8 is the blank row below the title
        let mid_x = 30u16;
        let mid_y = 8u16;
        let cell = buf.cell((mid_x, mid_y)).unwrap();
        // popup_base() uses REVERSED with default (Reset) fg/bg
        assert_eq!(
            cell.bg,
            Color::Reset,
            "dialog bg should be Reset (REVERSED)"
        );
        assert_eq!(
            cell.fg,
            Color::Reset,
            "dialog fg should be Reset (REVERSED)"
        );
        assert!(
            cell.modifier.contains(Modifier::REVERSED),
            "dialog should have REVERSED, got {:?}",
            cell.modifier
        );
    }

    #[test]
    fn input_area_has_sunken_bg() {
        let state = InputDialogState::new(
            DialogKind::NewFile,
            std::path::PathBuf::from("/tmp"),
            String::new(),
        );
        let buf = render_dialog(&state);
        // Input field is at dialog_rect.y + 2, dialog_rect.x + 2
        // Dialog is centered: x = (60 - 50) / 2 = 5, y = (20 - 5) / 2 = 7
        // input_x = 7, input_y = 9; skip pos 0 (cursor) and check pos 1+
        let input_x = 8u16; // 5 + 2 + 1 (skip cursor at offset 0)
        let input_y = 9u16; // 7 + 2
        let cell = buf.cell((input_x, input_y)).unwrap();
        assert_eq!(
            cell.bg,
            colors::popup_input_bg(),
            "input area should have POPUP_INPUT_BG"
        );
        assert!(
            !cell.modifier.contains(Modifier::REVERSED),
            "input area should NOT have REVERSED, got {:?}",
            cell.modifier
        );
    }

    #[test]
    fn title_has_bold_and_reversed() {
        let state = InputDialogState::new(
            DialogKind::Rename,
            std::path::PathBuf::from("/tmp"),
            "test.txt".to_string(),
        );
        let buf = render_dialog(&state);
        // Title is centered on the top border row
        // Dialog x = 5, y = 7, title "Rename" is 6 chars, padded to " Rename "
        // title_x = 5 + (50 - 8) / 2 = 5 + 21 = 26
        let title_x = 26u16;
        let title_y = 7u16;
        let cell = buf.cell((title_x + 1, title_y)).unwrap();
        assert!(
            cell.modifier.contains(Modifier::BOLD),
            "title should have BOLD, got {:?}",
            cell.modifier
        );
        assert!(
            cell.modifier.contains(Modifier::REVERSED),
            "title should have REVERSED (popup border style), got {:?}",
            cell.modifier
        );
    }
}
