use std::collections::{BTreeMap, HashMap};
use std::sync::OnceLock;

use ratatui::style::{Color, Modifier, Style};
use serde::{Deserialize, Serialize};

use crate::config::parse_color;

use super::semantic::SemanticToken;

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct SyntaxConfig {
    pub enabled: Option<bool>,
    #[serde(default)]
    pub tokens: BTreeMap<String, SyntaxTokenConfig>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct SyntaxTokenConfig {
    pub fg: Option<String>,
    pub bg: Option<String>,
    pub bold: Option<bool>,
    pub italic: Option<bool>,
    pub underline: Option<bool>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AnsiStyleSpec {
    pub fg: Option<Color>,
    pub bg: Option<Color>,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
}

impl AnsiStyleSpec {
    pub const fn plain() -> Self {
        Self {
            fg: None,
            bg: None,
            bold: false,
            italic: false,
            underline: false,
        }
    }

    pub const fn with_fg(self, fg: Color) -> Self {
        Self {
            fg: Some(fg),
            ..self
        }
    }

    pub const fn with_bg(self, bg: Color) -> Self {
        Self {
            bg: Some(bg),
            ..self
        }
    }

    pub const fn bold(self) -> Self {
        Self { bold: true, ..self }
    }

    pub const fn italic(self) -> Self {
        Self {
            italic: true,
            ..self
        }
    }

    pub const fn underline(self) -> Self {
        Self {
            underline: true,
            ..self
        }
    }

    pub fn merged_with(
        self,
        override_spec: &SyntaxTokenConfig,
        warnings: &mut Vec<String>,
    ) -> Self {
        let mut merged = self;

        if let Some(ref fg) = override_spec.fg {
            match parse_ansi_color(fg) {
                Ok(color) => merged.fg = Some(color),
                Err(msg) => warnings.push(format!("syntax.tokens.*.fg {msg}: {fg:?}")),
            }
        }

        if let Some(ref bg) = override_spec.bg {
            match parse_ansi_color(bg) {
                Ok(color) => merged.bg = Some(color),
                Err(msg) => warnings.push(format!("syntax.tokens.*.bg {msg}: {bg:?}")),
            }
        }

        if let Some(bold) = override_spec.bold {
            merged.bold = bold;
        }
        if let Some(italic) = override_spec.italic {
            merged.italic = italic;
        }
        if let Some(underline) = override_spec.underline {
            merged.underline = underline;
        }

        merged
    }

    pub fn to_style(self) -> Style {
        let mut style = Style::default();
        if let Some(fg) = self.fg {
            style = style.fg(fg);
        }
        if let Some(bg) = self.bg {
            style = style.bg(bg);
        }
        if self.bold {
            style = style.add_modifier(Modifier::BOLD);
        }
        if self.italic {
            style = style.add_modifier(Modifier::ITALIC);
        }
        if self.underline {
            style = style.add_modifier(Modifier::UNDERLINED);
        }
        style
    }
}

#[derive(Debug, Clone)]
pub struct SyntaxTheme {
    styles: HashMap<SemanticToken, AnsiStyleSpec>,
}

static ACTIVE_THEME: OnceLock<SyntaxTheme> = OnceLock::new();

impl Default for SyntaxTheme {
    fn default() -> Self {
        Self {
            styles: default_theme(),
        }
    }
}

impl SyntaxTheme {
    pub fn from_config(config: &SyntaxConfig) -> (Self, Vec<String>) {
        let mut warnings = Vec::new();
        let mut styles = default_theme();

        for (token_name, override_spec) in &config.tokens {
            let Ok(token) = token_name.parse::<SemanticToken>() else {
                warnings.push(format!(
                    "unknown syntax token name in config, falling back to text: {token_name:?}"
                ));
                continue;
            };

            let base = styles
                .get(&token)
                .copied()
                .unwrap_or_else(AnsiStyleSpec::plain);
            styles.insert(token, base.merged_with(override_spec, &mut warnings));
        }

        (Self { styles }, warnings)
    }

    pub fn spec_for(&self, token: SemanticToken) -> AnsiStyleSpec {
        self.styles
            .get(&token)
            .copied()
            .unwrap_or_else(AnsiStyleSpec::plain)
    }

    pub fn style_for(&self, token: SemanticToken) -> Style {
        self.spec_for(token).to_style()
    }
}

pub fn init(config: &SyntaxConfig) {
    let (theme, warnings) = SyntaxTheme::from_config(config);
    for warning in warnings {
        eprintln!("croot: warning: {warning}");
    }
    let _ = ACTIVE_THEME.set(theme);
}

pub fn active_theme() -> &'static SyntaxTheme {
    ACTIVE_THEME.get_or_init(SyntaxTheme::default)
}

pub fn parse_ansi_color(input: &str) -> Result<Color, &'static str> {
    match parse_color(input) {
        Some(Color::Rgb(..)) => Err("must be ANSI/indexed/reset, not RGB/hex"),
        Some(color) => Ok(color),
        None => Err("is not a recognized ANSI color"),
    }
}

