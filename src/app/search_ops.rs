use super::*;

impl App {
    /// Re-compute search state after structural changes (expand/collapse/refresh).
    pub(super) fn refresh_search_state(&mut self) {
        if self.search_state.query.is_empty() {
            self.search_state.match_indices.clear();
            self.search_state.visible_indices.clear();
            self.search_state.current_match = 0;
            return;
        }
        match self.search_state.mode {
            SearchMode::Find => self.update_find_matches(),
            SearchMode::Filter => self.update_filter_view(),
            SearchMode::Global => {} // global search doesn't depend on tree structure
        }
    }

    /// Find mode: compute `match_indices` (highlight only, no filtering).
    pub(super) fn update_find_matches(&mut self) {
        // ALWAYS clear positions -- node indices shift on expand/collapse
        self.search_state.match_char_positions.clear();

        if self.search_state.query.is_empty() {
            self.search_state.match_indices.clear();
            self.search_state.current_match = 0;
            return;
        }

        let (query, match_mode) = self.search_state.effective_query();
        let re = self.search_state.compiled_regex.take();
        let displayable = self.tree.build_displayable_indices();

        let mut matches = Vec::new();
        for idx in displayable {
            let display_name = self.tree.compact_display_name_for(idx);
            if let Some(positions) =
                do_match_positions(match_mode, &query, re.as_ref(), &display_name)
            {
                matches.push(idx);
                self.search_state
                    .match_char_positions
                    .insert(idx, positions);
            } else {
                // Fall back to path match (no character positions -- renderer uses FullName)
                let rel_path = self.tree.nodes[idx]
                    .path
                    .strip_prefix(&self.root)
                    .unwrap_or(&self.tree.nodes[idx].path)
                    .to_string_lossy()
                    .into_owned();
                if do_match(match_mode, &query, re.as_ref(), &rel_path) {
                    matches.push(idx);
                }
            }
        }

        self.search_state.compiled_regex = re;
        self.search_state.match_indices = matches;

        // Jump cursor to the closest match to origin_cursor
        if self.search_state.match_indices.is_empty() {
            self.search_state.current_match = 0;
        } else {
            let origin = self.search_state.origin_cursor;
            #[allow(clippy::cast_possible_wrap)]
            let closest = self
                .search_state
                .match_indices
                .iter()
                .enumerate()
                .min_by_key(|(_, &idx)| (idx as isize - origin as isize).unsigned_abs())
                .map_or(0, |(i, _)| i);
            self.search_state.current_match = closest;
            self.tree.cursor = self.search_state.match_indices[closest];
        }
    }

    /// Filter mode: compute `match_indices` and `visible_indices` (matches + ancestors).
    pub(super) fn update_filter_view(&mut self) {
        if self.search_state.query.is_empty() {
            self.search_state.match_indices.clear();
            self.search_state.visible_indices.clear();
            self.search_state.current_match = 0;
            return;
        }

        let (query, match_mode) = self.search_state.effective_query();
        let re = self.search_state.compiled_regex.take();
        let displayable = self.tree.build_displayable_indices();

        // Step 1: find matching displayable nodes
        let mut matches = Vec::new();
        for idx in displayable {
            let target_name = self.tree.compact_display_name_for(idx);
            let rel_path = self.tree.nodes[idx]
                .path
                .strip_prefix(&self.root)
                .unwrap_or(&self.tree.nodes[idx].path)
                .to_string_lossy()
                .into_owned();
            if do_match(match_mode, &query, re.as_ref(), &rel_path)
                || do_match(match_mode, &query, re.as_ref(), &target_name)
            {
                matches.push(idx);
            }
        }

        self.search_state.compiled_regex = re;

        // Step 2: collect ancestors for each match
        let mut visible_set = std::collections::HashSet::new();
        for &match_idx in &matches {
            visible_set.insert(match_idx);
            // Walk up the tree to find ancestors: scan backward from match_idx,
            // finding the closest node at each decreasing depth level.
            let match_depth = self.tree.nodes[match_idx].depth;
            if match_depth > 0 {
                let mut target_depth = match_depth - 1;
                for i in (0..match_idx).rev() {
                    if self.tree.nodes[i].depth == target_depth {
                        visible_set.insert(i);
                        if target_depth == 0 {
                            break;
                        }
                        target_depth -= 1;
                    } else if self.tree.nodes[i].depth < target_depth {
                        // Skipped a depth level -- adjust target and include this node
                        target_depth = self.tree.nodes[i].depth;
                        visible_set.insert(i);
                        if target_depth == 0 {
                            break;
                        }
                        target_depth -= 1;
                    }
                }
            }
        }

        // Step 3: sort visible set
        let mut visible: Vec<usize> = visible_set.into_iter().collect();
        visible.sort_unstable();

        self.search_state.match_indices = matches;
        self.search_state.visible_indices = visible;

        // Move cursor to first match if not already on one
        if !self.search_state.match_indices.is_empty()
            && !self.search_state.match_indices.contains(&self.tree.cursor)
        {
            self.tree.cursor = self.search_state.match_indices[0];
            self.search_state.current_match = 0;
        }
    }

