//! Unicode-width-aware string truncation helpers shared by the renderers.
//!
//! All three functions slice on valid char boundaries and treat multi-byte /
//! wide characters correctly. They differ only in which end of the string is
//! kept and whether an ellipsis is appended.

use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

/// Return the longest prefix of `s` that fits within `max_width` display columns.
pub(crate) fn truncate_to_display_width(s: &str, max_width: usize) -> String {
    let mut width = 0;
    let mut end = 0;
    for ch in s.chars() {
        let w = UnicodeWidthChar::width(ch).unwrap_or(0);
        if width + w > max_width {
            break;
        }
        width += w;
        end += ch.len_utf8();
    }
    s[..end].to_string()
}

/// Return the rightmost portion of `s` that fits within `max_width` display columns.
/// Useful for single-line text input, where we want to keep the recently-typed
/// tail visible.
pub(crate) fn truncate_start_to_display_width(s: &str, max_width: usize) -> String {
    let total = UnicodeWidthStr::width(s);
    if total <= max_width {
        return s.to_string();
    }
    let mut width = 0;
    let mut start_byte = s.len();
    for ch in s.chars().rev() {
        let cw = UnicodeWidthChar::width(ch).unwrap_or(0);
        if width + cw > max_width {
            break;
        }
        width += cw;
        start_byte -= ch.len_utf8();
    }
    s[start_byte..].to_string()
}

/// Return a prefix of `s` that fits within `max_width` columns, appending `'…'`
/// when truncation occurs. The ellipsis counts toward `max_width`.
pub(crate) fn truncate_with_ellipsis(s: &str, max_width: usize) -> String {
    if UnicodeWidthStr::width(s) <= max_width {
        return s.to_string();
    }
    let mut result = String::new();
    let mut width = 0;
    for ch in s.chars() {
        let cw = UnicodeWidthChar::width(ch).unwrap_or(0);
        if width + cw > max_width.saturating_sub(1) {
            result.push('…');
            break;
        }
        result.push(ch);
        width += cw;
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── truncate_start_to_display_width ────────────────────────────────

    #[test]
    fn truncate_start_ascii() {
        assert_eq!(truncate_start_to_display_width("abcdef", 4), "cdef");
    }

    #[test]
    fn truncate_start_multibyte() {
        // CJK characters are 2 display columns each
        let s = "你好世界"; // 4 chars × 2 = 8 columns
        assert_eq!(truncate_start_to_display_width(s, 4), "世界");
    }

    #[test]
    fn truncate_start_emoji() {
        // Emoji are typically 2 display columns
        let s = "hello🌍🌍";
        let result = truncate_start_to_display_width(s, 4);
        assert_eq!(result, "🌍🌍");
    }

    #[test]
    fn truncate_start_exact_fit() {
        assert_eq!(truncate_start_to_display_width("abc", 3), "abc");
        assert_eq!(truncate_start_to_display_width("abc", 10), "abc");
    }

    // ── truncate_to_display_width (Bug 1: unicode boundary safety) ─────

    #[test]
    fn truncate_to_display_width_cjk_no_panic() {
        // Simulates an error message with CJK chars being truncated to a narrow terminal
        let msg = "Error: 文件不存在 (file not found)";
        // Should not panic, even when width splits mid-character
        for w in 0..msg.len() + 5 {
            let _ = truncate_to_display_width(msg, w);
        }
    }

    #[test]
    fn truncate_to_display_width_emoji_no_panic() {
        let msg = "Error: 🔥🔥🔥 something failed";
        for w in 0..msg.len() + 5 {
            let _ = truncate_to_display_width(msg, w);
        }
    }

    #[test]
    fn truncate_to_display_width_respects_columns() {
        // "你好" = 4 display columns (2 each), 6 bytes
        assert_eq!(truncate_to_display_width("你好", 3), "你"); // Only first fits
        assert_eq!(truncate_to_display_width("你好", 4), "你好");
        assert_eq!(truncate_to_display_width("你好", 10), "你好");
    }

    // ── truncate_with_ellipsis ─────────────────────────────────────────

    #[test]
    fn truncate_with_ellipsis_no_truncation() {
        assert_eq!(truncate_with_ellipsis("abc", 3), "abc");
        assert_eq!(truncate_with_ellipsis("abc", 10), "abc");
    }

    #[test]
    fn truncate_with_ellipsis_ascii() {
        // Width 4 leaves room for 3 chars + '…'
        assert_eq!(truncate_with_ellipsis("abcdef", 4), "abc…");
    }

    #[test]
    fn truncate_with_ellipsis_cjk() {
        // "你好世界" = 8 columns; at width 5 we keep "你好" + '…' = 5 columns
        assert_eq!(truncate_with_ellipsis("你好世界", 5), "你好…");
    }

    #[test]
    fn truncate_with_ellipsis_does_not_panic_on_small_width() {
        // max_width = 0 or 1 should still return something sensible
        for w in 0..3 {
            let _ = truncate_with_ellipsis("hello", w);
        }
    }
}
