use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{StatefulWidget, Widget},
};

use crate::tree::forest::FileTree;
use crate::tree::node::GitStatus;

use super::icons;
use super::theme::Theme;

pub struct TreeView<'a> {
    pub theme: &'a Theme,
}

impl StatefulWidget for TreeView<'_> {
    type State = FileTree;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut FileTree) {
        let height = area.height as usize;
        state.adjust_scroll(height);

        let visible = state.visible_range(height);
        let scroll_offset = state.scroll_offset;

        for (i, node) in visible.iter().enumerate() {
            let y = area.y + i as u16;
            if y >= area.y + area.height {
                break;
            }

            let absolute_idx = scroll_offset + i;
            let is_selected = absolute_idx == state.cursor;

            // Build the line
            let mut spans = Vec::new();
            let bg = if is_selected {
                self.theme.selected_bg
            } else {
                ratatui::style::Color::Reset
            };

            // Tree connectors
            let guides = state.connector_guides(absolute_idx);
            for (d, &has_continuation) in guides.iter().enumerate() {
                if d == 0 && node.depth == 0 {
                    continue;
                }
                let connector = if has_continuation { "│ " } else { "  " };
                spans.push(Span::styled(
                    connector,
                    Style::default().fg(self.theme.tree_line).bg(bg),
                ));
            }

            // Branch connector for this node
            if node.depth > 0 {
                let is_last = state.is_last_sibling(absolute_idx);
                let branch = if is_last { "└─" } else { "├─" };
                spans.push(Span::styled(
                    branch,
                    Style::default().fg(self.theme.tree_line).bg(bg),
                ));
            }

            // Icon
            let icon_info = if node.is_dir() {
                let dir_icon = icons::dir_icon(node.is_expanded);
                icons::IconInfo {
                    icon: dir_icon,
                    color: self.theme.dir_color,
                }
            } else {
                icons::icon_for_file(&node.name, false, self.theme)
            };

            let is_ignored = node.git_status == GitStatus::Ignored;

            let mut icon_style = Style::default().fg(icon_info.color).bg(bg);
            if is_ignored {
                icon_style = icon_style.add_modifier(Modifier::DIM);
            }
            spans.push(Span::styled(format!("{} ", icon_info.icon), icon_style));

            // File name
            let name_color = git_status_color(node.git_status, self.theme);
            let name_style = if node.is_dir() {
                let mut s = Style::default()
                    .fg(name_color)
                    .bg(bg)
                    .add_modifier(Modifier::BOLD);
                if is_ignored {
                    s = s.add_modifier(Modifier::DIM);
                }
                s
            } else {
                let mut s = Style::default().fg(name_color).bg(bg);
                if is_ignored {
                    s = s.add_modifier(Modifier::DIM);
                }
                s
            };
            spans.push(Span::styled(&node.name, name_style));

            // Git status marker (right-aligned later; for now, append)
            let git_marker = git_status_marker(node.git_status);
            if !git_marker.is_empty() {
                let mut marker_style = Style::default()
                    .fg(git_status_color(node.git_status, self.theme))
                    .bg(bg);
                if is_ignored {
                    marker_style = marker_style.add_modifier(Modifier::DIM);
                }
                spans.push(Span::styled(format!(" {git_marker}"), marker_style));
            }

            // Fill remaining width with background color for selected row
            let line = Line::from(spans);
            let line_width = line.width() as u16;

            // Render the line content
            line.render(Rect::new(area.x, y, area.width.min(line_width + 1), 1), buf);

            // Fill rest of the row with bg color for selected highlight
            if is_selected && line_width < area.width {
                for x in (area.x + line_width)..(area.x + area.width) {
                    if let Some(cell) = buf.cell_mut((x, y)) {
                        cell.set_style(Style::default().bg(bg));
                    }
                }
            }
        }
    }
}

fn git_status_color(status: GitStatus, theme: &Theme) -> ratatui::style::Color {
    match status {
        GitStatus::Modified => theme.git_modified,
        GitStatus::Added | GitStatus::Untracked => theme.git_added,
        GitStatus::Deleted => theme.git_deleted,
        GitStatus::Ignored => theme.git_ignored,
        GitStatus::Conflicted => theme.git_conflicted,
        GitStatus::Clean => theme.default_fg,
    }
}

fn git_status_marker(status: GitStatus) -> &'static str {
    match status {
        GitStatus::Modified => "M",
        GitStatus::Added => "A",
        GitStatus::Deleted => "D",
        GitStatus::Untracked => "U",
        GitStatus::Ignored => "!",
        GitStatus::Conflicted => "C",
        GitStatus::Clean => "",
    }
}
