use std::time::SystemTime;

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{StatefulWidget, Widget},
};

use crate::config::TreeConfig;
use crate::tree::forest::FileTree;
use crate::tree::node::GitStatus;

use super::colors;
use super::icons;

pub struct TreeView<'a> {
    pub config: &'a TreeConfig,
    pub hover_row: Option<usize>,
    /// When non-empty, only show nodes at these indices (search filter).
    pub filter_indices: &'a [usize],
}

impl StatefulWidget for TreeView<'_> {
    type State = FileTree;

    #[allow(clippy::cast_possible_truncation, clippy::too_many_lines)]
    fn render(self, area: Rect, buf: &mut Buffer, state: &mut FileTree) {
        let height = area.height as usize;

        // Build a list of visible node indices, skipping compacted intermediate dirs.
        // When filter is active, use filtered indices instead.
        let visible_indices = if self.filter_indices.is_empty() {
            build_visible_indices(state, height)
        } else {
            build_filtered_visible(state, self.filter_indices, height)
        };

        // Store for mouse click resolution
        state.rendered_indices.clone_from(&visible_indices);

        // Precompute all connector guides in O(N) instead of O(D×N) per node
        let all_guides = state.precompute_all_guides();

        for (row, &absolute_idx) in visible_indices.iter().enumerate() {
            let y = area.y + row as u16;
            if y >= area.y + area.height {
                break;
            }

            let node = &state.nodes[absolute_idx];
            let is_cursor = absolute_idx == state.cursor;
            let is_multi_selected = state.selected_set.contains(&absolute_idx);
            let is_hovered = self.hover_row == Some(row);

            // Check if this node starts a compact chain
            let chain_len = state.compact_chain_len(absolute_idx);

            let mut spans = Vec::new();
            let bg = if is_cursor || is_multi_selected {
                colors::SELECTED_BG
            } else if is_hovered {
                colors::HOVER_BG
            } else {
                ratatui::style::Color::Reset
            };

            // Tree connectors (using precomputed guides)
            let guides = &all_guides[absolute_idx];
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

            // Icon — for compacted dirs, use the last dir in the chain's expand state
            let icon_info = if node.is_dir() {
                let last_in_chain = &state.nodes[absolute_idx + chain_len];
                let dir_icon = icons::dir_icon(last_in_chain.is_expanded);
                icons::IconInfo {
                    icon: dir_icon,
                    color: colors::DIR_COLOR,
                }
            } else {
                icons::icon_for_file(&node.name, false)
            };

            // Multi-select marker
            if is_multi_selected {
                spans.push(Span::styled(
                    "● ",
                    Style::default().fg(ratatui::style::Color::Cyan).bg(bg),
                ));
            }

            let is_ignored = node.git_status == GitStatus::Ignored;

            let mut icon_style = Style::default().fg(icon_info.color).bg(bg);
            if is_ignored {
                icon_style = icon_style.add_modifier(Modifier::DIM);
            }
            spans.push(Span::styled(format!("{} ", icon_info.icon), icon_style));

            // File/dir name — use compacted display name if applicable
            let display_name = if chain_len > 0 {
                state.compact_display_name(absolute_idx, chain_len)
            } else {
                node.name.clone()
            };

            let git_style = git_status_style(node.git_status);
            let name_style = if node.is_dir() {
                let mut s = git_style.bg(bg).add_modifier(Modifier::BOLD);
                if is_ignored {
                    s = s.add_modifier(Modifier::DIM);
                }
                s
            } else {
                let mut s = git_style.bg(bg);
                if is_ignored {
                    s = s.add_modifier(Modifier::DIM);
                }
                s
            };
            spans.push(Span::styled(display_name, name_style));

            // Git status marker
            let git_marker = git_status_marker(node.git_status);
            if !git_marker.is_empty() {
                let mut marker_style = git_status_style(node.git_status).bg(bg);
                if is_ignored {
                    marker_style = marker_style.add_modifier(Modifier::DIM);
                }
                spans.push(Span::styled(format!(" {git_marker}"), marker_style));
            }

            // Build info columns (size + modified) for right-aligned display
            let info_text = build_info_text(node, self.config.show_size, self.config.show_modified);

            // Fill entire row with bg first for cursor/selected/hover highlight
            if is_cursor || is_multi_selected || is_hovered {
                for x in area.x..(area.x + area.width) {
                    if let Some(cell) = buf.cell_mut((x, y)) {
                        cell.set_style(Style::default().bg(bg));
                    }
                }
            }

            // Render the line content (overwrites bg cells)
            let line = Line::from(spans);
            let line_width = line.width() as u16;
            line.render(Rect::new(area.x, y, area.width.min(line_width + 1), 1), buf);

            // Render right-aligned info columns if present
            if !info_text.is_empty() {
                let info_width = info_text.len() as u16;
                let min_gap = 2;
                if line_width + min_gap + info_width < area.width {
                    let info_x = area.x + area.width - info_width;
                    let info_style = Style::default()
                        .fg(colors::GIT_IGNORED) // dim gray
                        .bg(bg);
                    let info_span = Line::from(Span::styled(info_text, info_style));
                    info_span.render(Rect::new(info_x, y, info_width, 1), buf);
                }
            }
        }
    }
}

