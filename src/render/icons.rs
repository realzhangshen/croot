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
            color: colors::DIR_COLOR,
        };
    }

    let ext = name.rsplit('.').next().unwrap_or("").to_ascii_lowercase();

    match ext.as_str() {
        // Rust
        "rs" => IconInfo {
            icon: "\u{e7a8}",
            color: Color::Red,
        },
        // JavaScript / TypeScript
        "js" | "mjs" | "cjs" => IconInfo {
            icon: "\u{e74e}",
            color: Color::Yellow,
        },
        "ts" | "mts" | "cts" => IconInfo {
            icon: "\u{e628}",
            color: Color::Blue,
        },
        "jsx" => IconInfo {
            icon: "\u{e7ba}",
            color: Color::Cyan,
        },
        "tsx" => IconInfo {
            icon: "\u{e7ba}",
            color: Color::Blue,
        },
        // Web
        "html" | "htm" => IconInfo {
            icon: "\u{e736}",
            color: Color::LightRed,
        },
        "css" => IconInfo {
            icon: "\u{e749}",
            color: Color::Blue,
        },
        "scss" | "sass" => IconInfo {
            icon: "\u{e603}",
            color: Color::LightMagenta,
        },
        "vue" => IconInfo {
            icon: "\u{f0844}",
            color: Color::Cyan,
        },
        "svelte" => IconInfo {
            icon: "\u{e697}",
            color: Color::LightRed,
        },
        // Config / Data
        "json" => IconInfo {
            icon: "\u{e60b}",
            color: Color::Yellow,
        },
        "yaml" | "yml" => IconInfo {
            icon: "\u{e6a8}",
            color: Color::LightYellow,
        },
        "toml" => IconInfo {
            icon: "\u{e6b2}",
            color: Color::LightYellow,
        },
        "xml" => IconInfo {
            icon: "\u{f05c0}",
            color: Color::LightRed,
        },
        "csv" => IconInfo {
            icon: "\u{f0219}",
            color: Color::Green,
        },
        // Python
        "py" | "pyi" => IconInfo {
            icon: "\u{e73c}",
            color: Color::Blue,
        },
        "ipynb" => IconInfo {
            icon: "\u{e678}",
            color: Color::LightBlue,
        },
        // Go
        "go" => IconInfo {
            icon: "\u{e724}",
            color: Color::Cyan,
        },
        // C / C++
        "c" | "h" => IconInfo {
            icon: "\u{e61e}",
            color: Color::LightBlue,
        },
        "cpp" | "cxx" | "cc" | "hpp" => IconInfo {
            icon: "\u{e61d}",
            color: Color::LightBlue,
        },
        // Java / Kotlin
        "java" => IconInfo {
            icon: "\u{e738}",
            color: Color::LightRed,
        },
        "kt" | "kts" => IconInfo {
            icon: "\u{e634}",
            color: Color::Magenta,
        },
        // Shell
        "sh" | "bash" | "zsh" | "fish" => IconInfo {
            icon: "\u{e795}",
            color: Color::Green,
        },
        // Lua
        "lua" => IconInfo {
            icon: "\u{e620}",
            color: Color::Blue,
        },
        // Ruby
        "rb" => IconInfo {
            icon: "\u{e739}",
            color: Color::Red,
        },
        // Markdown / Docs
        "md" | "mdx" => IconInfo {
            icon: "\u{e73e}",
            color: Color::LightCyan,
        },
        "txt" => IconInfo {
            icon: "\u{f0219}",
            color: Color::White,
        },
        "pdf" => IconInfo {
            icon: "\u{f0722}",
            color: Color::Red,
        },
        // Images
        "png" | "jpg" | "jpeg" | "gif" | "bmp" | "ico" | "webp" | "svg" => IconInfo {
            icon: "\u{f021f}",
            color: Color::Magenta,
        },
        // Git
        "gitignore" | "gitmodules" | "gitattributes" => IconInfo {
            icon: "\u{e702}",
            color: Color::LightRed,
        },
        // Docker
        "dockerfile" => IconInfo {
            icon: "\u{f0868}",
            color: Color::LightBlue,
        },
        // Lock files
        "lock" => IconInfo {
            icon: "\u{f023a}",
            color: Color::DarkGray,
        },
        // Env
        "env" => IconInfo {
            icon: "\u{f0614}",
            color: Color::Yellow,
        },
        // Zip / Archive
        "zip" | "tar" | "gz" | "bz2" | "xz" | "7z" | "rar" => IconInfo {
            icon: "\u{f0187}",
            color: Color::Yellow,
        },
        // Misc
        "sql" => IconInfo {
            icon: "\u{f01bc}",
            color: Color::White,
        },
        "graphql" | "gql" => IconInfo {
            icon: "\u{e662}",
            color: Color::Magenta,
        },
        "wasm" => IconInfo {
            icon: "\u{e6a1}",
            color: Color::LightMagenta,
        },
        _ => default_icon(name),
    }
}

fn default_icon(name: &str) -> IconInfo {
    // Special file names
    let lower = name.to_ascii_lowercase();
    match lower.as_str() {
        "dockerfile" => IconInfo {
            icon: "\u{f0868}",
            color: Color::LightBlue,
        },
        "makefile" | "justfile" => IconInfo {
            icon: "\u{e779}",
            color: Color::Green,
        },
        "cargo.toml" | "cargo.lock" => IconInfo {
            icon: "\u{e7a8}",
            color: Color::Red,
        },
        "license" | "licence" => IconInfo {
            icon: "\u{f0fc3}",
            color: Color::LightYellow,
        },
        _ => IconInfo {
            icon: "\u{f0214}",
            color: colors::DEFAULT_FG,
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
