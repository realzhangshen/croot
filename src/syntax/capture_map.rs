use tree_sitter_highlight::Highlight;

use super::semantic::SemanticToken;

const CAPTURE_TOKEN_MAP: [(&str, SemanticToken); 43] = [
    ("attribute", SemanticToken::Attribute),
    ("boolean", SemanticToken::Keyword),
    ("carriage-return", SemanticToken::Punctuation),
    ("comment", SemanticToken::Comment),
    ("comment.documentation", SemanticToken::Comment),
    ("constant", SemanticToken::Number),
    ("constant.builtin", SemanticToken::Keyword),
    ("constructor", SemanticToken::Function),
    ("constructor.builtin", SemanticToken::Function),
    ("embedded", SemanticToken::Text),
    ("error", SemanticToken::Text),
    ("escape", SemanticToken::String),
    ("function", SemanticToken::Function),
    ("function.builtin", SemanticToken::Function),
    ("function.call", SemanticToken::Function),
    ("function.method", SemanticToken::Method),
    ("function.method.builtin", SemanticToken::Method),
    ("keyword", SemanticToken::Keyword),
    ("markup", SemanticToken::Tag),
    ("markup.bold", SemanticToken::Tag),
    ("markup.heading", SemanticToken::Tag),
    ("markup.italic", SemanticToken::Tag),
    ("markup.link", SemanticToken::String),
    ("markup.link.url", SemanticToken::String),
    ("markup.list", SemanticToken::Tag),
    ("markup.list.checked", SemanticToken::Tag),
    ("markup.list.numbered", SemanticToken::Tag),
    ("markup.list.unchecked", SemanticToken::Tag),
    ("markup.list.unnumbered", SemanticToken::Tag),
    ("markup.quote", SemanticToken::Tag),
    ("markup.raw", SemanticToken::String),
    ("markup.raw.block", SemanticToken::String),
    ("markup.raw.inline", SemanticToken::String),
    ("markup.strikethrough", SemanticToken::Tag),
    ("module", SemanticToken::Module),
    ("number", SemanticToken::Number),
    ("operator", SemanticToken::Operator),
    ("property", SemanticToken::Property),
    ("property.builtin", SemanticToken::Property),
    ("punctuation", SemanticToken::Punctuation),
    ("punctuation.bracket", SemanticToken::Punctuation),
    ("punctuation.delimiter", SemanticToken::Punctuation),
    ("punctuation.special", SemanticToken::Punctuation),
];

const EXTRA_CAPTURE_TOKEN_MAP: [(&str, SemanticToken); 13] = [
    ("string", SemanticToken::String),
    ("string.escape", SemanticToken::String),
    ("string.regexp", SemanticToken::String),
    ("string.special", SemanticToken::String),
    ("string.special.symbol", SemanticToken::String),
    ("tag", SemanticToken::Tag),
    ("type", SemanticToken::Type),
    ("type.builtin", SemanticToken::TypeBuiltin),
    ("variable", SemanticToken::Variable),
    ("variable.builtin", SemanticToken::Variable),
    ("variable.member", SemanticToken::Property),
    ("variable.parameter", SemanticToken::Parameter),
    ("label", SemanticToken::Attribute),
];

pub fn recognized_capture_names() -> Vec<&'static str> {
    CAPTURE_TOKEN_MAP
        .iter()
        .chain(EXTRA_CAPTURE_TOKEN_MAP.iter())
        .map(|(name, _)| *name)
        .collect()
}

pub fn token_for_highlight(highlight: Highlight) -> SemanticToken {
    CAPTURE_TOKEN_MAP
        .iter()
        .chain(EXTRA_CAPTURE_TOKEN_MAP.iter())
        .nth(highlight.0)
        .map(|(_, token)| *token)
        .unwrap_or(SemanticToken::Text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognized_capture_names_list_matches_token_lookup_order() {
        let names = recognized_capture_names();
        assert_eq!(names.len(), 56);
        assert_eq!(names[0], "attribute");
        assert_eq!(token_for_highlight(Highlight(0)), SemanticToken::Attribute);
        assert_eq!(
            token_for_highlight(Highlight(names.len() - 1)),
            SemanticToken::Attribute
        );
    }

    #[test]
    fn out_of_bounds_highlight_falls_back_to_text() {
        assert_eq!(token_for_highlight(Highlight(999)), SemanticToken::Text);
    }
}
