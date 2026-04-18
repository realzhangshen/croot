use super::*;

impl App {
    pub(super) fn draw(&mut self, frame: &mut ratatui::Frame) {
        let size = frame.area();
        self.last_terminal_area = size;

        let show_search_bar = self.ui.input_mode == InputMode::Search
            || (!self.search_state.is_empty() && self.search_state.mode != SearchMode::Global);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(if show_search_bar {
                vec![
                    Constraint::Min(1),
                    Constraint::Length(1),
                    Constraint::Length(1),
                ]
            } else {
                vec![Constraint::Min(1), Constraint::Length(1)]
            })
            .split(size);

        let main_area = chunks[0];
        let status_area = chunks[1];

        self.status_bar_y = status_area.y;
        self.search_bar_y = if show_search_bar {
            Some(chunks[2].y)
        } else {
            None
        };

        let content_area = main_area;
        self.tree_area_y = content_area.y;
        self.main_area_width = main_area.width;

        if self.preview.visible && content_area.width > 20 {
            let ratio = self.config.preview.split_ratio.clamp(0.2, 0.8);
            let tree_width = (f32::from(content_area.width) * (1.0 - ratio)) as u16;
            let separator_width: u16 = 1;
            let preview_width = content_area
                .width
                .saturating_sub(tree_width + separator_width);

            let tree_area = ratatui::layout::Rect {
                x: content_area.x,
                y: content_area.y,
                width: tree_width,
                height: content_area.height,
            };
            let separator_area = ratatui::layout::Rect {
                x: content_area.x + tree_width,
                y: content_area.y,
                width: separator_width,
                height: content_area.height,
            };
            let preview_area = ratatui::layout::Rect {
                x: content_area.x + tree_width + separator_width,
                y: content_area.y,
                width: preview_width,
                height: content_area.height,
            };

            self.tree_area_height = tree_area.height;

            TreeView {
                config: &self.config.tree,
                hover_row: self.hover_row,
                filter_indices: if self.search_state.mode == SearchMode::Filter
                    && !self.search_state.visible_indices.is_empty()
                {
                    &self.search_state.visible_indices
                } else {
                    &[]
                },
                highlight_indices: if self.search_state.mode == SearchMode::Find
                    && !self.search_state.match_indices.is_empty()
                {
                    &self.search_state.match_indices
                } else {
                    &[]
                },
                highlight_char_positions: &self.search_state.match_char_positions,
            }
            .render(tree_area, frame.buffer_mut(), &mut self.tree);

            let sep_style = colors::tree_connector();
            for y in separator_area.y..separator_area.y + separator_area.height {
                frame
                    .buffer_mut()
                    .set_string(separator_area.x, y, "\u{2502}", sep_style);
            }

            self.preview.content_width = preview_width;
            self.preview.area_x = Some(preview_area.x);

            let content_area_y = preview_area.y + 1;
            let content_area_height = preview_area.height.saturating_sub(1);
            let gutter_width = crate::render::preview_view::compute_gutter_width(
                self.config.preview.show_line_numbers,
                self.config.preview.show_git_diff,
                &self.preview.state.kind,
                self.preview.state.total_lines,
                self.preview.state.line_diffs.is_some(),
            );
            self.preview.layout = Some(PreviewLayout {
                x: preview_area.x + gutter_width,
                y: content_area_y,
                height: content_area_height,
            });

            PreviewView {
                config: &self.config.preview,
                focused: self.focus == FocusPane::Preview,
            }
            .render(preview_area, frame.buffer_mut(), &mut self.preview.state);
        } else {
            self.preview.area_x = None;
            self.preview.layout = None;
            self.tree_area_height = content_area.height;

            TreeView {
                config: &self.config.tree,
                hover_row: self.hover_row,
                filter_indices: if self.search_state.mode == SearchMode::Filter
                    && !self.search_state.visible_indices.is_empty()
                {
                    &self.search_state.visible_indices
                } else {
                    &[]
                },
                highlight_indices: if self.search_state.mode == SearchMode::Find
                    && !self.search_state.match_indices.is_empty()
                {
                    &self.search_state.match_indices
                } else {
                    &[]
                },
                highlight_char_positions: &self.search_state.match_char_positions,
            }
            .render(content_area, frame.buffer_mut(), &mut self.tree);
        }

        let root_name = self.root.file_name().map_or_else(
            || self.root.to_string_lossy().into_owned(),
            |n| n.to_string_lossy().into_owned(),
        );
        let root_path = self.root.to_string_lossy().into_owned();

        let selected_rel = self.tree.selected().and_then(|n| {
            if n.is_dir() {
                None
            } else {
                n.path
                    .strip_prefix(&self.root)
                    .ok()
                    .map(|p| p.to_string_lossy().into_owned())
            }
        });
        let selected_abs = self.tree.selected().and_then(|n| {
            if n.is_dir() {
                None
            } else {
                Some(n.path.to_string_lossy().into_owned())
            }
        });

        let file_count = self.tree.file_count;
        let dir_count = self.tree.dir_count;
        let branch = self
            .git
            .as_ref()
            .and_then(|g| g.branch())
            .map(std::string::ToString::to_string);
        let cmux_indicator = if self.cmux.is_some() {
            Some("cmux")
        } else {
            None
        };

        let status_bar = StatusBar {
            branch: branch.as_deref(),
            file_count,
            dir_count,
            root_name: &root_name,
            root_path: &root_path,
            cmux_status: cmux_indicator,
            selected_path: selected_rel.as_deref(),
            selected_abs_path: selected_abs.as_deref(),
        };
        self.status_bar_branch_region = branch.as_ref().map(|b| {
            // UnicodeWidthStr reports Nerd-Font glyphs like U+E0A0 as 1 column, but
            // most terminals render them as 2. Over-estimating by one keeps the
            // click target accurate on Nerd-Font terminals without missing on
            // plain ones (false-positive click > missed click).
            let nerd_font_compensation: u16 = 1;
            let span_text = format!("  \u{e0a0} {b} ");
            let end = UnicodeWidthStr::width(span_text.as_str()) as u16 + nerd_font_compensation;
            (0, end)
        });

        self.hyperlink_regions = status_bar.hyperlink_regions(status_area);
        status_bar.render(status_area, frame.buffer_mut());

        // Overlay error message on the status bar; auto-dismisses after 3s.
        if let Some((ref msg, ts)) = self.ui.error_message {
            if ts.elapsed() < Duration::from_secs(3) {
                self.hyperlink_regions.clear();
                let error_style = ratatui::style::Style::default()
                    .fg(ratatui::style::Color::White)
                    .bg(ratatui::style::Color::Red)
                    .add_modifier(ratatui::style::Modifier::BOLD);
                let display = crate::render::text_util::truncate_to_display_width(
                    msg,
                    status_area.width as usize,
                );
                let buf = frame.buffer_mut();
                for x in status_area.x..status_area.x + status_area.width {
                    if let Some(cell) = buf.cell_mut((x, status_area.y)) {
                        cell.set_symbol(" ");
                        cell.set_style(error_style);
                    }
                }
                buf.set_string(status_area.x, status_area.y, display, error_style);
            } else {
                self.ui.error_message = None;
            }
        }

        if show_search_bar {
            let search_area = chunks[2];
            let search_bar = SearchBar {
                state: &self.search_state,
                show_close_button: true,
            };
            search_bar.render(search_area, frame.buffer_mut());
        }

        if let Some(ref menu) = self.ui.context_menu {
            let widget = ContextMenuWidget { state: menu };
            widget.render(size, frame.buffer_mut());
        }

        if let Some(ref dialog) = self.ui.input_dialog {
            let widget = InputDialogWidget { state: dialog };
            widget.render(size, frame.buffer_mut());
        }

        if let Some(ref mut picker) = self.ui.picker_state {
            PickerWidget::render_mut(picker, size, frame.buffer_mut());
        }

        if self.ui.input_mode == InputMode::GlobalSearch {
            self.search_state.global_visible_height =
                crate::render::global_search::global_search_results_height(size);

            let overlay = GlobalSearchOverlay {
                state: &self.search_state,
            };
            overlay.render(size, frame.buffer_mut());
        }
    }
}