/// Build a list of node indices to render, skipping intermediate compacted directories.
/// Also adjusts scroll so the cursor stays on a visible (non-skipped) row.
fn build_visible_indices(state: &mut FileTree, viewport_height: usize) -> Vec<usize> {
    // First pass: determine which indices are visible (not compacted away)
    let mut all_visible = Vec::with_capacity(state.nodes.len());
    let mut i = 0;
    while i < state.nodes.len() {
        all_visible.push(i);
        let chain = state.compact_chain_len(i);
        // Skip the intermediate dirs in the chain (they're merged into the display)
        i += chain + 1;
    }

    // Ensure cursor snaps to a visible index
    if let Some(pos) = all_visible.iter().position(|&idx| idx >= state.cursor) {
        if all_visible[pos] != state.cursor {
            state.cursor = all_visible[pos]; // snap forward to nearest visible
        }
    }

    // Apply scrolling within the visible-indices list
    let cursor_vis_pos = all_visible
        .iter()
        .position(|&idx| idx == state.cursor)
        .unwrap_or(0);

    // Adjust scroll offset to be in terms of visible rows
    if cursor_vis_pos < state.scroll_offset {
        state.scroll_offset = cursor_vis_pos;
    }
    if cursor_vis_pos >= state.scroll_offset + viewport_height {
        state.scroll_offset = cursor_vis_pos - viewport_height + 1;
    }

    let start = state.scroll_offset;
    let end = (start + viewport_height).min(all_visible.len());
    all_visible[start..end].to_vec()
}

/// Build visible indices from a pre-filtered set. Uses the same scroll logic.
fn build_filtered_visible(
    state: &mut FileTree,
    filter_indices: &[usize],
    viewport_height: usize,
) -> Vec<usize> {
    if filter_indices.is_empty() {
        return Vec::new();
    }

    // Ensure cursor snaps to a filtered index
    if !filter_indices.contains(&state.cursor) {
        // Find the nearest filtered index
        if let Some(&nearest) = filter_indices.first() {
            state.cursor = nearest;
        }
    }

    let cursor_pos = filter_indices
        .iter()
        .position(|&idx| idx == state.cursor)
        .unwrap_or(0);

    if cursor_pos < state.scroll_offset {
        state.scroll_offset = cursor_pos;
    }
    if cursor_pos >= state.scroll_offset + viewport_height {
        state.scroll_offset = cursor_pos - viewport_height + 1;
    }

    let start = state.scroll_offset;
    let end = (start + viewport_height).min(filter_indices.len());
    filter_indices[start..end].to_vec()
}

fn git_status_style(status: GitStatus) -> Style {
    match status {
        GitStatus::Modified => Style::default().fg(colors::GIT_MODIFIED),
        GitStatus::Added | GitStatus::Untracked => Style::default().fg(colors::GIT_ADDED),
        GitStatus::Deleted => Style::default().fg(colors::GIT_DELETED),
        GitStatus::Ignored => Style::default().fg(colors::GIT_IGNORED),
        GitStatus::Conflicted => Style::default().fg(colors::GIT_CONFLICTED).add_modifier(Modifier::BOLD),
        GitStatus::StagedModified => Style::default().fg(colors::GIT_STAGED_MODIFIED).add_modifier(Modifier::DIM),
        GitStatus::StagedAdded => Style::default().fg(colors::GIT_STAGED_ADDED).add_modifier(Modifier::DIM),
        GitStatus::StagedDeleted => Style::default().fg(colors::GIT_STAGED_DELETED).add_modifier(Modifier::DIM),
        GitStatus::Clean => Style::default().fg(colors::DEFAULT_FG),
    }
}

