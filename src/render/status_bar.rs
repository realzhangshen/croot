use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::Widget,
};
use unicode_width::UnicodeWidthStr;

use super::colors;

pub struct StatusBar<'a> {
    pub branch: Option<&'a str>,
    pub file_count: usize,
    pub dir_count: usize,
    pub root_name: &'a str,
    pub root_path: &'a str,
    pub cmux_status: Option<&'a str>,
    pub selected_path: Option<&'a str>,
    pub selected_abs_path: Option<&'a str>,
}

impl Widget for StatusBar<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let style = Style::default()
            .fg(colors::STATUS_BAR_FG)
            .bg(colors::STATUS_BAR_BG);

        // Fill background
        for x in area.x..area.x + area.width {
            if let Some(cell) = buf.cell_mut((x, area.y)) {
                cell.set_style(style);
            }
        }

        let mut spans = Vec::new();
        let mut col: u16 = 0;

        // Branch info
        if let Some(branch) = self.branch {
            let s = format!("  {branch} ");
            col += s.width() as u16;
            spans.push(Span::styled(s, style));
            col += 2;
            spans.push(Span::styled("│ ", style));
        }

        // Root name — track position for hyperlink
        let root_span = format!(" {} ", self.root_name);
        let root_start = col + 1; // after the leading space
        let root_end = root_start + self.root_name.width() as u16;
        col += root_span.width() as u16;
        spans.push(Span::styled(root_span, style));
        col += 2;
        spans.push(Span::styled("│ ", style));

        // Selected file path
        let mut sel_start: u16 = 0;
        let mut sel_end: u16 = 0;
        if let Some(sel_path) = self.selected_path {
            let sel_span = format!(" {sel_path} ");
            sel_start = col + 1;
            sel_end = sel_start + sel_path.width() as u16;
            col += sel_span.width() as u16;
            spans.push(Span::styled(sel_span, style));
            spans.push(Span::styled("│ ", style));
        }
        let _ = col; // suppress unused warning

        // File/dir counts
        spans.push(Span::styled(
            format!(" {} files  {} dirs", self.file_count, self.dir_count),
            style,
        ));

        // cmux indicator
        if let Some(status) = self.cmux_status {
            spans.push(Span::styled(" │ ", style));
            spans.push(Span::styled(
                format!(" {status} "),
                Style::default()
                    .fg(colors::GIT_ADDED)
                    .bg(colors::STATUS_BAR_BG),
            ));
        }

        let line = Line::from(spans);
        line.render(area, buf);

        // Apply OSC 8 hyperlinks by embedding escape sequences in cell symbols
        let root_url = format!("file://{}", self.root_path);
        apply_osc8_hyperlink(buf, area.x, area.y, root_start, root_end, &root_url);

        if let Some(abs_path) = self.selected_abs_path {
            if sel_end > sel_start {
                let sel_url = format!("file://{abs_path}");
                apply_osc8_hyperlink(buf, area.x, area.y, sel_start, sel_end, &sel_url);
            }
        }
    }
}

/// Embed OSC 8 hyperlink escape sequences into buffer cells.
/// Prepends the OSC 8 open to the first cell's symbol and appends the close to the last cell's.
fn apply_osc8_hyperlink(
    buf: &mut Buffer,
    base_x: u16,
    y: u16,
    start_col: u16,
    end_col: u16,
    url: &str,
) {
    if start_col >= end_col {
        return;
    }
    let first_x = base_x + start_col;
    let last_x = base_x + end_col - 1;

    // Prepend OSC 8 open to first cell
    if let Some(cell) = buf.cell_mut((first_x, y)) {
        let old = cell.symbol().to_string();
        cell.set_symbol(&format!("\x1b]8;;{url}\x07{old}"));
    }

    // Append OSC 8 close to last cell
    if let Some(cell) = buf.cell_mut((last_x, y)) {
        let old = cell.symbol().to_string();
        cell.set_symbol(&format!("{old}\x1b]8;;\x07"));
    }
}
