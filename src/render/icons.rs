use ratatui::style::Color;

use super::colors;

pub struct IconInfo {
    pub icon: &'static str,
    pub color: Color,
}

/// Get Nerd Font icon and color for a file extension.
pub fn icon_for_file(name: &str, is_dir: bool) -> IconInfo {
    if is_dir {
        return IconInfo {
            icon: "\u{f024b}",
            color: colors::dir_color(),
        };
    }

    // Check full filename first (higher priority than extension)
    let lower = name.to_ascii_lowercase();
    match lower.as_str() {
        "cargo.toml" | "cargo.lock" => {
            return IconInfo {
                icon: "\u{e7a8}",
                color: Color::Red,
            }
        }
        "dockerfile" => {
            return IconInfo {
                icon: "\u{f0868}",
                color: Color::LightBlue,
            }
        }
        "makefile" | "justfile" => {
            return IconInfo {
                icon: "\u{e779}",
                color: Color::Yellow,
            }
        }
        "license" | "licence" => {
            return IconInfo {
                icon: "\u{f0fc3}",
                color: Color::White,
            }
        }
        _ => {}
    }

    let ext = name.rsplit('.').next().unwrap_or("").to_ascii_lowercase();

    match ext.as_str() {
        // Systems/Compiled — Red
        "rs" => IconInfo {
            icon: "\u{e7a8}",
            color: Color::Red,
        },
        "c" | "h" => IconInfo {
            icon: "\u{e61e}",
            color: Color::Red,
        },
        "cpp" | "cxx" | "cc" | "hpp" => IconInfo {
            icon: "\u{e61d}",
            color: Color::Red,
        },
        "java" => IconInfo {
            icon: "\u{e738}",
            color: Color::Red,
        },
        "kt" | "kts" => IconInfo {
            icon: "\u{e634}",
            color: Color::Red,
        },
        "rb" => IconInfo {
            icon: "\u{e739}",
            color: Color::Red,
        },
        // Scripting/Dynamic — Yellow
        "js" | "mjs" | "cjs" => IconInfo {
            icon: "\u{e74e}",
            color: Color::Yellow,
        },
        "py" | "pyi" => IconInfo {
            icon: "\u{e73c}",
            color: Color::Yellow,
        },
        "lua" => IconInfo {
            icon: "\u{e620}",
            color: Color::Yellow,
        },
        "sh" | "bash" | "zsh" | "fish" => IconInfo {
            icon: "\u{e795}",
            color: Color::Yellow,
        },
        "zip" | "tar" | "gz" | "bz2" | "xz" | "7z" | "rar" => IconInfo {
            icon: "\u{f0187}",
            color: Color::Yellow,
        },
        // Typed/Modern — Blue
        "ts" | "mts" | "cts" => IconInfo {
            icon: "\u{e628}",
            color: Color::Blue,
        },
        "tsx" => IconInfo {
            icon: "\u{e7ba}",
            color: Color::Blue,
        },
        "go" => IconInfo {
            icon: "\u{e724}",
            color: Color::Blue,
        },
        "css" => IconInfo {
            icon: "\u{e749}",
            color: Color::Blue,
        },
        // Markup/Web — Cyan
        "html" | "htm" => IconInfo {
            icon: "\u{e736}",
            color: Color::Cyan,
        },
        "jsx" => IconInfo {
            icon: "\u{e7ba}",
            color: Color::Cyan,
        },
        "vue" => IconInfo {
            icon: "\u{f0844}",
            color: Color::Cyan,
        },
        "svelte" => IconInfo {
            icon: "\u{e697}",
            color: Color::Cyan,
        },
        "graphql" | "gql" => IconInfo {
            icon: "\u{e662}",
            color: Color::Cyan,
        },
        "wasm" => IconInfo {
            icon: "\u{e6a1}",
            color: Color::Cyan,
        },
        // Config/Data — Green
        "json" => IconInfo {
            icon: "\u{e60b}",
            color: Color::Green,
        },
        "yaml" | "yml" => IconInfo {
            icon: "\u{e6a8}",
            color: Color::Green,
        },
        "toml" => IconInfo {
            icon: "\u{e6b2}",
            color: Color::Green,
        },
        "xml" => IconInfo {
            icon: "\u{f05c0}",
            color: Color::Green,
        },
        "csv" => IconInfo {
            icon: "\u{f0219}",
            color: Color::Green,
        },
        "sql" => IconInfo {
            icon: "\u{f01bc}",
            color: Color::Green,
        },
        "env" => IconInfo {
            icon: "\u{f0614}",
            color: Color::Green,
        },
        // Documentation — White
        "md" | "mdx" => IconInfo {
            icon: "\u{e73e}",
            color: Color::White,
        },
        "txt" => IconInfo {
            icon: "\u{f0219}",
            color: Color::White,
        },
        "pdf" => IconInfo {
            icon: "\u{f0722}",
            color: Color::White,
        },
        // Media — Magenta
        "png" | "jpg" | "jpeg" | "gif" | "bmp" | "ico" | "webp" | "svg" => IconInfo {
            icon: "\u{f021f}",
            color: Color::Magenta,
        },
        // DevOps — LightBlue
        "ipynb" => IconInfo {
            icon: "\u{e678}",
            color: Color::LightBlue,
        },
        "scss" | "sass" => IconInfo {
            icon: "\u{e603}",
            color: Color::LightBlue,
        },
        // VCS/Meta — DarkGray
        "gitignore" | "gitmodules" | "gitattributes" => IconInfo {
            icon: "\u{e702}",
            color: Color::DarkGray,
        },
        "lock" => IconInfo {
            icon: "\u{f023a}",
            color: Color::DarkGray,
        },
        // Default — Reset
        _ => IconInfo {
            icon: "\u{f0214}",
            color: Color::Reset,
        },
    }
}

