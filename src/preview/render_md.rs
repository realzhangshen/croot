use pulldown_cmark::{Event, HeadingLevel, Options, Parser, Tag, TagEnd};
use ratatui::style::{Color, Modifier, Style};

use crate::render::colors;

use super::highlight;
use super::state::StyledSpan;

/// Render Markdown source into pre-styled lines.
pub fn render_markdown(source: &str, width: usize) -> Vec<Vec<StyledSpan>> {
    let mut renderer = MdRenderer::new(width);
    let opts = Options::ENABLE_TABLES
        | Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_TASKLISTS
        | Options::ENABLE_HEADING_ATTRIBUTES;
    let parser = Parser::new_ext(source, opts);

    for event in parser {
        renderer.process(event);
    }

    renderer.finish()
}

#[allow(clippy::struct_excessive_bools)]
struct MdRenderer {
    lines: Vec<Vec<StyledSpan>>,
    current_line: Vec<StyledSpan>,
    style_stack: Vec<Style>,
    list_stack: Vec<Option<u64>>, // None = unordered, Some(n) = ordered starting at n
    in_code_block: bool,
    code_lang: Option<String>,
    code_buf: String,
    in_heading: bool,
    in_blockquote: bool,
    in_table: bool,
    table_alignments: Vec<pulldown_cmark::Alignment>,
    table_head: Vec<String>,      // single header row: cell texts
    table_rows: Vec<Vec<String>>, // body rows: each is a list of cell texts
    current_table_row: Vec<String>,
    current_cell_text: String,
    link_url: Option<String>,
    width: usize,
}

impl MdRenderer {
    fn new(width: usize) -> Self {
        Self {
            lines: Vec::new(),
            current_line: Vec::new(),
            style_stack: vec![Style::default()],
            list_stack: Vec::new(),
            in_code_block: false,
            code_lang: None,
            code_buf: String::new(),
            in_heading: false,
            in_blockquote: false,
            in_table: false,
            table_alignments: Vec::new(),
            table_head: Vec::new(),
            table_rows: Vec::new(),
            current_table_row: Vec::new(),
            current_cell_text: String::new(),
            link_url: None,
            width: width.max(20),
        }
    }

    fn current_style(&self) -> Style {
        self.style_stack.last().copied().unwrap_or_default()
    }

    fn push_style(&mut self, style: Style) {
        let base = self.current_style();
        let merged = merge_styles(base, style);
        self.style_stack.push(merged);
    }

    fn pop_style(&mut self) {
        if self.style_stack.len() > 1 {
            self.style_stack.pop();
        }
    }

    fn flush_line(&mut self) {
        let line = std::mem::take(&mut self.current_line);
        self.lines.push(line);
    }

    fn push_blank_line(&mut self) {
        if !self.in_table {
            self.lines.push(Vec::new());
        }
    }

    fn list_indent(&self) -> String {
        let depth = self.list_stack.len().saturating_sub(1);
        "  ".repeat(depth)
    }

    fn process(&mut self, event: Event) {
        if self.in_code_block {
            match event {
                Event::Text(text) => {
                    self.code_buf.push_str(&text);
                }
                Event::End(TagEnd::CodeBlock) => {
                    self.end_code_block();
                }
                _ => {}
            }
            return;
        }

        match event {
            Event::Start(tag) => self.start_tag(tag),
            Event::End(tag_end) => self.end_tag(tag_end),
            Event::Text(text) => self.text(&text),
            Event::Code(code) => self.inline_code(&code),
            Event::SoftBreak => {
                self.current_line
                    .push((" ".to_string(), self.current_style()));
            }
            Event::HardBreak => {
                self.flush_line();
            }
            Event::Rule => {
                self.flush_line();
                let rule: String = "─".repeat(self.width.min(80));
                self.current_line
                    .push((rule, Style::default().fg(Color::DarkGray)));
                self.flush_line();
            }
            Event::TaskListMarker(checked) => {
                let marker = if checked { "[x] " } else { "[ ] " };
                self.current_line
                    .push((marker.to_string(), self.current_style()));
            }
            _ => {}
        }
    }

