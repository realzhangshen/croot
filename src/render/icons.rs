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
            icon: "",
            color: colors::DIR_COLOR,
        };
    }

    let ext = name.rsplit('.').next().unwrap_or("").to_ascii_lowercase();

    match ext.as_str() {
        // Rust
        "rs" => IconInfo {
            icon: "",
            color: Color::Rgb(0xDE, 0xA5, 0x84),
        },
        // JavaScript / TypeScript
        "js" | "mjs" | "cjs" => IconInfo {
            icon: "",
            color: Color::Rgb(0xF1, 0xE0, 0x5A),
        },
        "ts" | "mts" | "cts" => IconInfo {
            icon: "",
            color: Color::Rgb(0x31, 0x78, 0xC6),
        },
        "jsx" => IconInfo {
            icon: "",
            color: Color::Rgb(0x61, 0xDA, 0xFB),
        },
        "tsx" => IconInfo {
            icon: "",
            color: Color::Rgb(0x31, 0x78, 0xC6),
        },
        // Web
        "html" | "htm" => IconInfo {
            icon: "",
            color: Color::Rgb(0xE4, 0x4D, 0x26),
        },
        "css" => IconInfo {
            icon: "",
            color: Color::Rgb(0x56, 0x3D, 0x7C),
        },
        "scss" | "sass" => IconInfo {
            icon: "",
            color: Color::Rgb(0xCD, 0x67, 0x99),
        },
        "vue" => IconInfo {
            icon: "󰡄",
            color: Color::Rgb(0x41, 0xB8, 0x83),
        },
        "svelte" => IconInfo {
            icon: "",
            color: Color::Rgb(0xFF, 0x3E, 0x00),
        },
        // Config / Data
        "json" => IconInfo {
            icon: "",
            color: Color::Rgb(0xF1, 0xE0, 0x5A),
        },
        "yaml" | "yml" => IconInfo {
            icon: "",
            color: Color::Rgb(0xCB, 0x17, 0x1E),
        },
        "toml" => IconInfo {
            icon: "",
            color: Color::Rgb(0x9C, 0x40, 0x21),
        },
        "xml" => IconInfo {
            icon: "󰗀",
            color: Color::Rgb(0xE4, 0x4D, 0x26),
        },
        "csv" => IconInfo {
            icon: "",
            color: Color::Rgb(0x89, 0xA0, 0x2C),
        },
        // Python
        "py" | "pyi" => IconInfo {
            icon: "",
            color: Color::Rgb(0x35, 0x72, 0xA5),
        },
        "ipynb" => IconInfo {
            icon: "",
            color: Color::Rgb(0xF3, 0x76, 0x26),
        },
        // Go
        "go" => IconInfo {
            icon: "",
            color: Color::Rgb(0x00, 0xAD, 0xD8),
        },
        // C / C++
        "c" | "h" => IconInfo {
            icon: "",
            color: Color::Rgb(0x55, 0x9B, 0xD4),
        },
        "cpp" | "cxx" | "cc" | "hpp" => IconInfo {
            icon: "",
            color: Color::Rgb(0x55, 0x9B, 0xD4),
        },
        // Java / Kotlin
        "java" => IconInfo {
            icon: "",
            color: Color::Rgb(0xB0, 0x72, 0x19),
        },
        "kt" | "kts" => IconInfo {
            icon: "",
            color: Color::Rgb(0x7F, 0x52, 0xFF),
        },
        // Shell
        "sh" | "bash" | "zsh" | "fish" => IconInfo {
            icon: "",
            color: Color::Rgb(0x89, 0xE0, 0x51),
        },
        // Lua
        "lua" => IconInfo {
            icon: "",
            color: Color::Rgb(0x00, 0x00, 0x80),
        },
        // Ruby
        "rb" => IconInfo {
            icon: "",
            color: Color::Rgb(0xCC, 0x34, 0x2D),
        },
        // Markdown / Docs
        "md" | "mdx" => IconInfo {
            icon: "",
            color: Color::Rgb(0x51, 0x9A, 0xBA),
        },
        "txt" => IconInfo {
            icon: "󰈙",
            color: Color::Rgb(0x89, 0xA0, 0x2C),
        },
        "pdf" => IconInfo {
            icon: "",
            color: Color::Rgb(0xBD, 0x00, 0x00),
        },
        // Images
        "png" | "jpg" | "jpeg" | "gif" | "bmp" | "ico" | "webp" | "svg" => IconInfo {
            icon: "",
            color: Color::Rgb(0xA0, 0x74, 0xC4),
        },
        // Git
        "gitignore" | "gitmodules" | "gitattributes" => IconInfo {
            icon: "",
            color: Color::Rgb(0xF0, 0x50, 0x33),
        },
        // Docker
        "dockerfile" => IconInfo {
            icon: "󰡨",
            color: Color::Rgb(0x38, 0x4D, 0x54),
        },
        // Lock files
        "lock" => IconInfo {
            icon: "",
            color: Color::Rgb(0x80, 0x80, 0x80),
        },
        // Env
        "env" => IconInfo {
            icon: "",
            color: Color::Rgb(0xFA, 0xF7, 0x43),
        },
        // Zip / Archive
        "zip" | "tar" | "gz" | "bz2" | "xz" | "7z" | "rar" => IconInfo {
            icon: "",
            color: Color::Rgb(0xDA, 0xA5, 0x20),
        },
        // Misc
        "sql" => IconInfo {
            icon: "",
            color: Color::Rgb(0xDA, 0xD8, 0xD8),
        },
        "graphql" | "gql" => IconInfo {
            icon: "",
            color: Color::Rgb(0xE1, 0x00, 0x98),
        },
        "wasm" => IconInfo {
            icon: "",
            color: Color::Rgb(0x65, 0x4F, 0xF0),
        },
        _ => default_icon(name),
    }
}

fn default_icon(name: &str) -> IconInfo {
    // Special file names
    let lower = name.to_ascii_lowercase();
    match lower.as_str() {
        "dockerfile" => IconInfo {
            icon: "󰡨",
            color: Color::Rgb(0x38, 0x4D, 0x54),
        },
        "makefile" | "justfile" => IconInfo {
            icon: "",
            color: Color::Rgb(0x6D, 0x8A, 0x88),
        },
        "cargo.toml" | "cargo.lock" => IconInfo {
            icon: "",
            color: Color::Rgb(0xDE, 0xA5, 0x84),
        },
        "license" | "licence" => IconInfo {
            icon: "󰿃",
            color: Color::Rgb(0xD0, 0xBF, 0x41),
        },
        _ => IconInfo {
            icon: "",
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