/// Icon for expanded/collapsed directory indicator.
pub fn dir_icon(expanded: bool) -> &'static str {
    if expanded {
        "\u{f0770}"
    } else {
        "\u{f024b}"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::style::Color;

    /// All ANSI 16 colors (the only ones allowed for built-in icon defaults).
    fn is_ansi16_or_reset(color: Color) -> bool {
        matches!(
            color,
            Color::Black
                | Color::Red
                | Color::Green
                | Color::Yellow
                | Color::Blue
                | Color::Magenta
                | Color::Cyan
                | Color::Gray
                | Color::DarkGray
                | Color::LightRed
                | Color::LightGreen
                | Color::LightYellow
                | Color::LightBlue
                | Color::LightMagenta
                | Color::LightCyan
                | Color::White
                | Color::Reset
        )
    }

    #[test]
    fn all_known_extensions_use_ansi16_colors() {
        let extensions = [
            "rs",
            "c",
            "h",
            "cpp",
            "cxx",
            "cc",
            "hpp",
            "java",
            "kt",
            "kts",
            "rb",
            "js",
            "mjs",
            "cjs",
            "py",
            "pyi",
            "lua",
            "sh",
            "bash",
            "zsh",
            "fish",
            "zip",
            "tar",
            "gz",
            "bz2",
            "xz",
            "7z",
            "rar",
            "ts",
            "mts",
            "cts",
            "tsx",
            "go",
            "css",
            "html",
            "htm",
            "jsx",
            "vue",
            "svelte",
            "graphql",
            "gql",
            "wasm",
            "json",
            "yaml",
            "yml",
            "toml",
            "xml",
            "csv",
            "sql",
            "env",
            "md",
            "mdx",
            "txt",
            "pdf",
            "png",
            "jpg",
            "jpeg",
            "gif",
            "bmp",
            "ico",
            "webp",
            "svg",
            "ipynb",
            "scss",
            "sass",
            "gitignore",
            "gitmodules",
            "gitattributes",
            "lock",
        ];
        for ext in &extensions {
            let name = format!("test.{ext}");
            let info = icon_for_file(&name, false);
            assert!(
                is_ansi16_or_reset(info.color),
                "Extension .{ext} uses non-ANSI-16 color: {:?}",
                info.color
            );
        }
    }

    #[test]
    fn all_known_basenames_use_ansi16_colors() {
        let basenames = [
            "Cargo.toml",
            "Cargo.lock",
            "Dockerfile",
            "Makefile",
            "justfile",
            "LICENSE",
            "LICENCE",
        ];
        for name in &basenames {
            let info = icon_for_file(name, false);
            assert!(
                is_ansi16_or_reset(info.color),
                "Basename {name} uses non-ANSI-16 color: {:?}",
                info.color
            );
        }
    }

    #[test]
    fn cargo_toml_returns_rust_icon_not_toml() {
        let info = icon_for_file("Cargo.toml", false);
        assert_eq!(info.color, Color::Red, "Cargo.toml should be Red (Rust)");
        assert_eq!(info.icon, "\u{e7a8}", "Cargo.toml should use Rust icon");
    }

    #[test]
    fn cargo_lock_returns_rust_icon_not_lock() {
        let info = icon_for_file("Cargo.lock", false);
        assert_eq!(info.color, Color::Red, "Cargo.lock should be Red (Rust)");
        assert_eq!(info.icon, "\u{e7a8}", "Cargo.lock should use Rust icon");
    }

    #[test]
    fn systems_compiled_category_is_red() {
        for ext in &["rs", "c", "h", "cpp", "java", "kt", "rb"] {
            let name = format!("file.{ext}");
            let info = icon_for_file(&name, false);
            assert_eq!(
                info.color,
                Color::Red,
                ".{ext} should be Red (Systems/Compiled)"
            );
        }
    }

    #[test]
    fn scripting_category_is_yellow() {
        for ext in &["js", "py", "lua", "sh", "bash"] {
            let name = format!("file.{ext}");
            let info = icon_for_file(&name, false);
            assert_eq!(
                info.color,
                Color::Yellow,
                ".{ext} should be Yellow (Scripting)"
            );
        }
    }

    #[test]
    fn unknown_extension_returns_reset() {
        let info = icon_for_file("file.xyz", false);
        assert_eq!(info.color, Color::Reset);
    }
}
