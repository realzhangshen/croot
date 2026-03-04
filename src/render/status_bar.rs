use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::Widget,
};

use super::theme::Theme;

pub struct StatusBar<'a> {
    pub branch: Option<&'a str>,
    pub file_count: usize,
    pub dir_count: usize,
    pub root_name: &'a str,
    pub cmux_status: Option<&'a str>,
    pub theme: &'a Theme,
}

impl Widget for StatusBar<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let style = Style::default()
            .fg(self.theme.status_bar_fg)
            .bg(self.theme.status_bar_bg);

        // Fill background
        for x in area.x..area.x + area.width {
            if let Some(cell) = buf.cell_mut((x, area.y)) {
                cell.set_style(style);
            }
        }

        let mut spans = Vec::new();

        // Branch info
        if let Some(branch) = self.branch {
            spans.push(Span::styled(format!("  {branch} "), style));
            spans.push(Span::styled("│ ", style));
        }

        // Root name
        spans.push(Span::styled(format!(" {} ", self.root_name), style));
        spans.push(Span::styled("│ ", style));

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
                    .fg(self.theme.git_added)
                    .bg(self.theme.status_bar_bg),
            ));
        }

        let line = Line::from(spans);
        line.render(area, buf);
    }
}
