use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{StatefulWidget, Widget},
};

use crate::tree::forest::FileTree;
use crate::tree::node::GitStatus;

use super::colors;
use super::icons;

pub struct TreeView;

impl StatefulWidget for TreeView {
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
                colors::SELECTED_BG
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
                    Style::default().fg(colors::TREE_LINE).bg(bg),
                ));
            }

            // Branch connector for this node
            if node.depth > 0 {
                let is_last = state.is_last_sibling(absolute_idx);
                let branch = if is_last { "└─" } else { "├─" };
                spans.push(Span::styled(
                    branch,
                    Style::default().fg(colors::TREE_LINE).bg(bg),
                ));
            }

            // Icon
            let icon_info = if node.is_dir() {
                let dir_icon = icons::dir_icon(node.is_expanded);
                icons::IconInfo {
                    icon: dir_icon,
                    color: colors::DIR_COLOR,
                }
            } else {
                icons::icon_for_file(&node.name, false)
            };

            spans.push(Span::styled(
                format!("{} ", icon_info.icon),
                Style::default().fg(icon_info.color).bg(bg),
            ));

            // File name
            let name_color = git_status_color(node.git_status);
            let name_style = if node.is_dir() {
                Style::default()
                    .fg(name_color)
                    .bg(bg)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(name_color).bg(bg)
            };
            spans.push(Span::styled(&node.name, name_style));

            // Git status marker (right-aligned later; for now, append)
            let git_marker = git_status_marker(node.git_status);
            if !git_marker.is_empty() {
                spans.push(Span::styled(
                    format!(" {}", git_marker),
                    Style::default().fg(git_status_color(node.git_status)).bg(bg),
                ));
            }

            // Fill remaining width with background color for selected row
            let line = Line::from(spans);
            let line_width = line.width() as u16;

            // Render the line content
            line.render(
                Rect::new(area.x, y, area.width.min(line_width + 1), 1),
                buf,
            );

            // Fill rest of the row with bg color for selected highlight
            if is_selected && line_width < area.width {
                for x in (area.x + line_width)..=(area.x + area.width - 1) {
                    if let Some(cell) = buf.cell_mut((x, y)) {
                        cell.set_style(Style::default().bg(bg));
                    }
                }
            }
        }
    }
}

fn git_status_color(status: GitStatus) -> ratatui::style::Color {
    match status {
        GitStatus::Modified => colors::GIT_MODIFIED,
        GitStatus::Added | GitStatus::Untracked => colors::GIT_ADDED,
        GitStatus::Deleted => colors::GIT_DELETED,
        GitStatus::Ignored => colors::GIT_IGNORED,
        GitStatus::Conflicted => colors::GIT_CONFLICTED,
        GitStatus::Clean => colors::DEFAULT_FG,
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