fn git_status_marker(status: GitStatus) -> &'static str {
    match status {
        GitStatus::Modified | GitStatus::StagedModified => "M",
        GitStatus::Added | GitStatus::StagedAdded => "A",
        GitStatus::Deleted | GitStatus::StagedDeleted => "D",
        GitStatus::Untracked => "U",
        GitStatus::Conflicted => "C",
        GitStatus::Ignored | GitStatus::Clean => "",
    }
}

// ── Info columns ─────────────────────────────────────────────────────────

use crate::tree::node::TreeNode;

/// Build the right-aligned info text combining size and/or modified time.
fn build_info_text(node: &TreeNode, show_size: bool, show_modified: bool) -> String {
    let mut parts = Vec::new();

    if show_size {
        if let Some(size) = node.size {
            parts.push(format_size(size));
        } else if node.is_dir() && show_modified {
            // Directories don't show size; add padding to align with files
        }
    }

    if show_modified {
        if let Some(modified) = node.modified {
            parts.push(format_time(modified));
        }
    }

    if parts.is_empty() {
        return String::new();
    }

    parts.join("  ")
}

/// Format bytes into human-readable size: 892, 4.2K, 1.3M, 2.1G
#[allow(clippy::cast_precision_loss)]
fn format_size(bytes: u64) -> String {
    if bytes < 1000 {
        return format!("{bytes:>4}");
    }
    let units = ['K', 'M', 'G', 'T'];
    let mut value = bytes as f64;
    for unit in &units {
        value /= 1024.0;
        if value < 10.0 {
            return format!("{value:>3.1}{unit}");
        }
        if value < 1000.0 {
            return format!("{value:>3.0}{unit}");
        }
    }
    format!("{value:.0}T")
}

/// Format modification time as relative ("2h ago", "3d ago") or date ("Jan 18").
fn format_time(time: SystemTime) -> String {
    let Ok(elapsed) = time.elapsed() else {
        return String::new();
    };

    let secs = elapsed.as_secs();
    if secs < 60 {
        return "just now".to_string();
    }

    let mins = secs / 60;
    if mins < 60 {
        return format!("{mins}m ago");
    }

    let hours = mins / 60;
    if hours < 24 {
        return format!("{hours}h ago");
    }

    let days = hours / 24;
    if days <= 7 {
        return format!("{days}d ago");
    }

    // Beyond 7 days: show date
    // Convert SystemTime to a simple month + day
    // Use days since epoch to compute rough month/day
    let secs_since_epoch = time
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    format_epoch_date(secs_since_epoch)
}

/// Convert seconds since epoch to "Mon DD" format (e.g., "Jan 18").
#[allow(clippy::cast_possible_wrap)]
fn format_epoch_date(epoch_secs: u64) -> String {
    // Days since epoch
    let total_days = epoch_secs / 86400;

    // Compute year, month, day from days since epoch (civil calendar)
    let (y, m, d) = days_to_civil(total_days as i64 + 719_468);

    let months = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];

    let month_str = months.get(m as usize - 1).unwrap_or(&"???");

    // If it's the current year, show "Mon DD"; otherwise "Mon DD YY"
    let now_epoch = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let now_days = now_epoch / 86400;
    let (current_year, _, _) = days_to_civil(now_days as i64 + 719_468);

    if y == current_year {
        format!("{month_str} {d:>2}")
    } else {
        format!("{month_str} {d:>2} '{}", y % 100)
    }
}

/// Convert days since epoch 0000-03-01 to (year, month, day).
/// Algorithm from Howard Hinnant's chrono-compatible date library.
#[allow(clippy::cast_possible_truncation, clippy::cast_lossless)]
fn days_to_civil(days: i64) -> (i64, u32, u32) {
    let era = days.div_euclid(146_097);
    let doe = days.rem_euclid(146_097) as u32;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y, m, d)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_size_bytes() {
        assert_eq!(format_size(0), "   0");
        assert_eq!(format_size(892), " 892");
        assert_eq!(format_size(999), " 999");
    }

    #[test]
    fn format_size_kilobytes() {
        assert_eq!(format_size(1024), "1.0K");
        assert_eq!(format_size(4300), "4.2K");
        assert_eq!(format_size(10240), " 10K");
    }

    #[test]
    fn format_size_megabytes() {
        assert_eq!(format_size(1_400_000), "1.3M");
        assert_eq!(format_size(52_428_800), " 50M");
    }

    #[test]
    fn format_epoch_date_produces_valid_output() {
        // 2024-01-18 = 1705536000
        let s = format_epoch_date(1_705_536_000);
        assert!(s.starts_with("Jan"), "got: {s}");
    }
}
