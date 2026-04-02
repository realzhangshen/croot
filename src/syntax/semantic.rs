use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum SemanticToken {
    Text,
    Keyword,
    Type,
    TypeBuiltin,
    String,
    Escape,
    Number,
    Constant,
    Comment,
    Function,
    Method,
    Constructor,
    Variable,
    Parameter,
    Property,
    Operator,
    Punctuation,
    Module,
    Tag,
    Attribute,
    Macro,
    Lifetime,
}

impl SemanticToken {
    pub const ALL: [Self; 22] = [
        Self::Text,
        Self::Keyword,
        Self::Type,
        Self::TypeBuiltin,
        Self::String,
        Self::Escape,
        Self::Number,
        Self::Constant,
        Self::Comment,
        Self::Function,
        Self::Method,
        Self::Constructor,
        Self::Variable,
        Self::Parameter,
        Self::Property,
        Self::Operator,
        Self::Punctuation,
        Self::Module,
        Self::Tag,
        Self::Attribute,
        Self::Macro,
        Self::Lifetime,
    ];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Text => "text",
            Self::Keyword => "keyword",
            Self::Type => "type",
            Self::TypeBuiltin => "type_builtin",
            Self::String => "string",
            Self::Number => "number",
            Self::Comment => "comment",
            Self::Function => "function",
            Self::Method => "method",
            Self::Variable => "variable",
            Self::Parameter => "parameter",
            Self::Property => "property",
            Self::Operator => "operator",
            Self::Punctuation => "punctuation",
            Self::Module => "module",
            Self::Tag => "tag",
            Self::Attribute => "attribute",
            Self::Escape => "escape",
            Self::Constant => "constant",
            Self::Constructor => "constructor",
            Self::Macro => "macro",
            Self::Lifetime => "lifetime",
        }
    }
}

impl FromStr for SemanticToken {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "text" => Ok(Self::Text),
            "keyword" => Ok(Self::Keyword),
            "type" => Ok(Self::Type),
            "type_builtin" | "type-builtin" => Ok(Self::TypeBuiltin),
            "string" => Ok(Self::String),
            "number" => Ok(Self::Number),
            "comment" => Ok(Self::Comment),
            "function" => Ok(Self::Function),
            "method" => Ok(Self::Method),
            "variable" => Ok(Self::Variable),
            "parameter" => Ok(Self::Parameter),
            "property" => Ok(Self::Property),
            "operator" => Ok(Self::Operator),
            "punctuation" => Ok(Self::Punctuation),
            "module" => Ok(Self::Module),
            "tag" => Ok(Self::Tag),
            "attribute" => Ok(Self::Attribute),
            "escape" => Ok(Self::Escape),
            "constant" => Ok(Self::Constant),
            "constructor" => Ok(Self::Constructor),
            "macro" | "macro_call" | "macro-call" => Ok(Self::Macro),
            "lifetime" => Ok(Self::Lifetime),
            _ => Err(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn token_names_round_trip() {
        for token in SemanticToken::ALL {
            assert_eq!(token.as_str().parse::<SemanticToken>(), Ok(token));
        }
    }

    #[test]
    fn accepts_hyphenated_type_builtin() {
        assert_eq!(
            "type-builtin".parse::<SemanticToken>(),
            Ok(SemanticToken::TypeBuiltin)
        );
    }

    #[test]
    fn rejects_unknown_token() {
        assert!("unknown_token_xyz".parse::<SemanticToken>().is_err());
    }

    #[test]
    fn new_tokens_round_trip() {
        // These tokens are new — verify they parse and stringify
        for name in ["escape", "constant", "constructor", "macro", "lifetime"] {
            let token: SemanticToken = name.parse().expect(name);
            assert_eq!(token.as_str(), name);
        }
    }

    #[test]
    fn all_array_has_22_tokens() {
        assert_eq!(SemanticToken::ALL.len(), 22);
    }
}
