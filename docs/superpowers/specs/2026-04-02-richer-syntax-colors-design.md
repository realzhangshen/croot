# Richer Syntax Highlighting Colors

## Problem

The default syntax theme uses only 10 of 16 available ANSI colors, and 3 common
token types (`Variable`, `Operator`, `Text`) render with `Color::Reset` — making
them indistinguishable from plain text. Additionally, several semantically
distinct code elements (escape sequences, constants, constructors, macros,
lifetimes) are collapsed into other tokens, reducing visual differentiation.

## Solution

Add 5 new semantic tokens and assign distinct ANSI colors + modifiers to all 22
tokens, utilizing all 16 ANSI colors.

## New Semantic Tokens

| Token | Purpose | Example |
|-------|---------|---------|
| `Escape` | String escape sequences | `\n`, `\t`, `\"` |
| `Constant` | Named constants & builtin constants | `MAX_SIZE`, `true`, `None` |
| `Constructor` | Type constructors | `Some(x)`, `Ok(v)` |
| `Macro` | Macro invocations | `println!()`, `vec![]` |
| `Lifetime` | Lifetime annotations (Rust) | `'a`, `'static` |

## Capture Map Changes

### Remapped captures

| Capture | Old Token | New Token |
|---------|-----------|-----------|
| `escape` | String | Escape |
| `constant` | Number | Constant |
| `constant.builtin` | Keyword | Constant |
| `constructor` | Function | Constructor |
| `constructor.builtin` | Function | Constructor |

### New captures

| Capture | Token |
|---------|-------|
| `function.macro` | Macro |
| `keyword.directive` | Macro |
| `lifetime` | Lifetime |

Total capture count: 56 -> 59.

## Default Color Theme (22 tokens)

| Token | Color | Modifier | Change |
|-------|-------|----------|--------|
| Text | Reset | | unchanged |
| Keyword | Magenta | Bold | unchanged |
| Type | Cyan | | unchanged |
| TypeBuiltin | LightCyan | | unchanged |
| String | Green | | unchanged |
| Escape | LightGreen | | new |
| Number | Yellow | | unchanged |
| Constant | LightYellow | | new |
| Comment | DarkGray | Italic | unchanged |
| Function | Blue | | unchanged |
| Method | LightBlue | | unchanged |
| Constructor | LightCyan | Bold | new |
| Variable | White | | was Reset |
| Parameter | White | Italic | add italic |
| Property | LightBlue | | unchanged |
| Operator | LightMagenta | | was Reset |
| Punctuation | Gray | | was DarkGray |
| Module | Blue | Bold | unchanged |
| Tag | Magenta | | unchanged |
| Attribute | Yellow | Italic | add italic |
| Macro | LightRed | | new |
| Lifetime | Red | Italic | new |

## Files Changed

All changes within `src/syntax/`:

1. **`semantic.rs`** — Add 5 variants, update `ALL` (17->22), `as_str()`, `FromStr`
2. **`capture_map.rs`** — Add 3 new captures, remap 5 existing, update array sizes
3. **`theme.rs`** — Add 5 new default styles, update Variable/Operator/Punctuation/Attribute

No changes to `engine.rs`, `languages.rs`, or `config.rs` — they are generic
over the token set.

## Backward Compatibility

- Existing user `[syntax.tokens.*]` config continues to work
- Unknown token names in user config already emit warnings (no change needed)
- New tokens use defaults until user configures them

## Testing

- `semantic.rs`: `token_names_round_trip` auto-covers new variants via `ALL`
- `capture_map.rs`: Update count assertion 56->59, verify new capture mappings
- `theme.rs`: Add test that all 22 tokens (except Text) have non-plain defaults
- `engine.rs`: Existing integration tests pass unchanged
