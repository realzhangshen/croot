use super::semantic::SemanticToken;

/// TextMate scope prefix rules, ordered from most specific to least specific.
/// A scope matches a rule if it equals the prefix exactly, or starts with the
/// prefix followed by a `.` (so `"keyword"` matches `"keyword.control"` but
/// not `"keywords_extra"`).
const SCOPE_RULES: &[(&str, SemanticToken)] = &[
    // keyword (specific before general)
    ("keyword.operator", SemanticToken::Operator),
    ("keyword.control.directive", SemanticToken::Macro),
    ("keyword", SemanticToken::Keyword),
    // storage → Keyword
    ("storage", SemanticToken::Keyword),
    // constant (specific before general)
    ("constant.numeric", SemanticToken::Number),
    ("constant.character.escape", SemanticToken::Escape),
    ("constant.language", SemanticToken::Constant),
    ("constant", SemanticToken::Constant),
    // string
    ("string", SemanticToken::String),
    // comment
    ("comment", SemanticToken::Comment),
    // entity.name (specific before general)
    ("entity.name.function.method", SemanticToken::Method),
    (
        "entity.name.function.constructor",
        SemanticToken::Constructor,
    ),
    ("entity.name.function.macro", SemanticToken::Macro),
    ("entity.name.function", SemanticToken::Function),
    ("entity.name.type", SemanticToken::Type),
    ("entity.name.tag", SemanticToken::Tag),
    ("entity.name.section", SemanticToken::Tag),
    ("entity.other.attribute-name", SemanticToken::Attribute),
    ("entity.other.inherited-class", SemanticToken::Type),
    ("entity.name", SemanticToken::Variable),
    // variable (specific before general)
    ("variable.parameter", SemanticToken::Parameter),
    ("variable.function", SemanticToken::Function),
    ("variable.other.property", SemanticToken::Property),
    ("variable.other.member", SemanticToken::Property),
    ("variable", SemanticToken::Variable),
    // support (specific before general)
    ("support.function", SemanticToken::Function),
    ("support.type", SemanticToken::TypeBuiltin),
    ("support.class", SemanticToken::TypeBuiltin),
    ("support.constant", SemanticToken::Constant),
    ("support.module", SemanticToken::Module),
    ("support", SemanticToken::Variable),
    // preprocessor / macro directives
    ("meta.preprocessor", SemanticToken::Macro),
    // punctuation (specific before general)
    ("punctuation.definition.comment", SemanticToken::Comment),
    ("punctuation.definition.string", SemanticToken::String),
    ("punctuation.definition.tag", SemanticToken::Tag),
    ("punctuation", SemanticToken::Punctuation),
    // markup
    ("markup.raw", SemanticToken::String),
    ("markup", SemanticToken::Tag),
    // meta → transparent
    ("meta", SemanticToken::Text),
    // source → transparent
    ("source", SemanticToken::Text),
];

/// Map a TextMate scope name to a `SemanticToken` using longest-prefix matching.
///
/// A rule matches when either:
/// - `scope == prefix` (exact match), or
/// - `scope.starts_with(prefix)` and the character immediately following the
///   prefix is `'.'` (dotted sub-scope).
///
/// Returns `SemanticToken::Text` for empty or unknown scopes.
pub fn token_for_scope(scope: &str) -> SemanticToken {
    if scope.is_empty() {
        return SemanticToken::Text;
    }
    for &(prefix, token) in SCOPE_RULES {
        if scope == prefix {
            return token;
        }
        if scope.starts_with(prefix) {
            // The next char after the prefix must be '.' to avoid matching
            // "keywords_extra" with the "keyword" rule.
            if scope.as_bytes().get(prefix.len()) == Some(&b'.') {
                return token;
            }
        }
    }
    SemanticToken::Text
}

#[cfg(test)]
mod tests {
    use super::*;
    use SemanticToken::*;

    fn check(scope: &str, expected: SemanticToken) {
        assert_eq!(
            token_for_scope(scope),
            expected,
            "scope {:?} expected {:?}",
            scope,
            expected
        );
    }

    #[test]
    fn keyword_mappings() {
        check("keyword.control.rust", Keyword);
        check("keyword.operator.assignment", Operator);
        check("keyword.operator", Operator);
        check("keyword.other.fn.rust", Keyword);
        check("keyword", Keyword);
    }

    #[test]
    fn storage_mappings() {
        check("storage.type", Keyword);
        check("storage.modifier", Keyword);
    }

    #[test]
    fn string_mappings() {
        check("string.quoted.double", String);
        check("string.quoted.single", String);
        check("string.regexp", String);
    }

    #[test]
    fn constant_mappings() {
        check("constant.numeric.integer", Number);
        check("constant.numeric.float", Number);
        check("constant.numeric", Number);
        check("constant.character.escape", Escape);
        check("constant.language", Constant);
        check("constant.other", Constant);
    }

    #[test]
    fn entity_name_mappings() {
        check("entity.name.function", Function);
        check("entity.name.function.rust", Function);
        check("entity.name.function.method", Method);
        check("entity.name.function.method.python", Method);
        check("entity.name.function.constructor", Constructor);
        check("entity.name.function.macro", Macro);
        check("entity.name.type", Type);
        check("entity.name.type.class", Type);
        check("entity.name.tag", Tag);
        check("entity.name.tag.html", Tag);
        check("entity.name.section", Tag);
        check("entity.other.attribute-name", Attribute);
    }

    #[test]
    fn comment_mappings() {
        check("comment.line", Comment);
        check("comment.block", Comment);
        check("comment.block.documentation", Comment);
    }

    #[test]
    fn variable_mappings() {
        check("variable.other", Variable);
        check("variable.other.property", Property);
        check("variable.other.property.js", Property);
        check("variable.other.member", Property);
        check("variable.parameter", Parameter);
        check("variable.language", Variable);
        check("variable.function", Function);
    }

    #[test]
    fn support_mappings() {
        check("support.function", Function);
        check("support.type", TypeBuiltin);
        check("support.class", TypeBuiltin);
        check("support.constant", Constant);
        check("support.module", Module);
    }

    #[test]
    fn punctuation_mappings() {
        check("punctuation.definition.string", String);
        check("punctuation.separator", Punctuation);
        check("punctuation.section", Punctuation);
        check("punctuation.terminator", Punctuation);
        check("punctuation.accessor", Punctuation);
        check("punctuation.definition.comment", Comment);
        check("punctuation.definition.tag", Tag);
    }

    #[test]
    fn markup_mappings() {
        check("markup.heading", Tag);
        check("markup.bold", Tag);
        check("markup.italic", Tag);
        check("markup.raw", String);
        check("markup.raw.inline", String);
    }

    #[test]
    fn macro_and_preprocessor_mappings() {
        check("meta.preprocessor", Macro);
        check("meta.preprocessor.include", Macro);
        check("keyword.control.directive", Macro);
    }

    #[test]
    fn meta_mappings() {
        check("meta.function", Text);
        check("meta.block", Text);
    }

    #[test]
    fn fallback_cases() {
        check("", Text);
        check("totally.unknown.scope", Text);
    }

    #[test]
    fn no_false_prefix_match() {
        // "keywords_extra" must NOT match the "keyword" rule
        check("keywords_extra", Text);
        // "strings" must NOT match the "string" rule
        check("strings", Text);
    }
}