fn default_theme() -> HashMap<SemanticToken, AnsiStyleSpec> {
    use SemanticToken::{
        Attribute, Comment, Constant, Constructor, Escape, Function, Keyword, Lifetime, Macro,
        Method, Module, Number, Operator, Parameter, Property, Punctuation, String, Tag, Text,
        Type, TypeBuiltin, Variable,
    };

    HashMap::from([
        (Text, AnsiStyleSpec::plain().with_fg(Color::Reset)),
        (
            Keyword,
            AnsiStyleSpec::plain().with_fg(Color::Magenta).bold(),
        ),
        (Type, AnsiStyleSpec::plain().with_fg(Color::Cyan)),
        (
            TypeBuiltin,
            AnsiStyleSpec::plain().with_fg(Color::LightCyan),
        ),
        (String, AnsiStyleSpec::plain().with_fg(Color::Green)),
        (Escape, AnsiStyleSpec::plain().with_fg(Color::LightGreen)),
        (Number, AnsiStyleSpec::plain().with_fg(Color::Yellow)),
        (Constant, AnsiStyleSpec::plain().with_fg(Color::LightYellow)),
        (
            Comment,
            AnsiStyleSpec::plain().with_fg(Color::DarkGray).italic(),
        ),
        (Function, AnsiStyleSpec::plain().with_fg(Color::Blue)),
        (Method, AnsiStyleSpec::plain().with_fg(Color::LightBlue)),
        (
            Constructor,
            AnsiStyleSpec::plain().with_fg(Color::LightCyan).bold(),
        ),
        (Variable, AnsiStyleSpec::plain().with_fg(Color::White)),
        (
            Parameter,
            AnsiStyleSpec::plain().with_fg(Color::White).italic(),
        ),
        (Property, AnsiStyleSpec::plain().with_fg(Color::LightBlue)),
        (
            Operator,
            AnsiStyleSpec::plain().with_fg(Color::LightMagenta),
        ),
        (Punctuation, AnsiStyleSpec::plain().with_fg(Color::Gray)),
        (Module, AnsiStyleSpec::plain().with_fg(Color::Blue).bold()),
        (Tag, AnsiStyleSpec::plain().with_fg(Color::Magenta)),
        (
            Attribute,
            AnsiStyleSpec::plain().with_fg(Color::Yellow).italic(),
        ),
        (Macro, AnsiStyleSpec::plain().with_fg(Color::LightRed)),
        (
            Lifetime,
            AnsiStyleSpec::plain().with_fg(Color::Red).italic(),
        ),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_ansi_color_accepts_ansi_and_indexed() {
        assert_eq!(parse_ansi_color("cyan"), Ok(Color::Cyan));
        assert_eq!(parse_ansi_color("indexed:240"), Ok(Color::Indexed(240)));
    }

    #[test]
    fn parse_ansi_color_rejects_hex() {
        assert_eq!(
            parse_ansi_color("#ff0000"),
            Err("must be ANSI/indexed/reset, not RGB/hex")
        );
    }

    #[test]
    fn theme_from_config_overrides_known_tokens() {
        let mut tokens = BTreeMap::new();
        tokens.insert(
            "type".to_string(),
            SyntaxTokenConfig {
                fg: Some("light_blue".to_string()),
                bold: Some(true),
                ..SyntaxTokenConfig::default()
            },
        );
        let config = SyntaxConfig {
            enabled: Some(true),
            tokens,
        };

        let (theme, warnings) = SyntaxTheme::from_config(&config);
        assert!(warnings.is_empty());
        let spec = theme.spec_for(SemanticToken::Type);
        assert_eq!(spec.fg, Some(Color::LightBlue));
        assert!(spec.bold);
    }

    #[test]
    fn default_theme_covers_all_tokens_with_distinct_styles() {
        let theme = default_theme();
        // Every token should have an entry
        for token in SemanticToken::ALL {
            assert!(
                theme.contains_key(&token),
                "default theme missing entry for {:?}",
                token
            );
        }
        // Every non-Text token should have fg or a modifier (not completely plain)
        for token in SemanticToken::ALL {
            if token == SemanticToken::Text {
                continue;
            }
            let spec = theme[&token];
            assert!(
                spec.fg.is_some() || spec.bold || spec.italic || spec.underline,
                "{:?} has no visual distinction from plain text",
                token
            );
        }
    }

    #[test]
    fn theme_from_config_warns_on_unknown_tokens_and_hex() {
        let mut tokens = BTreeMap::new();
        tokens.insert(
            "aaa_unknown_token".to_string(),
            SyntaxTokenConfig {
                fg: Some("#ff0000".to_string()),
                ..SyntaxTokenConfig::default()
            },
        );
        tokens.insert(
            "string".to_string(),
            SyntaxTokenConfig {
                fg: Some("#00ff00".to_string()),
                ..SyntaxTokenConfig::default()
            },
        );

        let (theme, warnings) = SyntaxTheme::from_config(&SyntaxConfig {
            enabled: None,
            tokens,
        });

        assert_eq!(theme.spec_for(SemanticToken::String).fg, Some(Color::Green));
        assert_eq!(warnings.len(), 2);
        assert!(warnings[0].contains("unknown syntax token"));
        assert!(warnings[1].contains("must be ANSI/indexed/reset"));
    }
}
