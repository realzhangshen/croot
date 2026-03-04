use std::env;

use super::protocol::CmuxCli;

/// Manages the preview pane lifecycle within a cmux session.
/// When cmux is not available, this is `None` and croot runs standalone.
pub struct CmuxBridge {
    preview_surface: Option<String>,
}

impl CmuxBridge {
    /// Detect if we're running inside a cmux session.
    pub fn detect() -> Option<Self> {
        let socket = env::var("CMUX_SOCKET_PATH").ok()?;
        if socket.is_empty() {
            return None;
        }
        Some(Self {
            preview_surface: None,
        })
    }

    /// Ensure a preview pane exists, creating one if needed. Returns surface ID.
    async fn ensure_preview_pane(&mut self) -> anyhow::Result<String> {
        if let Some(ref surface) = self.preview_surface {
            if self.is_preview_alive().await {
                return Ok(surface.clone());
            }
        }

        let result = CmuxCli::new_split("right").await?;
        self.preview_surface = Some(result.surface_ref.clone());
        Ok(result.surface_ref)
    }

    /// Send a command to the preview pane.
    /// Sends Ctrl-C first to cancel any running command, then the new command.
    pub async fn send_to_preview(&mut self, cmd: &str) -> anyhow::Result<()> {
        let surface = self.ensure_preview_pane().await?;

        // Send Ctrl-C to cancel previous command
        CmuxCli::send_text(&surface, "\x03").await?;

        // Small delay to let the previous command terminate
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;

        // Send the new command with newline
        let cmd_with_newline = format!("{cmd}\n");
        CmuxCli::send_text(&surface, &cmd_with_newline).await?;

        Ok(())
    }

    /// Check if the preview pane is still alive.
    async fn is_preview_alive(&self) -> bool {
        let Some(ref surface) = self.preview_surface else {
            return false;
        };

        match CmuxCli::list_panes().await {
            Ok(output) => output.contains(surface.as_str()),
            Err(_) => false,
        }
    }

    /// Close the preview pane on exit.
    pub async fn close_preview(&mut self) {
        if let Some(ref surface) = self.preview_surface {
            let _ = CmuxCli::send_text(surface, "exit\n").await;
            self.preview_surface = None;
        }
    }
}
