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
