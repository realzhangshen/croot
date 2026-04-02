use std::path::Path;
use std::sync::OnceLock;

use tree_sitter::Language;
use tree_sitter_highlight::HighlightConfiguration;

use super::capture_map::recognized_capture_names;

#[derive(Clone, Copy)]
pub struct LanguageDefinition {
    pub name: &'static str,
    pub aliases: &'static [&'static str],
    pub extensions: &'static [&'static str],
    pub config: fn() -> &'static HighlightConfiguration,
}

const LANGUAGE_DEFINITIONS: [LanguageDefinition; 6] = [
    LanguageDefinition {
        name: "rust",
        aliases: &["rust", "rs"],
        extensions: &["rs"],
        config: rust_config,
    },
    LanguageDefinition {
        name: "typescript",
        aliases: &["typescript", "ts"],
        extensions: &["ts", "mts", "cts"],
        config: typescript_config,
    },
    LanguageDefinition {
        name: "tsx",
        aliases: &["tsx", "typescriptreact", "typescript-react"],
        extensions: &["tsx"],
        config: tsx_config,
    },
    LanguageDefinition {
        name: "javascript",
        aliases: &["javascript", "js", "jsx"],
        extensions: &["js", "mjs", "cjs", "jsx"],
        config: javascript_config,
    },
    LanguageDefinition {
        name: "json",
        aliases: &["json"],
        extensions: &["json"],
        config: json_config,
    },
    LanguageDefinition {
        name: "markdown",
        aliases: &["markdown", "md"],
        extensions: &["md", "markdown"],
        config: markdown_config,
    },
];

pub fn find_by_path(path: &Path) -> Option<&'static LanguageDefinition> {
    let ext = path.extension()?.to_str()?.to_ascii_lowercase();
    LANGUAGE_DEFINITIONS
        .iter()
        .find(|def| def.extensions.iter().any(|candidate| *candidate == ext))
}

pub fn find_by_token(token: &str) -> Option<&'static LanguageDefinition> {
    let token = token.trim().to_ascii_lowercase();
    LANGUAGE_DEFINITIONS
        .iter()
        .find(|def| def.aliases.iter().any(|candidate| *candidate == token))
}

fn configure_highlights(mut config: HighlightConfiguration) -> HighlightConfiguration {
    let recognized = recognized_capture_names();
    config.configure(&recognized);
    config
}

fn rust_config() -> &'static HighlightConfiguration {
    static CONFIG: OnceLock<HighlightConfiguration> = OnceLock::new();
    CONFIG.get_or_init(|| {
        configure_highlights(
            HighlightConfiguration::new(
                language_from(tree_sitter_rust::LANGUAGE),
                "rust",
                tree_sitter_rust::HIGHLIGHTS_QUERY,
                tree_sitter_rust::INJECTIONS_QUERY,
                "",
            )
            .expect("rust highlight configuration should load"),
        )
    })
}

fn typescript_config() -> &'static HighlightConfiguration {
    static CONFIG: OnceLock<HighlightConfiguration> = OnceLock::new();
    CONFIG.get_or_init(|| {
        configure_highlights(
            HighlightConfiguration::new(
                language_from(tree_sitter_typescript::LANGUAGE_TYPESCRIPT),
                "typescript",
                tree_sitter_typescript::HIGHLIGHTS_QUERY,
                "",
                tree_sitter_typescript::LOCALS_QUERY,
            )
            .expect("typescript highlight configuration should load"),
        )
    })
}

fn tsx_config() -> &'static HighlightConfiguration {
    static CONFIG: OnceLock<HighlightConfiguration> = OnceLock::new();
    CONFIG.get_or_init(|| {
        configure_highlights(
            HighlightConfiguration::new(
                language_from(tree_sitter_typescript::LANGUAGE_TSX),
                "tsx",
                tree_sitter_typescript::HIGHLIGHTS_QUERY,
                "",
                tree_sitter_typescript::LOCALS_QUERY,
            )
            .expect("tsx highlight configuration should load"),
        )
    })
}

fn javascript_config() -> &'static HighlightConfiguration {
    static CONFIG: OnceLock<HighlightConfiguration> = OnceLock::new();
    CONFIG.get_or_init(|| {
        let highlights = format!(
            "{}\n{}",
            tree_sitter_javascript::HIGHLIGHT_QUERY,
            tree_sitter_javascript::JSX_HIGHLIGHT_QUERY
        );
        configure_highlights(
            HighlightConfiguration::new(
                language_from(tree_sitter_javascript::LANGUAGE),
                "javascript",
                &highlights,
                tree_sitter_javascript::INJECTIONS_QUERY,
                tree_sitter_javascript::LOCALS_QUERY,
            )
            .expect("javascript highlight configuration should load"),
        )
    })
}

fn json_config() -> &'static HighlightConfiguration {
    static CONFIG: OnceLock<HighlightConfiguration> = OnceLock::new();
    CONFIG.get_or_init(|| {
        configure_highlights(
            HighlightConfiguration::new(
                language_from(tree_sitter_json::LANGUAGE),
                "json",
                tree_sitter_json::HIGHLIGHTS_QUERY,
                "",
                "",
            )
            .expect("json highlight configuration should load"),
        )
    })
}

fn markdown_config() -> &'static HighlightConfiguration {
    static CONFIG: OnceLock<HighlightConfiguration> = OnceLock::new();
    CONFIG.get_or_init(|| {
        configure_highlights(
            HighlightConfiguration::new(
                language_from(tree_sitter_md::LANGUAGE),
                "markdown",
                tree_sitter_md::HIGHLIGHT_QUERY_BLOCK,
                tree_sitter_md::INJECTION_QUERY_BLOCK,
                "",
            )
            .expect("markdown highlight configuration should load"),
        )
    })
}

fn language_from(language: tree_sitter_language::LanguageFn) -> Language {
    language.into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_typescript_by_extension() {
        let def = find_by_path(Path::new("example.ts")).unwrap();
        assert_eq!(def.name, "typescript");
    }

    #[test]
    fn finds_jsx_by_token_alias() {
        let def = find_by_token("jsx").unwrap();
        assert_eq!(def.name, "javascript");
    }
}
