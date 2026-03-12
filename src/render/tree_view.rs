use std::collections::HashMap;
use std::time::SystemTime;

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{StatefulWidget, Widget},
};

use crate::config::TreeConfig;
use crate::tree::forest::FileTree;
use crate::tree::node::{GitStatus, TreeNode};

use super::colors;
use super::icons;

pub struct TreeView<'a> {
    pub config: &'a TreeConfig,
    pub hover_row: Option<usize>,
    /// When non-empty, only show nodes at these indices (filter mode).
    pub filter_indices: &'a [usize],
    /// When non-empty, highlight (underline) nodes at these indices (find mode).
    pub highlight_indices: &'a [usize],
    /// Per-node byte positions of matched characters for character-level highlighting.
    pub highlight_char_positions: &'a HashMap<usize, Vec<usize>>,
}

impl StatefulWidget for TreeView<'_> {
    type State = FileTree;

    #[allow(clippy::cast_possible_truncation, clippy::too_many_lines)]
    fn render(self, area: Rect, buf: &mut Buffer, state: &mut FileTree) {
        // Row highlight modes:
        //   Cursor  → REVERSED (strips fg for clean bar)
        //   Hover   → REVERSED | DIM (subtler than cursor)
        //   None    → transparent
        #[derive(PartialEq)]
        enum RowMode {
            Cursor,
            Hover,
            None,
        }

        let height = area.height as usize;
        let is_filtered = !self.filter_indices.is_empty();

        // Build a list of visible node indices, skipping compacted intermediate dirs.
        // When filter is active, use filtered indices instead.
        let visible_indices = if is_filtered {
            build_filtered_visible(state, self.filter_indices, height)
        } else {
            build_visible_indices(state, height)
        };

        // Store for mouse click resolution
        state.rendered_indices.clone_from(&visible_indices);

        // Precompute guides: use filtered guides when in filter mode
        let filtered_guides = if is_filtered {
            Some(precompute_filtered_guides(
                &state.nodes,
                self.filter_indices,
            ))
        } else {
            Option::None
        };
        let all_guides = if filtered_guides.is_none() {
            state.precompute_all_guides()
        } else {
            Vec::new()
        };

        for (row, &absolute_idx) in visible_indices.iter().enumerate() {
            let y = area.y + row as u16;
            if y >= area.y + area.height {
                break;
            }

            let node = &state.nodes[absolute_idx];
            let is_cursor = absolute_idx == state.cursor;
            let is_hovered = self.hover_row == Some(row);

            // Check if this node is highlighted (Find mode match)
            let is_highlighted = !self.highlight_indices.is_empty()
                && self.highlight_indices.binary_search(&absolute_idx).is_ok();

            // In filter mode, disable compaction (semantics unclear after filtering)
            let chain_len = if is_filtered {
                0
            } else {
                state.compact_chain_len(absolute_idx)
            };

            let mut spans = Vec::new();

            let row_mode = if is_cursor {
                RowMode::Cursor
            } else if is_hovered {
                RowMode::Hover
            } else {
                RowMode::None
            };

            // Build a row style: REVERSED variants strip fg for a clean bar.
            let row_style = |base: Style| -> Style {
                match &row_mode {
                    RowMode::Cursor => Style::default().add_modifier(
                        Modifier::REVERSED | (base.add_modifier & (Modifier::BOLD | Modifier::DIM)),
                    ),
                    RowMode::Hover => Style::default().add_modifier(
                        Modifier::REVERSED | Modifier::DIM | (base.add_modifier & Modifier::BOLD),
                    ),
                    RowMode::None => base,
                }
            };

            // Tree connectors
            let guides: &[bool] = if let Some(ref fg) = filtered_guides {
                // Look up position in the full filter_indices list
                let pos = self
                    .filter_indices
                    .binary_search(&absolute_idx)
                    .unwrap_or(0);
                if pos < fg.len() {
                    &fg[pos]
                } else {
                    &[]
                }
            } else {
                &all_guides[absolute_idx]
            };

            for (d, &has_continuation) in guides.iter().enumerate() {
                if d == 0 && node.depth == 0 {
                    continue;
                }
                let connector = if has_continuation { "│ " } else { "  " };
                spans.push(Span::styled(
                    connector,
                    row_style(Style::default().fg(colors::TREE_LINE)),
                ));
            }

            // Branch connector for this node
            if node.depth > 0 {
                let is_last = if is_filtered {
                    let pos = self
                        .filter_indices
                        .binary_search(&absolute_idx)
                        .unwrap_or(0);
                    is_last_visible_sibling(&state.nodes, self.filter_indices, pos)
                } else {
                    state.is_last_sibling(absolute_idx)
                };
                let branch = if is_last { "└─" } else { "├─" };
                spans.push(Span::styled(
                    branch,
                    row_style(Style::default().fg(colors::TREE_LINE)),
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

            let is_ignored = node.git_status == GitStatus::Ignored;

            let mut icon_base = Style::default().fg(icon_info.color);
            if is_ignored {
                icon_base = icon_base.add_modifier(Modifier::DIM);
            }
            let icon_style = row_style(icon_base);
            spans.push(Span::styled(format!("{} ", icon_info.icon), icon_style));

            // File/dir name — use compacted display name if applicable
            let display_name = if chain_len > 0 {
                state.compact_display_name(absolute_idx, chain_len)
            } else {
                node.name.clone()
            };

            let git_style = git_status_style(node.git_status);
            let git_name_base = {
                let mut s = git_style;
                if node.is_dir() {
                    s = s.add_modifier(Modifier::BOLD);
                }
                if is_ignored {
                    s = s.add_modifier(Modifier::DIM);
                }
                s
            };

            // Determine highlight mode for this row's name
            let name_highlight = if is_cursor || is_hovered || !is_highlighted {
                NameHighlight::None
            } else if let Some(positions) = self.highlight_char_positions.get(&absolute_idx) {
                NameHighlight::Characters(positions)
            } else {
                NameHighlight::FullName
            };

            let name_spans = build_name_spans(
                &display_name,
                git_name_base,
                name_highlight,
                colors::FIND_MATCH,
                &row_style,
            );
            spans.extend(name_spans);

            // Git status marker
            let git_marker = git_status_marker(node.git_status);
            if !git_marker.is_empty() {
                let mut marker_base = git_status_style(node.git_status);
                if is_ignored {
                    marker_base = marker_base.add_modifier(Modifier::DIM);
                }
                spans.push(Span::styled(
                    format!(" {git_marker}"),
                    row_style(marker_base),
                ));
            }

            // Build info columns (size + modified) for right-aligned display
            let info_text = build_info_text(node, self.config.show_size, self.config.show_modified);

            // Fill entire row with highlight style
            if row_mode != RowMode::None {
                let fill_style = match &row_mode {
                    RowMode::Cursor => Style::default().add_modifier(Modifier::REVERSED),
                    RowMode::Hover => colors::hover_style(),
                    RowMode::None => unreachable!(),
                };
                for x in area.x..(area.x + area.width) {
                    if let Some(cell) = buf.cell_mut((x, y)) {
                        cell.set_style(fill_style);
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
                    let info_style =
                        row_style(Style::default().fg(colors::GIT_IGNORED).add_modifier(
                            if row_mode == RowMode::Cursor {
                                Modifier::DIM
                            } else {
                                Modifier::empty()
                            },
                        ));
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

/// Check if a node at `pos` in `visible_indices` is the last visible sibling at its depth.
fn is_last_visible_sibling(nodes: &[TreeNode], visible_indices: &[usize], pos: usize) -> bool {
    if pos >= visible_indices.len() {
        return true;
    }
    let depth = nodes[visible_indices[pos]].depth;

    for &next_idx in &visible_indices[pos + 1..] {
        let next_depth = nodes[next_idx].depth;
        if next_depth <= depth {
            return next_depth < depth;
        }
    }
    true
}

/// Precompute connector guides for a filtered subset of nodes.
/// Returns one Vec<bool> per entry in `visible_indices`.
fn precompute_filtered_guides(nodes: &[TreeNode], visible_indices: &[usize]) -> Vec<Vec<bool>> {
    let n = visible_indices.len();
    if n == 0 {
        return Vec::new();
    }

    let max_depth = visible_indices
        .iter()
        .map(|&idx| nodes[idx].depth)
        .max()
        .unwrap_or(0);

    let mut has_more = vec![false; max_depth + 1];
    let mut rev_results: Vec<Vec<bool>> = Vec::with_capacity(n);

    for &idx in visible_indices.iter().rev() {
        let depth = nodes[idx].depth;
        let mut guides = vec![false; depth];
        for (d, guide) in guides.iter_mut().enumerate() {
            *guide = has_more[d];
        }
        rev_results.push(guides);

        has_more[depth] = true;
        for item in &mut has_more[(depth + 1)..=max_depth] {
            *item = false;
        }
    }

    rev_results.reverse();
    rev_results
}

// ── Character-level find highlighting ─────────────────────────────────

#[derive(Clone, Copy)]
enum NameHighlight<'a> {
    /// No highlight (not a match, or cursor/hover row).
    None,
    /// Highlight specific matched characters (display-name match).
    Characters(&'a [usize]),
    /// Underline entire name (path-only match fallback).
    FullName,
}

/// Build styled spans for a filename with optional character-level highlighting.
fn build_name_spans(
    display_name: &str,
    base_style: Style,
    highlight: NameHighlight<'_>,
    highlight_color: Color,
    row_style_fn: &dyn Fn(Style) -> Style,
) -> Vec<Span<'static>> {
    match highlight {
        NameHighlight::None => {
            vec![Span::styled(
                display_name.to_owned(),
                row_style_fn(base_style),
            )]
        }
        NameHighlight::FullName => {
            vec![Span::styled(
                display_name.to_owned(),
                row_style_fn(base_style.add_modifier(Modifier::UNDERLINED)),
            )]
        }
        NameHighlight::Characters(positions) => {
            let highlight_style = row_style_fn(
                base_style
                    .fg(highlight_color)
                    .add_modifier(Modifier::UNDERLINED),
            );
            let normal_style = row_style_fn(base_style);

            let mut spans = Vec::new();
            let mut current_start = 0;
            let mut current_is_match = false;
            let mut pos_idx = 0;

            for (byte_idx, ch) in display_name.char_indices() {
                let is_match = pos_idx < positions.len() && positions[pos_idx] == byte_idx;
                if is_match {
                    pos_idx += 1;
                }

                if byte_idx == 0 {
                    current_is_match = is_match;
                    continue;
                }

                if is_match != current_is_match {
                    // Flush the accumulated segment
                    let segment = &display_name[current_start..byte_idx];
                    let style = if current_is_match {
                        highlight_style
                    } else {
                        normal_style
                    };
                    spans.push(Span::styled(segment.to_owned(), style));
                    current_start = byte_idx;
                    current_is_match = is_match;
                }

                // Advance past multibyte chars
                let _ = ch;
            }

            // Flush final segment
            if current_start < display_name.len() {
                let segment = &display_name[current_start..];
                let style = if current_is_match {
                    highlight_style
                } else {
                    normal_style
                };
                spans.push(Span::styled(segment.to_owned(), style));
            }

            spans
        }
    }
}

fn git_status_style(status: GitStatus) -> Style {
    match status {
        GitStatus::Modified => Style::default().fg(colors::GIT_MODIFIED),
        GitStatus::Added | GitStatus::Untracked => Style::default().fg(colors::GIT_ADDED),
        GitStatus::Deleted => Style::default().fg(colors::GIT_DELETED),
        GitStatus::Ignored => Style::default().fg(colors::GIT_IGNORED),
        GitStatus::Conflicted => Style::default()
            .fg(colors::GIT_CONFLICTED)
            .add_modifier(Modifier::BOLD),
        GitStatus::StagedModified => Style::default()
            .fg(colors::GIT_STAGED_MODIFIED)
            .add_modifier(Modifier::DIM),
        GitStatus::StagedAdded => Style::default()
            .fg(colors::GIT_STAGED_ADDED)
            .add_modifier(Modifier::DIM),
        GitStatus::StagedDeleted => Style::default()
            .fg(colors::GIT_STAGED_DELETED)
            .add_modifier(Modifier::DIM),
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
    use crate::tree::node::{NodeKind, TreeNode};
    use std::path::PathBuf;

    /// Build a minimal FileTree with two file nodes for rendering tests.
    fn make_test_tree() -> FileTree {
        let config = crate::config::TreeConfig {
            show_hidden: true,
            show_ignored: true,
            dirs_first: true,
            exclude: vec![],
            compact_folders: false,
            show_size: false,
            show_modified: false,
        };
        FileTree {
            nodes: vec![
                TreeNode::new(PathBuf::from("/tmp/a.txt"), NodeKind::File, 0),
                TreeNode::new(PathBuf::from("/tmp/b.txt"), NodeKind::File, 0),
            ],
            cursor: 0,
            scroll_offset: 0,
            root: PathBuf::from("/tmp"),
            config,
            rendered_indices: vec![],
            file_count: 2,
            dir_count: 0,
        }
    }

    fn render_tree(tree: &mut FileTree, hover_row: Option<usize>) -> Buffer {
        let config = tree.config.clone();
        let area = ratatui::layout::Rect::new(0, 0, 40, 10);
        let mut buf = Buffer::empty(area);
        let empty_positions = HashMap::new();
        let widget = TreeView {
            config: &config,
            hover_row,
            filter_indices: &[],
            highlight_indices: &[],
            highlight_char_positions: &empty_positions,
        };
        widget.render(area, &mut buf, tree);
        buf
    }

    #[test]
    fn hover_row_has_reversed_and_dim() {
        let mut tree = make_test_tree();
        tree.cursor = 0; // cursor on row 0
        let buf = render_tree(&mut tree, Some(1)); // hover on row 1
        let cell = buf.cell((5, 1)).unwrap();
        assert!(
            cell.modifier.contains(Modifier::REVERSED) && cell.modifier.contains(Modifier::DIM),
            "hover row should have REVERSED | DIM, got {:?}",
            cell.modifier
        );
    }

    #[test]
    fn cursor_overrides_hover() {
        let mut tree = make_test_tree();
        tree.cursor = 0;
        let buf = render_tree(&mut tree, Some(0)); // hover AND cursor on row 0
        let cell = buf.cell((5, 0)).unwrap();
        assert!(
            cell.modifier.contains(Modifier::REVERSED),
            "cursor+hover row should have REVERSED, got {:?}",
            cell.modifier
        );
        assert!(
            !cell.modifier.contains(Modifier::DIM),
            "cursor+hover row should NOT have DIM (cursor priority)"
        );
    }

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

    #[test]
    fn is_last_visible_sibling_basic() {
        let nodes = vec![
            TreeNode::new(PathBuf::from("a"), NodeKind::File, 0),
            TreeNode::new(PathBuf::from("b"), NodeKind::File, 0),
            TreeNode::new(PathBuf::from("c"), NodeKind::File, 0),
        ];
        let visible = vec![0, 1, 2];
        assert!(!is_last_visible_sibling(&nodes, &visible, 0));
        assert!(!is_last_visible_sibling(&nodes, &visible, 1));
        assert!(is_last_visible_sibling(&nodes, &visible, 2));
    }

    #[test]
    fn is_last_visible_sibling_filtered() {
        // Only nodes 0 and 2 are visible (1 is filtered out)
        let nodes = vec![
            TreeNode::new(PathBuf::from("a"), NodeKind::File, 0),
            TreeNode::new(PathBuf::from("b"), NodeKind::File, 0),
            TreeNode::new(PathBuf::from("c"), NodeKind::File, 0),
        ];
        let visible = vec![0, 2];
        assert!(!is_last_visible_sibling(&nodes, &visible, 0));
        assert!(is_last_visible_sibling(&nodes, &visible, 1));
    }

    #[test]
    fn precompute_filtered_guides_basic() {
        let nodes = vec![
            TreeNode::new(PathBuf::from("dir"), NodeKind::Directory, 0),
            TreeNode::new(PathBuf::from("dir/a"), NodeKind::File, 1),
            TreeNode::new(PathBuf::from("dir/b"), NodeKind::File, 1),
        ];
        let visible = vec![0, 1, 2];
        let guides = precompute_filtered_guides(&nodes, &visible);
        assert_eq!(guides.len(), 3);
        assert_eq!(guides[0], Vec::<bool>::new()); // depth 0 → no guides
                                                   // guides[d] tracks whether depth d has a continuation below.
                                                   // For depth-1 nodes, guides[0] = whether there's a depth-0 node below.
                                                   // There isn't (node 0 is the only depth-0 node and it's above both), so [false].
        assert_eq!(guides[1], vec![false]);
        assert_eq!(guides[2], vec![false]);
    }

    // ── build_name_spans tests ──────────────────────────────────────────

    #[test]
    fn name_spans_none_single_span() {
        let base = Style::default().fg(colors::DEFAULT_FG);
        let identity = |s: Style| s;
        let spans = build_name_spans(
            "app.rs",
            base,
            NameHighlight::None,
            colors::FIND_MATCH,
            &identity,
        );
        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].content, "app.rs");
        assert_eq!(spans[0].style, base);
    }

    #[test]
    fn name_spans_full_name_underlined() {
        let base = Style::default().fg(colors::DEFAULT_FG);
        let identity = |s: Style| s;
        let spans = build_name_spans(
            "app.rs",
            base,
            NameHighlight::FullName,
            colors::FIND_MATCH,
            &identity,
        );
        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].content, "app.rs");
        assert!(spans[0].style.add_modifier.contains(Modifier::UNDERLINED));
    }

    #[test]
    fn name_spans_characters_at_start() {
        let base = Style::default().fg(colors::DEFAULT_FG);
        let identity = |s: Style| s;
        let positions = vec![0, 1, 2]; // "app" in "app.rs"
        let spans = build_name_spans(
            "app.rs",
            base,
            NameHighlight::Characters(&positions),
            colors::FIND_MATCH,
            &identity,
        );
        assert_eq!(spans.len(), 2);
        // First span: highlighted "app"
        assert_eq!(spans[0].content, "app");
        assert!(spans[0].style.add_modifier.contains(Modifier::UNDERLINED));
        assert_eq!(spans[0].style.fg, Some(colors::FIND_MATCH));
        // Second span: ".rs"
        assert_eq!(spans[1].content, ".rs");
        assert!(!spans[1].style.add_modifier.contains(Modifier::UNDERLINED));
    }

    #[test]
    fn name_spans_characters_scattered() {
        let base = Style::default().fg(colors::DEFAULT_FG);
        let identity = |s: Style| s;
        let positions = vec![0, 4, 5]; // "a", "r", "s" in "app.rs"
        let spans = build_name_spans(
            "app.rs",
            base,
            NameHighlight::Characters(&positions),
            colors::FIND_MATCH,
            &identity,
        );
        // "a" (match), "pp." (no match), "rs" (match)
        assert_eq!(spans.len(), 3);
        assert_eq!(spans[0].content, "a");
        assert!(spans[0].style.add_modifier.contains(Modifier::UNDERLINED));
        assert_eq!(spans[1].content, "pp.");
        assert!(!spans[1].style.add_modifier.contains(Modifier::UNDERLINED));
        assert_eq!(spans[2].content, "rs");
        assert!(spans[2].style.add_modifier.contains(Modifier::UNDERLINED));
    }

    #[test]
    fn name_spans_preserves_bold_on_highlight() {
        let base = Style::default()
            .fg(colors::DEFAULT_FG)
            .add_modifier(Modifier::BOLD);
        let identity = |s: Style| s;
        let positions = vec![0];
        let spans = build_name_spans(
            "src/",
            base,
            NameHighlight::Characters(&positions),
            colors::FIND_MATCH,
            &identity,
        );
        // Highlighted span should have BOLD + UNDERLINED
        assert!(spans[0].style.add_modifier.contains(Modifier::BOLD));
        assert!(spans[0].style.add_modifier.contains(Modifier::UNDERLINED));
    }
}
