use std::path::Path;

/// Build the preview command for a given file path.
pub fn preview_command(path: &Path) -> String {
    let path_str = shell_escape(path);

    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();

    match ext.as_str() {
        // Images → chafa with kitty backend
        "png" | "jpg" | "jpeg" | "gif" | "bmp" | "webp" | "ico" | "svg" => {
            format!("chafa --format=kitty {path_str} 2>/dev/null || file {path_str}")
        }
        // Binary / archives → file info
        "zip" | "tar" | "gz" | "bz2" | "xz" | "7z" | "rar" | "exe" | "dll" | "so" | "dylib" => {
            format!("file {path_str} && ls -lh {path_str}")
        }
        // Everything else → bat with syntax highlighting
        _ => {
            format!(
                "bat --paging=always --style=numbers,grid --color=always {path_str} 2>/dev/null || cat {path_str}"
            )
        }
    }
}

/// Build the command to open a file in $EDITOR.
pub fn editor_command(path: &Path) -> String {
    let path_str = shell_escape(path);
    let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vim".into());
    format!("{editor} {path_str}")
}

/// Escape a path for shell use.
fn shell_escape(path: &Path) -> String {
    let s = path.to_string_lossy();
    // Wrap in single quotes, escaping any existing single quotes
    format!("'{}'", s.replace('\'', "'\\''"))
}
