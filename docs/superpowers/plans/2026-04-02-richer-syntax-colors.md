# Richer Syntax Highlighting Colors — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Enrich syntax highlighting by adding 5 new semantic tokens and utilizing all 16 ANSI colors so every code element gets a distinct visual style.

**Architecture:** Add `Escape`, `Constant`, `Constructor`, `Macro`, `Lifetime` to `SemanticToken` enum. Remap existing tree-sitter captures and add 3 new ones. Assign distinct ANSI colors to all 22 tokens (previously 3 used `Reset`). All changes are within `src/syntax/`.

**Tech Stack:** Rust, tree-sitter-highlight, ratatui (ANSI `Color` / `Style`)

---

### Task 1: Add new SemanticToken variants

**Files:**
- Modify: `src/syntax/semantic.rs:4-93`

- [ ] **Step 1: Write failing tests for new token variants**

Add these tests at the bottom of the existing `mod tests` block in `src/syntax/semantic.rs`:

```rust
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
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p croot --lib syntax::semantic::tests`
Expected: FAIL — `new_tokens_round_trip` fails because variants don't exist, `all_array_has_22_tokens` fails because `ALL.len()` is 17.

- [ ] **Step 3: Add the 5 new variants to the enum**

In `src/syntax/semantic.rs`, add 5 variants to the `SemanticToken` enum (after `Attribute`):

```rust
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
```

Update `ALL` to include all 22 variants:

```rust
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
```

Update `as_str()` — add arms after the existing ones:

```rust
Self::Escape => "escape",
Self::Constant => "constant",
Self::Constructor => "constructor",
Self::Macro => "macro",
Self::Lifetime => "lifetime",
```

Update `FromStr` — add arms:

```rust
"escape" => Ok(Self::Escape),
"constant" => Ok(Self::Constant),
"constructor" => Ok(Self::Constructor),
"macro" | "macro_call" | "macro-call" => Ok(Self::Macro),
"lifetime" => Ok(Self::Lifetime),
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p croot --lib syntax::semantic::tests`
Expected: ALL PASS (including existing `token_names_round_trip` which iterates `ALL`)

- [ ] **Step 5: Commit**

```bash
git add src/syntax/semantic.rs
git commit -m "feat(syntax): add Escape, Constant, Constructor, Macro, Lifetime tokens"
```

---

### Task 2: Update capture map with new token mappings

**Files:**
- Modify: `src/syntax/capture_map.rs:1-104`

- [ ] **Step 1: Write failing tests for new capture mappings**

Add these tests to the existing `mod tests` block in `src/syntax/capture_map.rs`:

```rust
#[test]
fn new_captures_map_to_correct_tokens() {
    let names = recognized_capture_names();
    // Find index of each new/remapped capture and verify its token
    let find = |name: &str| -> SemanticToken {
        let idx = names.iter().position(|n| *n == name).expect(name);
        token_for_highlight(Highlight(idx))
    };

    assert_eq!(find("escape"), SemanticToken::Escape);
    assert_eq!(find("constant"), SemanticToken::Constant);
    assert_eq!(find("constant.builtin"), SemanticToken::Constant);
    assert_eq!(find("constructor"), SemanticToken::Constructor);
    assert_eq!(find("constructor.builtin"), SemanticToken::Constructor);
    assert_eq!(find("function.macro"), SemanticToken::Macro);
    assert_eq!(find("keyword.directive"), SemanticToken::Macro);
    assert_eq!(find("lifetime"), SemanticToken::Lifetime);
}

#[test]
fn recognized_capture_names_has_59_entries() {
    assert_eq!(recognized_capture_names().len(), 59);
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p croot --lib syntax::capture_map::tests`
Expected: FAIL — `escape` still maps to `String`, count is still 56, new captures don't exist.

- [ ] **Step 3: Update the capture arrays**

Replace the two arrays in `src/syntax/capture_map.rs` with:

```rust
const CAPTURE_TOKEN_MAP: [(&str, SemanticToken); 46] = [
    ("attribute", SemanticToken::Attribute),
    ("boolean", SemanticToken::Keyword),
    ("carriage-return", SemanticToken::Punctuation),
    ("comment", SemanticToken::Comment),
    ("comment.documentation", SemanticToken::Comment),
    ("constant", SemanticToken::Constant),
    ("constant.builtin", SemanticToken::Constant),
    ("constructor", SemanticToken::Constructor),
    ("constructor.builtin", SemanticToken::Constructor),
    ("embedded", SemanticToken::Text),
    ("error", SemanticToken::Text),
    ("escape", SemanticToken::Escape),
    ("function", SemanticToken::Function),
    ("function.builtin", SemanticToken::Function),
    ("function.call", SemanticToken::Function),
    ("function.macro", SemanticToken::Macro),
    ("function.method", SemanticToken::Method),
    ("function.method.builtin", SemanticToken::Method),
    ("keyword", SemanticToken::Keyword),
    ("keyword.directive", SemanticToken::Macro),
    ("lifetime", SemanticToken::Lifetime),
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
    ("string.escape", SemanticToken::Escape),
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
```