    fn start_tag(&mut self, tag: Tag) {
        match tag {
            Tag::Heading { level, .. } => {
                self.in_heading = true;
                let style = heading_style(level);
                self.push_style(style);
            }
            Tag::Paragraph => {}
            Tag::BlockQuote(_) => {
                self.in_blockquote = true;
                self.push_style(Style::default().fg(Color::DarkGray));
            }
            Tag::CodeBlock(kind) => {
                let lang = match kind {
                    pulldown_cmark::CodeBlockKind::Fenced(lang) => {
                        let l = lang.to_string();
                        if l.is_empty() {
                            None
                        } else {
                            Some(l)
                        }
                    }
                    pulldown_cmark::CodeBlockKind::Indented => None,
                };
                self.in_code_block = true;
                self.code_lang = lang;
                self.code_buf.clear();
            }
            Tag::List(start) => {
                self.list_stack.push(start);
            }
            Tag::Item => {
                self.flush_line();
                let indent = self.list_indent();
                let marker = match self.list_stack.last() {
                    Some(Some(n)) => {
                        let s = format!("{indent}{n}. ");
                        // Increment the counter
                        if let Some(Some(ref mut counter)) = self.list_stack.last_mut() {
                            *counter += 1;
                        }
                        s
                    }
                    _ => format!("{indent}• "),
                };
                self.current_line.push((marker, self.current_style()));
            }
            Tag::Emphasis => {
                self.push_style(Style::default().add_modifier(Modifier::ITALIC));
            }
            Tag::Strong => {
                self.push_style(Style::default().add_modifier(Modifier::BOLD));
            }
            Tag::Strikethrough => {
                self.push_style(Style::default().add_modifier(Modifier::CROSSED_OUT));
            }
            Tag::Link { dest_url, .. } => {
                self.push_style(
                    Style::default()
                        .fg(Color::Blue)
                        .add_modifier(Modifier::UNDERLINED),
                );
                self.link_url = Some(dest_url.to_string());
            }
            Tag::Image { dest_url, .. } => {
                self.current_line.push((
                    format!("[image: {dest_url}]"),
                    Style::default()
                        .fg(Color::DarkGray)
                        .add_modifier(Modifier::ITALIC),
                ));
            }
            Tag::Table(alignments) => {
                self.in_table = true;
                self.table_alignments = alignments;
                self.table_head.clear();
                self.table_rows.clear();
                self.current_table_row.clear();
            }
            Tag::TableHead => {
                self.current_table_row.clear();
            }
            Tag::TableRow => {
                self.current_table_row.clear();
            }
            Tag::TableCell => {
                self.current_cell_text.clear();
            }
            _ => {}
        }
    }

    fn end_tag(&mut self, tag_end: TagEnd) {
        match tag_end {
            TagEnd::Heading(_) => {
                self.in_heading = false;
                self.flush_line();
                self.pop_style();
                self.push_blank_line();
            }
            TagEnd::Paragraph => {
                self.flush_line();
                self.push_blank_line();
            }
            TagEnd::BlockQuote(_) => {
                self.in_blockquote = false;
                self.pop_style();
            }
            TagEnd::CodeBlock => {
                // handled in process() directly
            }
            TagEnd::List(_) => {
                self.list_stack.pop();
                if self.list_stack.is_empty() {
                    self.push_blank_line();
                }
            }
            TagEnd::Item => {
                self.flush_line();
            }
            TagEnd::Emphasis | TagEnd::Strong | TagEnd::Strikethrough => {
                self.pop_style();
            }
            TagEnd::Link => {
                self.pop_style();
                if let Some(url) = self.link_url.take() {
                    self.current_line
                        .push((format!(" ({url})"), Style::default().fg(Color::DarkGray)));
                }
            }
            TagEnd::Table => {
                self.render_table();
                self.in_table = false;
            }
            TagEnd::TableHead => {
                self.table_head = std::mem::take(&mut self.current_table_row);
            }
            TagEnd::TableRow => {
                let row = std::mem::take(&mut self.current_table_row);
                self.table_rows.push(row);
            }
            TagEnd::TableCell => {
                let text = std::mem::take(&mut self.current_cell_text);
                self.current_table_row.push(text);
            }
            _ => {}
        }
    }

    fn text(&mut self, text: &str) {
        if self.in_table {
            self.current_cell_text.push_str(text);
            return;
        }
        if self.in_blockquote && self.current_line.is_empty() {
            self.current_line
                .push(("│ ".to_string(), Style::default().fg(Color::DarkGray)));
        }
        self.current_line
            .push((text.to_string(), self.current_style()));
    }

    fn inline_code(&mut self, code: &str) {
        if self.in_table {
            self.current_cell_text.push('`');
            self.current_cell_text.push_str(code);
            self.current_cell_text.push('`');
            return;
        }
        let style = Style::default().fg(colors::inline_code());
        self.current_line.push((format!("`{code}`"), style));
    }

    fn end_code_block(&mut self) {
        self.in_code_block = false;
        let code = std::mem::take(&mut self.code_buf);
        let lang = self.code_lang.take();

        let border_style = Style::default().fg(Color::DarkGray);
        let highlighted = match lang.as_deref() {
            Some(l) if !l.is_empty() => highlight::highlight_code(l, &code, 10_000),
            _ => code
                .lines()
                .map(|line| vec![(line.to_string(), Style::default())])
                .collect(),
        };

        for hl_line in &highlighted {
            let mut line: Vec<StyledSpan> = Vec::new();
            line.push(("│ ".to_string(), border_style));
            for span in hl_line {
                line.push(span.clone());
            }
            self.lines.push(line);
        }
        self.push_blank_line();
    }

