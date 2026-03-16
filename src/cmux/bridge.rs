use std::env;

/// Marker type indicating croot is running inside a cmux session.
/// Used for status bar display; no preview pane management.
pub struct CmuxBridge;

impl CmuxBridge {
    /// Detect if we're running inside a cmux session.
    pub fn detect() -> Option<Self> {
        let socket = env::var("CMUX_SOCKET_PATH").ok()?;
        if socket.is_empty() {
            return None;
        }
        Some(Self)
    }
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
}