Key changes from the original:
- `CAPTURE_TOKEN_MAP` grows from 43 to 46 entries (added `function.macro`, `keyword.directive`, `lifetime`)
- `constant` / `constant.builtin`: `Number`/`Keyword` -> `Constant`
- `constructor` / `constructor.builtin`: `Function` -> `Constructor`
- `escape`: `String` -> `Escape`
- `function.macro`: new -> `Macro`
- `keyword.directive`: new -> `Macro`
- `lifetime`: new -> `Lifetime`
- `string.escape`: `String` -> `Escape`
- `EXTRA_CAPTURE_TOKEN_MAP` stays at 13 entries (only `string.escape` token changed)

Also update the existing test assertion for the count:

```rust
#[test]
fn recognized_capture_names_list_matches_token_lookup_order() {
    let names = recognized_capture_names();
    assert_eq!(names.len(), 59);
    assert_eq!(names[0], "attribute");
    assert_eq!(token_for_highlight(Highlight(0)), SemanticToken::Attribute);
    assert_eq!(
        token_for_highlight(Highlight(names.len() - 1)),
        SemanticToken::Attribute,
    );
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p croot --lib syntax::capture_map::tests`
Expected: ALL PASS

- [ ] **Step 5: Commit**

```bash
git add src/syntax/capture_map.rs
git commit -m "feat(syntax): remap captures to new tokens and add macro/lifetime captures"
```

---

### Task 3: Update default theme with richer palette

**Files:**
- Modify: `src/syntax/theme.rs:204-238`

- [ ] **Step 1: Write failing test for complete theme coverage**

Add this test to the existing `mod tests` block in `src/syntax/theme.rs`:

```rust
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
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p croot --lib syntax::theme::tests::default_theme_covers_all_tokens`
Expected: FAIL — missing entries for `Escape`, `Constant`, `Constructor`, `Macro`, `Lifetime`; `Variable` and `Operator` still have `Reset` fg and no modifiers.

- [ ] **Step 3: Replace `default_theme()` with the full 22-token palette**

Replace the `default_theme()` function in `src/syntax/theme.rs`:

```rust
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
        (
            Constant,
            AnsiStyleSpec::plain().with_fg(Color::LightYellow),
        ),
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
```

Changes from the original:
- `Variable`: `Reset` -> `White`
- `Operator`: `Reset` -> `LightMagenta`
- `Punctuation`: `DarkGray` -> `Gray`
- `Parameter`: added `.italic()`
- `Attribute`: added `.italic()`
- New: `Escape` (LightGreen), `Constant` (LightYellow), `Constructor` (LightCyan bold), `Macro` (LightRed), `Lifetime` (Red italic)

- [ ] **Step 4: Run all syntax tests to verify they pass**

Run: `cargo test -p croot --lib syntax`
Expected: ALL PASS

- [ ] **Step 5: Commit**

```bash
git add src/syntax/theme.rs
git commit -m "feat(syntax): richer default color palette with all 16 ANSI colors"
```

---

### Task 4: Full integration test & final commit

**Files:**
- None created — just run existing + new tests end-to-end

- [ ] **Step 1: Run the complete test suite**

Run: `cargo test`
Expected: ALL PASS — no regressions in engine, languages, config, or any other module.

- [ ] **Step 2: Verify Rust highlighting produces new token colors**

Add this test to `src/syntax/engine.rs` in the existing `mod tests` block:

```rust
#[test]
fn rust_escape_sequence_gets_escape_style() {
    let lines = highlight_code("rs", r#"let s = "hello\n";"#, 100);
    // Should have more than one distinct style (at minimum keyword, variable, string)
    let styles: std::collections::HashSet<_> =
        lines.iter().flatten().map(|(_, style)| *style).collect();
    assert!(
        styles.len() >= 3,
        "Rust code should produce at least 3 distinct styles, got {}",
        styles.len()
    );
}
```

- [ ] **Step 3: Run all tests one final time**

Run: `cargo test`
Expected: ALL PASS

- [ ] **Step 4: Commit**

```bash
git add src/syntax/engine.rs
git commit -m "test(syntax): add integration test for richer highlighting styles"
```
