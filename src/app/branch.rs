use super::*;

impl App {
    pub(super) fn open_branch_picker(&mut self) {
        let Some(ref git) = self.git else { return };
        let branches = crate::git::branches::list_branches(git.repo_root());
        self.ui.picker_state = Some(PickerState::new_branch(&branches));
        self.ui.input_mode = InputMode::Picker;
    }

    pub(super) fn confirm_picker(&mut self) {
        let Some(picker) = self.ui.picker_state.take() else {
            return;
        };

        let Some(item) = picker.selected_item().cloned() else {
            self.ui.input_mode = InputMode::Normal;
            return;
        };

        // Don't switch to the already-current branch
        if item.is_current {
            self.ui.input_mode = InputMode::Normal;
            return;
        }

        let Some(ref git) = self.git else {
            self.ui.input_mode = InputMode::Normal;
            return;
        };

        // Prevent triggering a new switch while one is in-flight
        if self.branch_switch_rx.is_some() {
            self.ui.input_mode = InputMode::Normal;
            self.show_error("Branch switch already in progress".to_string());
            return;
        }

        let repo_root = git.repo_root().to_path_buf();
        let branch_data = item.data.clone();
        let is_remote = item.is_remote;

        let (tx, rx) = mpsc::channel::<BranchSwitchResult>(1);
        self.branch_switch_rx = Some(rx);
        self.ui.input_mode = InputMode::Normal;

        tokio::task::spawn_blocking(move || {
            let result = if is_remote {
                std::process::Command::new("git")
                    .arg("-C")
                    .arg(&repo_root)
                    .arg("switch")
                    .arg("--track")
                    .arg(&branch_data)
                    .output()
            } else {
                std::process::Command::new("git")
                    .arg("-C")
                    .arg(&repo_root)
                    .arg("switch")
                    .arg(&branch_data)
                    .output()
            };

            let (success, stderr) = match result {
                Ok(output) if output.status.success() => (true, String::new()),
                Ok(output) => (
                    false,
                    String::from_utf8_lossy(&output.stderr).trim().to_string(),
                ),
                Err(e) => (false, format!("git: {e}")),
            };
            let _ = tx.blocking_send(BranchSwitchResult {
                success,
                stderr,
                repo_root,
            });
        });
    }
}
