use pulldown_cmark::{Alignment, Event, HeadingLevel, Options, Parser, Tag, TagEnd};
use ratatui::style::{Color, Modifier, Style};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use crate::render::{colors, text_util::truncate_with_ellipsis};

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
    list_item_prefixes: Vec<Vec<StyledSpan>>,
    in_code_block: bool,
    code_lang: Option<String>,
    code_buf: String,
    in_heading: bool,
    blockquote_depth: usize,
    in_image: bool,
    in_table: bool,
    table_alignments: Vec<pulldown_cmark::Alignment>,
    table_head: Vec<String>,      // single header row: cell texts
    table_rows: Vec<Vec<String>>, // body rows: each is a list of cell texts
    current_table_row: Vec<String>,
    current_cell_text: String,
    line_has_text_content: bool,
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
            list_item_prefixes: Vec::new(),
            in_code_block: false,
            code_lang: None,
            code_buf: String::new(),
            in_heading: false,
            blockquote_depth: 0,
            in_image: false,
            in_table: false,
            table_alignments: Vec::new(),
            table_head: Vec::new(),
            table_rows: Vec::new(),
            current_table_row: Vec::new(),
            current_cell_text: String::new(),
            line_has_text_content: false,
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
        self.trim_trailing_space();
        let line = std::mem::take(&mut self.current_line);
        self.lines.push(line);
        self.line_has_text_content = false;
    }

    fn flush_line_if_not_empty(&mut self) {
        if !self.current_line.is_empty() {
            self.flush_line();
        }
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

    fn current_line_width(&self) -> usize {
        line_width(&self.current_line)
    }

    fn trim_trailing_space(&mut self) {
        if let Some((text, _)) = self.current_line.last_mut() {
            while text.ends_with(' ') {
                text.pop();
            }
            if text.is_empty() {
                self.current_line.pop();
            }
        }
    }

    fn blockquote_prefix(&self) -> Vec<StyledSpan> {
        if self.blockquote_depth == 0 {
            return Vec::new();
        }

        vec![(
            "│ ".repeat(self.blockquote_depth),
            Style::default().fg(Color::DarkGray),
        )]
    }

    fn active_line_prefix(&self) -> Vec<StyledSpan> {
        self.list_item_prefixes
            .last()
            .cloned()
            .unwrap_or_else(|| self.blockquote_prefix())
    }

    fn ensure_line_prefix(&mut self) {
        if self.current_line.is_empty() {
            self.current_line.extend(self.active_line_prefix());
        }
    }

    fn start_wrapped_line(&mut self) {
        self.flush_line();
        self.current_line = self.active_line_prefix();
    }

    fn push_space(&mut self, style: Style) {
        if !self.line_has_text_content {
            return;
        }

        if self.current_line_width() + 1 > self.width {
            self.start_wrapped_line();
            return;
        }

        if self
            .current_line
            .last()
            .is_some_and(|(text, _)| text.ends_with(' '))
        {
            return;
        }

        self.current_line.push((" ".to_string(), style));
    }

    fn push_token(&mut self, token: &str, style: Style) {
        if token.is_empty() {
            return;
        }

        self.ensure_line_prefix();

        if self.line_has_text_content
            && self.current_line_width() + UnicodeWidthStr::width(token) > self.width
        {
            self.start_wrapped_line();
        }

        let mut remaining = token;
        loop {
            let available = self.width.saturating_sub(self.current_line_width());
            if available == 0 {
                self.start_wrapped_line();
                continue;
            }

            let end = prefix_fitting_width(remaining, available);
            if end == 0 {
                break;
            }

            let chunk = &remaining[..end];
            self.current_line.push((chunk.to_string(), style));
            self.line_has_text_content = true;
            remaining = &remaining[end..];

            if remaining.is_empty() {
                break;
            }

            self.start_wrapped_line();
        }
    }

    fn push_wrapped_text(&mut self, text: &str, style: Style) {
        if text.is_empty() {
            return;
        }

        let mut run = String::new();
        let mut run_is_whitespace: Option<bool> = None;

        for ch in text.chars() {
            let is_whitespace = ch.is_whitespace();
            match run_is_whitespace {
                Some(state) if state == is_whitespace => run.push(ch),
                Some(state) => {
                    if state {
                        self.push_space(style);
                    } else {
                        self.push_token(&run, style);
                    }
                    run.clear();
                    run.push(ch);
                    run_is_whitespace = Some(is_whitespace);
                }
                None => {
                    run.push(ch);
                    run_is_whitespace = Some(is_whitespace);
                }
            }
        }

        if !run.is_empty() {
            if run_is_whitespace == Some(true) {
                self.push_space(style);
            } else {
                self.push_token(&run, style);
            }
        }
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
                // Allow blockquote end to pass through even inside a code block
                // so that malformed markdown (unclosed code fence) doesn't corrupt
                // the blockquote depth and style stack permanently.
                Event::End(TagEnd::BlockQuote(_)) => {
                    if self.blockquote_depth > 0 {
                        self.blockquote_depth -= 1;
                        self.pop_style();
                    }
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
            Event::SoftBreak => self.push_wrapped_text(" ", self.current_style()),
            Event::HardBreak => {
                self.flush_line();
            }
            Event::Rule => {
                self.flush_line();
                let rule: String = "─".repeat(self.width.min(80));
                self.current_line
                    .push((rule, Style::default().fg(Color::DarkGray)));
                self.line_has_text_content = true;
                self.flush_line();
            }
            Event::TaskListMarker(checked) => {
                let marker = if checked { "[x] " } else { "[ ] " };
                let style = self.current_style();
                self.current_line.push((marker.to_string(), style));
                if !self.line_has_text_content {
                    if let Some(prefix) = self.list_item_prefixes.last_mut() {
                        prefix.push((" ".repeat(UnicodeWidthStr::width(marker)), style));
                    }
                }
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
                self.blockquote_depth += 1;
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
                self.flush_line_if_not_empty();
                self.current_line.extend(self.blockquote_prefix());
                let indent = self.list_indent();
                let marker = match self.list_stack.last() {
                    Some(Some(n)) => {
                        let s = format!("{indent}{n}. ");
                        if let Some(Some(ref mut counter)) = self.list_stack.last_mut() {
                            *counter += 1;
                        }
                        s
                    }
                    _ => format!("{indent}• "),
                };
                let marker_style = self.current_style();
                let marker_width = UnicodeWidthStr::width(marker.as_str());
                self.current_line.push((marker, self.current_style()));
                let mut continuation = self.blockquote_prefix();
                continuation.push((" ".repeat(marker_width), marker_style));
                self.list_item_prefixes.push(continuation);
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
                self.in_image = true;
                self.push_wrapped_text(
                    &format!("[image: {dest_url}]"),
                    Style::default()
                        .fg(Color::DarkGray)
                        .add_modifier(Modifier::ITALIC),
                );
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
                if self.blockquote_depth > 0 {
                    self.blockquote_depth -= 1;
                    self.pop_style();
                }
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
                self.flush_line_if_not_empty();
                self.list_item_prefixes.pop();
            }
            TagEnd::Emphasis | TagEnd::Strong | TagEnd::Strikethrough => {
                self.pop_style();
            }
            TagEnd::Image => {
                self.in_image = false;
            }
            TagEnd::Link => {
                self.pop_style();
                if let Some(url) = self.link_url.take() {
                    self.push_wrapped_text(
                        &format!(" ({url})"),
                        Style::default().fg(Color::DarkGray),
                    );
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
        // Suppress alt-text inside image tags (already rendered as [image: url])
        if self.in_image {
            return;
        }
        if self.in_table {
            self.current_cell_text.push_str(text);
            return;
        }
        self.push_wrapped_text(text, self.current_style());
    }

    fn inline_code(&mut self, code: &str) {
        if self.in_table {
            self.current_cell_text.push('`');
            self.current_cell_text.push_str(code);
            self.current_cell_text.push('`');
            return;
        }
        let style = Style::default().fg(colors::inline_code());
        self.push_wrapped_text(&format!("`{code}`"), style);
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
            line.extend(self.blockquote_prefix());
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
            col_widths[c] = col_widths[c].max(UnicodeWidthStr::width(cell.as_str()));
        }
        for row in body {
            for (c, cell) in row.iter().enumerate() {
                col_widths[c] = col_widths[c].max(UnicodeWidthStr::width(cell.as_str()));
            }
        }

        // Clamp total width
        let total: usize = col_widths.iter().sum::<usize>() + (num_cols + 1) * 3;
        if total > self.width {
            let available = self.width.saturating_sub((num_cols + 1) * 3);
            let per_col = available / num_cols.max(1);
            if per_col == 0 {
                // Terminal too narrow to render table — show fallback message
                self.lines.push(vec![(
                    "(table too wide to display)".to_string(),
                    Style::default().fg(Color::DarkGray),
                )]);
                return;
            }
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
            let line = format_table_row(
                head,
                &col_widths,
                &self.table_alignments,
                head_style,
                border_style,
            );
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
            let line = format_table_row(
                row,
                &col_widths,
                &self.table_alignments,
                Style::default(),
                border_style,
            );
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
    alignments: &[Alignment],
    text_style: Style,
    border_style: Style,
) -> Vec<StyledSpan> {
    let mut line: Vec<StyledSpan> = Vec::new();
    for (c, w) in col_widths.iter().enumerate() {
        line.push(("│ ".to_string(), border_style));
        let content = if c < row.len() { &row[c] } else { "" };
        let truncated = truncate_with_ellipsis(content, *w);
        let display_w = UnicodeWidthStr::width(truncated.as_str());
        let padding = w.saturating_sub(display_w);
        let (left_pad, right_pad) = match alignments.get(c).copied().unwrap_or(Alignment::None) {
            Alignment::Right => (padding, 0),
            Alignment::Center => (padding / 2, padding - (padding / 2)),
            Alignment::Left | Alignment::None => (0, padding),
        };
        let padded = format!(
            "{}{}{}",
            " ".repeat(left_pad),
            truncated,
            " ".repeat(right_pad)
        );
        line.push((padded, text_style));
        line.push((" ".to_string(), border_style));
    }
    line.push(("│".to_string(), border_style));
    line
}

fn line_width(line: &[StyledSpan]) -> usize {
    line.iter()
        .map(|(text, _)| UnicodeWidthStr::width(text.as_str()))
        .sum()
}

fn prefix_fitting_width(text: &str, max_width: usize) -> usize {
    let mut width = 0;
    let mut end = 0;
    for ch in text.chars() {
        let char_width = UnicodeWidthChar::width(ch).unwrap_or(0);
        if width + char_width > max_width {
            break;
        }
        width += char_width;
        end += ch.len_utf8();
    }
    end
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lines_to_text(lines: &[Vec<StyledSpan>]) -> Vec<String> {
        lines
            .iter()
            .map(|spans| spans.iter().map(|(text, _)| text.as_str()).collect())
            .collect()
    }

    fn line_width(line: &[StyledSpan]) -> usize {
        line.iter()
            .map(|(text, _)| UnicodeWidthStr::width(text.as_str()))
            .sum()
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
    fn paragraph_wraps_to_preview_width() {
        let result = render_markdown("alpha beta gamma delta", 20);
        let text = lines_to_text(&result);
        let non_empty: Vec<_> = text.into_iter().filter(|line| !line.is_empty()).collect();

        assert_eq!(non_empty, vec!["alpha beta gamma", "delta"]);
        assert!(result.iter().all(|line| line_width(line) <= 20));
    }

    #[test]
    fn list_item_wraps_with_hanging_indent() {
        let result = render_markdown("- alpha beta gamma delta", 20);
        let text = lines_to_text(&result);
        let non_empty: Vec<_> = text.into_iter().filter(|line| !line.is_empty()).collect();

        assert_eq!(non_empty, vec!["• alpha beta gamma", "  delta"]);
        assert!(result.iter().all(|line| line_width(line) <= 20));
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

    #[test]
    fn narrow_tables_truncate_cells_to_width() {
        let md = "| Column | Value |\n|---|---|\n| SuperLongCell | 123456 |";
        let result = render_markdown(md, 18);
        let text = lines_to_text(&result);

        assert!(text.iter().any(|line| line.contains('…')));
        assert!(result.iter().all(|line| line_width(line) <= 18));
    }

    #[test]
    fn image_alt_text_not_duplicated() {
        let md = "![photo of sunset](sunset.png)";
        let result = render_markdown(md, 80);
        let text = lines_to_text(&result);
        let full_text: String = text.join(" ");
        // Should contain the [image: url] placeholder
        assert!(
            full_text.contains("[image: sunset.png]"),
            "should show image placeholder: {full_text}"
        );
        // Alt-text "photo of sunset" should NOT appear as separate text
        assert!(
            !full_text.contains("photo of sunset"),
            "alt-text should be suppressed: {full_text}"
        );
    }
}
