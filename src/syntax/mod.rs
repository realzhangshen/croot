pub mod engine;
pub mod scope_map;
pub mod semantic;
pub mod theme;

use ratatui::style::Style;

/// A single styled text segment within a line.
///
/// Lives in `syntax` because it is the output type produced by the
/// highlighter and markdown renderer; `preview::state` consumes it.
pub type StyledSpan = (String, Style);