    pub(super) fn search_navigate_next(&mut self) {
        if self.search_state.match_indices.is_empty() {
            return;
        }
        let len = self.search_state.match_indices.len();
        let next = (self.search_state.current_match + 1) % len;
        self.search_state.current_match = next;
        self.tree.cursor = self.search_state.match_indices[next];
    }

    pub(super) fn search_navigate_prev(&mut self) {
        if self.search_state.match_indices.is_empty() {
            return;
        }
        let len = self.search_state.match_indices.len();
        let prev = if self.search_state.current_match == 0 {
            len - 1
        } else {
            self.search_state.current_match - 1
        };
        self.search_state.current_match = prev;
        self.tree.cursor = self.search_state.match_indices[prev];
    }

    /// Spawn an async global search (fd or rg) with debounce via `SearchJob`.
    pub(super) fn spawn_global_search(&mut self, search_tx: &mpsc::Sender<SearchBatch>) {
        self.abort_global_search_task(false);

        if self.search_state.query.is_empty() {
            return;
        }

        self.search_state.request_id = self.search_state.request_id.wrapping_add(1);
        self.search_state.global_loading = true;

        self.global_search_job = Some(SearchJob::spawn(
            self.search_state.request_id,
            self.search_state.query.clone(),
            self.search_state.global_search_type,
            self.root.clone(),
            self.config.search.fd_command.clone(),
            self.config.search.rg_command.clone(),
            self.config.search.max_results,
            search_tx.clone(),
            200,
        ));
    }

    pub(super) fn abort_global_search_task(&mut self, invalidate_request_id: bool) {
        if invalidate_request_id {
            self.search_state.request_id = self.search_state.request_id.wrapping_add(1);
        }
        if let Some(job) = self.global_search_job.take() {
            job.cancel();
        }
    }

    /// Handle Enter in content search: toggle file header or navigate to match line.
    pub(super) fn handle_content_search_confirm(&mut self) -> PostAction {
        let Some(item) = self
            .search_state
            .resolve_item(self.search_state.global_selected)
        else {
            return PostAction::None;
        };
        match item {
            GroupedItem::FileHeader(g) => {
                let header_idx = self.search_state.flat_index_of_header(g);
                let match_count = self.search_state.grouped_results[g].matches.len();
                // If selection is inside this group's matches, remap to header
                if !self.search_state.grouped_results[g].collapsed
                    && self.search_state.global_selected > header_idx
                    && self.search_state.global_selected <= header_idx + match_count
                {
                    self.search_state.global_selected = header_idx;
                }
                self.search_state.grouped_results[g].collapsed =
                    !self.search_state.grouped_results[g].collapsed;
                self.search_state.clamp_selection();
                PostAction::None
            }
            GroupedItem::MatchLine(g, m) => {
                let group = &self.search_state.grouped_results[g];
                let path = group.path.clone();
                let line = group.matches[m].line;
                self.abort_global_search_task(true);
                self.ui.input_mode = InputMode::Normal;
                self.search_state.clear();
                self.search_open_action(path, line)
            }
        }
    }

    /// Return the appropriate `PostAction` for opening a search result,
    /// based on the configured `search.open_mode`.
    pub(super) fn search_open_action(&self, path: PathBuf, line: Option<usize>) -> PostAction {
        match self.config.search.open_mode {
            crate::config::SearchOpenMode::External => PostAction::OpenExternalEditor(path, line),
            crate::config::SearchOpenMode::Editor => PostAction::OpenEditor(path, line),
        }
    }

    /// Navigate to the selected search result in the file tree (Tab key).
    pub(super) fn handle_content_search_goto(
        &mut self,
        preview_tx: &mpsc::Sender<(u64, PathBuf, LoadedPreview)>,
    ) {
        let Some(item) = self
            .search_state
            .resolve_item(self.search_state.global_selected)
        else {
            return;
        };
        match item {
            GroupedItem::FileHeader(g) => {
                let path = self.search_state.grouped_results[g].path.clone();
                self.abort_global_search_task(true);
                self.ui.input_mode = InputMode::Normal;
                self.search_state.clear();
                self.tree.navigate_to_path(&path);
                self.reapply_git();
                self.trigger_preview_load(preview_tx);
            }
            GroupedItem::MatchLine(g, m) => {
                let group = &self.search_state.grouped_results[g];
                let path = group.path.clone();
                let line = group.matches[m].line;
                self.abort_global_search_task(true);
                self.ui.input_mode = InputMode::Normal;
                if let Some(rg_line) = line {
                    self.preview.pending_line = Some((path.clone(), rg_line));
                }
                self.search_state.clear();
                self.tree.navigate_to_path(&path);
                self.reapply_git();
                self.trigger_preview_load(preview_tx);
            }
        }
    }
}
