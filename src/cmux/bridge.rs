use anyhow::{bail, Context};
use std::env;
use std::path::Path;
use std::process::Command;

/// Bridge to cmux — enables opening editors in new tabs instead of suspending.
pub struct CmuxBridge {
    socket_path: String,
}

impl CmuxBridge {
    /// Detect if we're running inside a cmux session.
    pub fn detect() -> Option<Self> {
        let socket = env::var("CMUX_SOCKET_PATH").ok()?;
        if socket.is_empty() {
            return None;
        }
        Some(Self {
            socket_path: socket,
        })
    }

    /// Open a file in the editor via a new cmux surface (tab).
    pub fn open_in_editor(&self, editor_cmd: &str, path: &Path) -> anyhow::Result<()> {
        let surface_ref = self.create_surface()?;
        self.send_to_surface(&surface_ref, editor_cmd, path)
    }

    /// Run `cmux new-surface` and parse the surface reference from the output.
    fn create_surface(&self) -> anyhow::Result<String> {
        let output = Command::new("cmux")
            .arg("new-surface")
            .env("CMUX_SOCKET_PATH", &self.socket_path)
            .output()
            .context("failed to run `cmux new-surface`")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            bail!("cmux new-surface failed: {stderr}");
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        parse_surface_ref(&stdout)
    }

    /// Send the editor command to the given surface.
    fn send_to_surface(
        &self,
        surface_ref: &str,
        editor_cmd: &str,
        path: &Path,
    ) -> anyhow::Result<()> {
        let full_cmd = build_editor_command(editor_cmd, path);

        let output = Command::new("cmux")
            .args(["send", "--surface", surface_ref, &full_cmd])
            .env("CMUX_SOCKET_PATH", &self.socket_path)
            .output()
            .context("failed to run `cmux send`")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            bail!("cmux send failed: {stderr}");
        }

        Ok(())
    }
}

/// Build the shell command string to send to a cmux surface.
///
/// Splits the editor command (e.g. `"nvim --wait"`) into tokens, appends the file
/// path, then joins everything with proper shell quoting via `shell_words::join`.
fn build_editor_command(editor_cmd: &str, path: &Path) -> String {
    let mut parts = shell_words::split(editor_cmd).unwrap_or_else(|_| vec![editor_cmd.to_string()]);
    parts.push(path.to_string_lossy().into_owned());
    format!("{}\n", shell_words::join(&parts))
}

/// Parse the surface reference (e.g. `"surface:15"`) from `cmux new-surface` output.
///
/// Expected format: `"OK surface:15 pane:9 workspace:6"`
fn parse_surface_ref(output: &str) -> anyhow::Result<String> {
    output
        .split_whitespace()
        .find(|token| token.starts_with("surface:"))
        .map(str::to_string)
        .ok_or_else(|| anyhow::anyhow!("no surface reference found in cmux output: {output}"))
}

#[cfg(test)]
#[allow(deprecated)] // env::set_var/remove_var — tests serialized via ENV_LOCK
mod tests {
    use super::*;
    use std::sync::Mutex;

    // Serialize env-var tests to avoid races.
    static ENV_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn detect_returns_none_when_env_unset() {
        let _lock = ENV_LOCK.lock().unwrap();
        env::remove_var("CMUX_SOCKET_PATH");
        assert!(CmuxBridge::detect().is_none());
    }

    #[test]
    fn detect_returns_none_when_env_empty() {
        let _lock = ENV_LOCK.lock().unwrap();
        env::set_var("CMUX_SOCKET_PATH", "");
        let result = CmuxBridge::detect();
        env::remove_var("CMUX_SOCKET_PATH");
        assert!(result.is_none());
    }

    #[test]
    fn detect_returns_some_when_env_non_empty() {
        let _lock = ENV_LOCK.lock().unwrap();
        env::set_var("CMUX_SOCKET_PATH", "/tmp/cmux.sock");
        let result = CmuxBridge::detect();
        env::remove_var("CMUX_SOCKET_PATH");
        assert!(result.is_some());
    }

    #[test]
    fn detect_stores_socket_path() {
        let _lock = ENV_LOCK.lock().unwrap();
        env::set_var("CMUX_SOCKET_PATH", "/tmp/cmux.sock");
        let bridge = CmuxBridge::detect().unwrap();
        env::remove_var("CMUX_SOCKET_PATH");
        assert_eq!(bridge.socket_path, "/tmp/cmux.sock");
    }

    #[test]
    fn parse_surface_ref_valid() {
        let result = parse_surface_ref("OK surface:15 pane:9 workspace:6");
        assert_eq!(result.unwrap(), "surface:15");
    }

    #[test]
    fn parse_surface_ref_missing() {
        let result = parse_surface_ref("OK pane:9");
        assert!(result.is_err());
    }

    #[test]
    fn parse_surface_ref_empty() {
        let result = parse_surface_ref("");
        assert!(result.is_err());
    }

    // --- build_editor_command tests ---

    #[test]
    fn build_cmd_simple_path() {
        let cmd = build_editor_command("nvim", Path::new("/tmp/file.txt"));
        assert_eq!(cmd, "nvim /tmp/file.txt\n");
    }

    #[test]
    fn build_cmd_path_with_spaces() {
        let cmd = build_editor_command("nvim", Path::new("/tmp/my file.txt"));
        assert_eq!(cmd, "nvim '/tmp/my file.txt'\n");
    }

    #[test]
    fn build_cmd_path_with_single_quotes() {
        let cmd = build_editor_command("nvim", Path::new("/tmp/it's.txt"));
        // shell_words escapes the single quote inside single quotes
        assert!(cmd.starts_with("nvim "));
        assert!(cmd.contains("it"));
        assert!(cmd.contains("s.txt"));
        // Verify round-trip: parsing the command back should yield the original path
        let parts = shell_words::split(cmd.trim()).unwrap();
        assert_eq!(parts.last().unwrap(), "/tmp/it's.txt");
    }

    #[test]
    fn build_cmd_path_with_unicode() {
        let cmd = build_editor_command("nvim", Path::new("/tmp/日本語.txt"));
        assert_eq!(cmd, "nvim /tmp/日本語.txt\n");
    }

    #[test]
    fn build_cmd_multi_word_editor() {
        let cmd = build_editor_command("nvim --wait", Path::new("/tmp/file.txt"));
        assert_eq!(cmd, "nvim --wait /tmp/file.txt\n");
    }

    #[test]
    fn build_cmd_editor_with_quoted_args() {
        let cmd = build_editor_command("code --goto", Path::new("/tmp/my file.txt"));
        assert_eq!(cmd, "code --goto '/tmp/my file.txt'\n");
    }
}
