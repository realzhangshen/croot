use std::process::Stdio;

use tokio::process::Command;

/// Wraps cmux CLI calls using `cmux --json` mode.
pub struct CmuxCli;

#[derive(Debug)]
pub struct SplitResult {
    pub surface_ref: String,
}

impl CmuxCli {
    /// Run a cmux command and return stdout.
    async fn run(args: &[&str]) -> anyhow::Result<String> {
        let output = Command::new("cmux")
            .args(args)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("cmux failed: {stderr}");
        }

        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    }

    /// Create a new split pane in the given direction.
    pub async fn new_split(direction: &str) -> anyhow::Result<SplitResult> {
        let stdout = Self::run(&["--json", "new-split", direction]).await?;

        let parsed: serde_json::Value = serde_json::from_str(&stdout)
            .map_err(|e| anyhow::anyhow!("failed to parse cmux JSON: {e}"))?;

        let surface_ref = parsed["surface_ref"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("missing surface_ref in cmux output"))?
            .to_string();

        Ok(SplitResult { surface_ref })
    }

    /// Send text to a specific surface.
    pub async fn send_text(surface: &str, text: &str) -> anyhow::Result<()> {
        Self::run(&["send", "--surface", surface, text]).await?;
        Ok(())
    }

    /// List panes to check if a surface is still alive.
    pub async fn list_panes() -> anyhow::Result<String> {
        Self::run(&["--json", "list-panes"]).await
    }
}