    fn render_table(&mut self) {
        let head = &self.table_head;
        let body = &self.table_rows;

        // Collect all rows to compute column widths
        let num_cols = head.len().max(body.iter().map(Vec::len).max().unwrap_or(0));
        if num_cols == 0 {
            return;
        }

        let mut col_widths = vec![0usize; num_cols];
        for (c, cell) in head.iter().enumerate() {
            col_widths[c] = col_widths[c].max(cell.len());
        }
        for row in body {
            for (c, cell) in row.iter().enumerate() {
                col_widths[c] = col_widths[c].max(cell.len());
            }
        }

        // Clamp total width
        let total: usize = col_widths.iter().sum::<usize>() + (num_cols + 1) * 3;
        if total > self.width {
            let available = self.width.saturating_sub((num_cols + 1) * 3);
            let per_col = available / num_cols.max(1);
            for w in &mut col_widths {
                if *w > per_col {
                    *w = per_col;
                }
            }
        }

        let border_style = Style::default().fg(Color::DarkGray);
        let head_style = Style::default()
            .add_modifier(Modifier::BOLD)
            .fg(Color::Cyan);

        // Render header
        if !head.is_empty() {
            let line = format_table_row(head, &col_widths, head_style, border_style);
            self.lines.push(line);
        }

        // Separator
        let mut sep_line: Vec<StyledSpan> = Vec::new();
        for (c, &w) in col_widths.iter().enumerate() {
            if c == 0 {
                sep_line.push(("├".to_string(), border_style));
            } else {
                sep_line.push(("┼".to_string(), border_style));
            }
            sep_line.push(("─".repeat(w + 2), border_style));
        }
        sep_line.push(("┤".to_string(), border_style));
        self.lines.push(sep_line);

        // Render body rows
        for row in body {
            let line = format_table_row(row, &col_widths, Style::default(), border_style);
            self.lines.push(line);
        }

        self.push_blank_line();
    }

    fn finish(mut self) -> Vec<Vec<StyledSpan>> {
        if !self.current_line.is_empty() {
            self.flush_line();
        }
        self.lines
    }
}

fn heading_style(level: HeadingLevel) -> Style {
    match level {
        HeadingLevel::H1 => Style::default()
            .fg(Color::Blue)
            .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
        HeadingLevel::H2 => Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
        HeadingLevel::H3 => Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
        _ => Style::default()
            .fg(Color::Magenta)
            .add_modifier(Modifier::BOLD),
    }
}

fn merge_styles(base: Style, overlay: Style) -> Style {
    let mut s = base;
    if let Some(fg) = overlay.fg {
        s = s.fg(fg);
    }
    if let Some(bg) = overlay.bg {
        s = s.bg(bg);
    }
    s = s.add_modifier(overlay.add_modifier);
    s
}

