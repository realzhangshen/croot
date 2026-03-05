use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
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

pub struct HyperlinkRegion {
    pub x: u16,
    pub y: u16,
    pub text: String,
    pub url: String,
}

impl StatusBar<'_> {
    /// Compute hyperlink regions for post-render OSC 8 emission.
    pub fn hyperlink_regions(&self, area: Rect) -> Vec<HyperlinkRegion> {
        let mut regions = Vec::new();
        let mut col: u16 = 0;

        // Branch info
        if let Some(branch) = self.branch {
            let s = format!("  {branch} ");
            col += s.width() as u16;
            col += 2; // "│ "
        }

        // Root name
        let root_span = format!(" {} ", self.root_name);
        let root_start = col + 1; // after the leading space
        col += root_span.width() as u16;
        col += 2; // "│ "

        regions.push(HyperlinkRegion {
            x: area.x + root_start,
            y: area.y,
            text: self.root_name.to_string(),
            url: format!("file://{}", self.root_path),
        });

        // Selected file path
        if let (Some(sel_path), Some(abs_path)) = (self.selected_path, self.selected_abs_path) {
            let sel_span = format!(" {sel_path} ");
            let sel_start = col + 1;
            col += sel_span.width() as u16;
            let _ = col;

            regions.push(HyperlinkRegion {
                x: area.x + sel_start,
                y: area.y,
                text: sel_path.to_string(),
                url: format!("file://{abs_path}"),
            });
        }

        regions
    }
}

