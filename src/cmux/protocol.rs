use std::process::Stdio;

use tokio::process::Command;

/// Wraps cmux CLI calls using `cmux --json` mode.
pub struct CmuxCli;

#[derive(Debug)]
pub struct SplitResult {
    pub surface_ref: String,
    pub pane_ref: String,
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
            anyhow::bail!("cmux failed: {}", stderr);
        }

        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    }

    /// Create a new split pane in the given direction.
    pub async fn new_split(direction: &str) -> anyhow::Result<SplitResult> {
        let stdout = Self::run(&["--json", "new-split", direction]).await?;

        // Parse JSON: {"surface_ref": "surface:23", "pane_ref": "pane:27", ...}
        let surface_ref = extract_json_string(&stdout, "surface_ref")
            .ok_or_else(|| anyhow::anyhow!("missing surface_ref in cmux output"))?;
        let pane_ref = extract_json_string(&stdout, "pane_ref")
            .ok_or_else(|| anyhow::anyhow!("missing pane_ref in cmux output"))?;

        Ok(SplitResult {
            surface_ref,
            pane_ref,
        })
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

    /// Get info about the current surface.
    pub async fn identify() -> anyhow::Result<String> {
        Self::run(&["--json", "identify"]).await
    }
}

/// Simple JSON string field extraction without pulling in serde_json.
fn extract_json_string(json: &str, key: &str) -> Option<String> {
    let pattern = format!("\"{}\"", key);
    let start = json.find(&pattern)?;
    let after_key = &json[start + pattern.len()..];
    // Skip whitespace and colon
    let after_colon = after_key.trim_start().strip_prefix(':')?;
    let after_ws = after_colon.trim_start();
    // Extract quoted string value
    let value_start = after_ws.strip_prefix('"')?;
    let end = value_start.find('"')?;
    Some(value_start[..end].to_string())
}
