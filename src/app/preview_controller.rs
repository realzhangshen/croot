use std::path::PathBuf;

use tokio::task::JoinHandle;

use crate::layout::PreviewLayout;
use crate::preview::state::PreviewState;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingPreviewHighlight {
    pub path: PathBuf,
    pub line: usize,
    pub query: String,
}

/// All preview-pane state grouped in one struct. Methods still live on `App`
/// so they can reach `Config`, `FileTree`, etc. without extra plumbing.
pub struct PreviewController {
    /// Rendered content + scroll + selection state for the preview pane.
    pub state: PreviewState,
    /// Whether the preview pane is currently shown.
    pub visible: bool,
    /// Pending debounced load task. `.take().abort()` cancels a stale load
    /// when the cursor moves before the previous load fires.
    pub debounce_handle: Option<JoinHandle<()>>,
    /// X coordinate of the preview pane's left edge (for mouse routing).
    pub area_x: Option<u16>,
    /// Cached layout of the preview pane (gutter widths, content rect, etc).
    pub layout: Option<PreviewLayout>,
    /// Width available for text content inside the preview pane.
    pub content_width: u16,
    /// Monotonic counter for preview requests. Stale results are discarded
    /// on the receive side of the preview channel.
    pub generation: u64,
    /// Pending "scroll to line N of path P" request, used by content search
    /// to navigate to a match after the preview finishes loading.
    pub pending_line: Option<(PathBuf, usize)>,
    /// Pending "highlight the match on line N of path P" request.
    pub pending_highlight: Option<PendingPreviewHighlight>,
    /// Picker for encoding the terminal's image capability set.
    #[cfg(feature = "image-preview")]
    pub image_picker: Option<ratatui_image::picker::Picker>,
    /// Channel to the background image resize worker thread.
    #[cfg(feature = "image-preview")]
    pub resize_tx: Option<std::sync::mpsc::Sender<ratatui_image::thread::ResizeRequest>>,
    /// Channel receiving resized image protocols from the worker thread.
    #[cfg(feature = "image-preview")]
    pub resize_response_rx: Option<
        std::sync::mpsc::Receiver<
            Result<ratatui_image::thread::ResizeResponse, ratatui_image::errors::Errors>,
        >,
    >,
}

impl PreviewController {
    pub fn new(visible: bool, render_markdown: bool) -> Self {
        let mut state = PreviewState::new();
        state.render_markdown = render_markdown;
        Self {
            state,
            visible,
            debounce_handle: None,
            area_x: None,
            layout: None,
            content_width: 80,
            generation: 0,
            pending_line: None,
            pending_highlight: None,
            #[cfg(feature = "image-preview")]
            image_picker: None,
            #[cfg(feature = "image-preview")]
            resize_tx: None,
            #[cfg(feature = "image-preview")]
            resize_response_rx: None,
        }
    }
}