fn format_table_row(
    row: &[String],
    col_widths: &[usize],
    text_style: Style,
    border_style: Style,
) -> Vec<StyledSpan> {
    let mut line: Vec<StyledSpan> = Vec::new();
    for (c, w) in col_widths.iter().enumerate() {
        line.push(("│ ".to_string(), border_style));
        let content = if c < row.len() { &row[c] } else { "" };
        let padded = format!("{content:<w$}");
        line.push((padded, text_style));
        line.push((" ".to_string(), border_style));
    }
    line.push(("│".to_string(), border_style));
    line
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Extract the plain text from styled lines.
    fn lines_to_text(lines: &[Vec<StyledSpan>]) -> Vec<String> {
        lines
            .iter()
            .map(|spans| spans.iter().map(|(text, _)| text.as_str()).collect())
            .collect()
    }

    #[test]
    fn empty_input_returns_empty() {
        let result = render_markdown("", 80);
        assert!(result.is_empty() || result.iter().all(Vec::is_empty));
    }

    #[test]
    fn heading_produces_styled_output() {
        let result = render_markdown("# Hello", 80);
        let text = lines_to_text(&result);
        assert!(
            text.iter().any(|l| l.contains("Hello")),
            "Heading text missing: {text:?}"
        );

        // Check H1 style: should be Blue + Bold + Underlined
        let heading_line = &result[0];
        let (_, style) = heading_line
            .iter()
            .find(|(t, _)| t.contains("Hello"))
            .unwrap();
        assert_eq!(style.fg, Some(Color::Blue));
        assert!(style.add_modifier.contains(Modifier::BOLD));
    }

    #[test]
    fn h2_and_h3_headings() {
        let result = render_markdown("## Sub\n### Detail", 80);
        let text = lines_to_text(&result);
        assert!(text.iter().any(|l| l.contains("Sub")));
        assert!(text.iter().any(|l| l.contains("Detail")));
    }

    #[test]
    fn bold_and_italic_text() {
        let result = render_markdown("**bold** and *italic*", 80);
        let all_spans: Vec<&StyledSpan> = result.iter().flat_map(|l| l.iter()).collect();

        let bold_span = all_spans
            .iter()
            .find(|(t, _)| t.contains("bold"))
            .expect("bold text missing");
        assert!(bold_span.1.add_modifier.contains(Modifier::BOLD));

        let italic_span = all_spans
            .iter()
            .find(|(t, _)| t.contains("italic"))
            .expect("italic text missing");
        assert!(italic_span.1.add_modifier.contains(Modifier::ITALIC));
    }

    #[test]
    fn inline_code_rendered() {
        let result = render_markdown("Use `foo()` here", 80);
        let text = lines_to_text(&result);
        assert!(
            text.iter().any(|l| l.contains("`foo()`")),
            "Inline code missing: {text:?}"
        );
    }

    #[test]
    fn code_block_rendered() {
        let md = "```rust\nlet x = 42;\n```";
        let result = render_markdown(md, 80);
        let text = lines_to_text(&result);
        assert!(
            text.iter().any(|l| l.contains("let x = 42;")),
            "Code block content missing: {text:?}"
        );
    }

    #[test]
    fn code_block_without_language() {
        let md = "```\nplain code\n```";
        let result = render_markdown(md, 80);
        let text = lines_to_text(&result);
        assert!(text.iter().any(|l| l.contains("plain code")));
    }

    #[test]
    fn unordered_list() {
        let md = "- item one\n- item two\n- item three";
        let result = render_markdown(md, 80);
        let text = lines_to_text(&result);
        assert!(text.iter().any(|l| l.contains("item one")));
        assert!(text.iter().any(|l| l.contains("item two")));
        // Check bullet marker
        assert!(
            text.iter().any(|l| l.contains('•')),
            "Bullet marker missing: {text:?}"
        );
    }

    #[test]
    fn ordered_list() {
        let md = "1. first\n2. second\n3. third";
        let result = render_markdown(md, 80);
        let text = lines_to_text(&result);
        assert!(text.iter().any(|l| l.contains("1.")));
        assert!(text.iter().any(|l| l.contains("first")));
    }

    #[test]
    fn nested_list() {
        let md = "- outer\n  - inner\n    - deep";
        let result = render_markdown(md, 80);
        let text = lines_to_text(&result);
        assert!(text.iter().any(|l| l.contains("outer")));
        assert!(text.iter().any(|l| l.contains("inner")));
        assert!(text.iter().any(|l| l.contains("deep")));
    }

    #[test]
    fn horizontal_rule() {
        let md = "above\n\n---\n\nbelow";
        let result = render_markdown(md, 80);
        let text = lines_to_text(&result);
        assert!(text.iter().any(|l| l.contains('─')));
    }

    #[test]
    fn link_rendered_with_url() {
        let md = "[click here](https://example.com)";
        let result = render_markdown(md, 80);
        let text = lines_to_text(&result);
        assert!(text.iter().any(|l| l.contains("click here")));
        assert!(text.iter().any(|l| l.contains("https://example.com")));
    }

    #[test]
    fn blockquote_prefixed() {
        let md = "> quoted text";
        let result = render_markdown(md, 80);
        let text = lines_to_text(&result);
        assert!(
            text.iter().any(|l| l.contains('│')),
            "Blockquote prefix missing: {text:?}"
        );
        assert!(text.iter().any(|l| l.contains("quoted text")));
    }

    #[test]
    fn strikethrough_text() {
        let md = "~~deleted~~";
        let result = render_markdown(md, 80);
        let all_spans: Vec<&StyledSpan> = result.iter().flat_map(|l| l.iter()).collect();
        let span = all_spans
            .iter()
            .find(|(t, _)| t.contains("deleted"))
            .expect("strikethrough text missing");
        assert!(span.1.add_modifier.contains(Modifier::CROSSED_OUT));
    }

    #[test]
    fn table_renders_with_borders() {
        let md = "| A | B |\n|---|---|\n| 1 | 2 |";
        let result = render_markdown(md, 80);
        let text = lines_to_text(&result);
        assert!(text.iter().any(|l| l.contains('│')));
        assert!(text.iter().any(|l| l.contains('A')));
        assert!(text.iter().any(|l| l.contains('1')));
    }
}