impl Widget for StatusBar<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let style = Style::default().add_modifier(Modifier::REVERSED);

        // Fill background
        for x in area.x..area.x + area.width {
            if let Some(cell) = buf.cell_mut((x, area.y)) {
                cell.set_style(style);
            }
        }

        let mut spans = Vec::new();

        // Branch info
        if let Some(branch) = self.branch {
            let s = format!("  {branch} ");
            spans.push(Span::styled(s, style));
            spans.push(Span::styled("│ ", style));
        }

        // Root name
        let root_span = format!(" {} ", self.root_name);
        spans.push(Span::styled(root_span, style));
        spans.push(Span::styled("│ ", style));

        // Selected file path
        if let Some(sel_path) = self.selected_path {
            let sel_span = format!(" {sel_path} ");
            spans.push(Span::styled(sel_span, style));
            spans.push(Span::styled("│ ", style));
        }

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
                    .add_modifier(Modifier::REVERSED),
            ));
        }

        let line = Line::from(spans);
        line.render(area, buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::buffer::Buffer;
    use ratatui::layout::Rect;

    fn make_status_bar<'a>(
        branch: Option<&'a str>,
        root_name: &'a str,
        root_path: &'a str,
        selected_path: Option<&'a str>,
        selected_abs_path: Option<&'a str>,
        file_count: usize,
        dir_count: usize,
    ) -> StatusBar<'a> {
        StatusBar {
            branch,
            file_count,
            dir_count,
            root_name,
            root_path,
            cmux_status: None,
            selected_path,
            selected_abs_path,
        }
    }

    fn render_to_buffer(bar: StatusBar, width: u16) -> Buffer {
        let area = Rect::new(0, 0, width, 1);
        let mut buf = Buffer::empty(area);
        bar.render(area, &mut buf);
        buf
    }

    #[test]
    fn test_cells_have_no_embedded_escape_sequences() {
        let bar = make_status_bar(
            Some("main"),
            "croot",
            "/home/user/croot",
            Some("src/app.rs"),
            Some("/home/user/croot/src/app.rs"),
            42,
            8,
        );
        let buf = render_to_buffer(bar, 80);

        for x in 0..80 {
            let cell = buf.cell((x, 0)).unwrap();
            let sym = cell.symbol();
            assert!(
                !sym.contains("\x1b]8"),
                "Cell at x={x} contains OSC 8 sequence: {sym:?}"
            );
            assert!(
                sym.width() <= 1,
                "Cell at x={x} has unicode width {}: {sym:?}",
                sym.width()
            );
        }
    }

    #[test]
    fn test_status_bar_uses_reversed_style() {
        let bar = make_status_bar(Some("main"), "croot", "/tmp", None, None, 0, 0);
        let buf = render_to_buffer(bar, 80);

        // Check a few cells have REVERSED modifier
        for x in 0..5 {
            let cell = buf.cell((x, 0)).unwrap();
            assert!(
                cell.modifier.contains(Modifier::REVERSED),
                "Cell at x={x} missing REVERSED modifier"
            );
        }
    }

    #[test]
    fn test_status_bar_content_with_branch() {
        let bar = make_status_bar(
            Some("main"),
            "croot",
            "/tmp",
            Some("src/app.rs"),
            Some("/tmp/src/app.rs"),
            42,
            8,
        );
        let buf = render_to_buffer(bar, 80);

        let text: String = (0..80).map(|x| buf.cell((x, 0)).unwrap().symbol().to_string()).collect();
        assert!(text.contains("main"), "Missing branch name in: {text:?}");
        assert!(text.contains("croot"), "Missing root name in: {text:?}");
        assert!(text.contains("src/app.rs"), "Missing selected path in: {text:?}");
        assert!(text.contains("42 files"), "Missing file count in: {text:?}");
        assert!(text.contains("8 dirs"), "Missing dir count in: {text:?}");
        assert!(text.contains("│"), "Missing separator in: {text:?}");
    }

    #[test]
    fn test_status_bar_content_no_branch() {
        let bar = make_status_bar(None, "myproject", "/tmp", None, None, 10, 3);
        let buf = render_to_buffer(bar, 80);

        let text: String = (0..80).map(|x| buf.cell((x, 0)).unwrap().symbol().to_string()).collect();
        // Root name should appear near the start (no branch prefix)
        let root_pos = text.find("myproject").expect("Missing root name");
        assert!(root_pos < 5, "Root should be near start, found at {root_pos}");
    }

    #[test]
    fn test_status_bar_content_no_selected_file() {
        let bar = make_status_bar(Some("main"), "croot", "/tmp", None, None, 5, 2);
        let buf = render_to_buffer(bar, 80);

        let text: String = (0..80).map(|x| buf.cell((x, 0)).unwrap().symbol().to_string()).collect();
        assert!(text.contains("croot"), "Missing root name");
        assert!(text.contains("5 files"), "Missing file count");
        // Should not have extra separators from selected path
        // Count separators: branch│ root│ counts — should have exactly 2
        let sep_count = text.matches('│').count();
        assert_eq!(sep_count, 2, "Expected 2 separators, got {sep_count} in: {text:?}");
    }

    #[test]
    fn test_hyperlink_regions_returned() {
        let bar = make_status_bar(
            Some("main"),
            "croot",
            "/home/user/croot",
            Some("src/app.rs"),
            Some("/home/user/croot/src/app.rs"),
            42,
            8,
        );
        let area = Rect::new(0, 5, 80, 1);
        let regions = bar.hyperlink_regions(area);

        assert_eq!(regions.len(), 2);

        // Root hyperlink
        assert_eq!(regions[0].text, "croot");
        assert_eq!(regions[0].url, "file:///home/user/croot");
        assert_eq!(regions[0].y, 5);
        assert!(regions[0].x > 0, "Root link should not start at column 0");

        // Selected file hyperlink
        assert_eq!(regions[1].text, "src/app.rs");
        assert_eq!(regions[1].url, "file:///home/user/croot/src/app.rs");
        assert_eq!(regions[1].y, 5);
        assert!(regions[1].x > regions[0].x, "Selected link should be after root link");
    }

    #[test]
    fn test_hyperlink_regions_no_selected() {
        let bar = make_status_bar(None, "proj", "/tmp/proj", None, None, 0, 0);
        let area = Rect::new(0, 0, 80, 1);
        let regions = bar.hyperlink_regions(area);

        assert_eq!(regions.len(), 1);
        assert_eq!(regions[0].text, "proj");
    }
}
