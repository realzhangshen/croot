# Review Execution Plan

This document turns the preview/syntax/render review into an execution plan.
It includes the corrections found during code verification so the work can be
implemented in small, testable commits.

## Ground Rules

- Follow TDD for every behavior change: write the failing test first, make it
  pass, then refactor.
- Keep each phase independently shippable.
- Run `cargo test` before every commit.
- Use Criterion only for performance evidence; do not make bench numbers part of
  the required test gate.
- Do not move tests out of source files unless the local AGENTS.md rule changes.
  The current rule says tests belong in `#[cfg(test)] mod tests` at the bottom of
  the source file.

## Verified Corrections

### Resize And Split Drag

The original review said terminal resize and split-ratio dragging both cause a
full re-highlight. That is not accurate.

- Terminal resize currently calls `trigger_preview_load`.
- `trigger_preview_load` short-circuits on path, mtime, and git diff hint, so
  ordinary text previews usually do not re-highlight on resize.
- Split-ratio dragging updates `preview.split_ratio`, but it does not call
  `trigger_preview_load`.

The real issue is width-aware rendered content. Rendered Markdown depends on
preview width, but width is not part of the cache key. Text previews should stay
cached on width-only changes. Rendered Markdown should reflow when the computed
preview content width changes.

Implementation should not capture the old width from the resize event. The
current `content_width` is recomputed during draw, so the safer design is:

- Track the previous preview content width.
- Let draw update `preview.content_width` and mark a width-changed flag.
- After draw, trigger a width-only refresh only when the current preview kind is
  `Rendered` or the selected file is Markdown with render mode enabled.
- Do not reload plain text previews for width-only changes.

This also fixes split dragging for Markdown because the drag changes layout,
draw observes the new width, and the same width-change path can reflow rendered
Markdown.

### Preview Highlight Wrapper

`src/preview/highlight.rs` is a thin wrapper over `src/syntax/engine.rs`, but it
is not purely private cleanup.

- `loader` and `render_md` call it internally.
- `benches/core_ops.rs` imports `croot::preview::highlight::highlight_code`.
- `src/preview/mod.rs` exports the module publicly.

Deleting it may be a public API change for the crate. Treat removal as an
optional cleanup after deciding whether `preview::highlight` is supported API.
If it is removed, update benches and consider a deprecation period first.

### StyledSpan Ownership

The review is right that `StyledSpan = (String, Style)` forces one allocation per
styled token. Replacing it with `Cow<'static, str>` alone is not the main fix:
`Cow<'static, str>` cannot borrow from the loaded file content unless the content
itself is owned in a compatible long-lived structure.

Use two steps:

1. Convert the tuple alias to a real `StyledSpan` struct while preserving
   `String` ownership. This reduces future churn and makes call sites explicit.
2. In a later, dedicated performance phase, move preview text ownership into a
   structure that can store the source text once and reference spans by ranges,
   such as `Arc<str>` plus `(line, start, end, style)` spans or a flattened span
   list with line offsets.

Do not combine the struct conversion and range-backed storage in one commit.

### SyntaxSet Newline Branch

The review mentioned a possible `SyntaxSet::load_defaults_nonewlines` branch.
The current code uses `two_face::syntax::extra_newlines`, so that branch does not
exist. Do not plan work around removing manually appended newlines unless the
syntax-set strategy changes.

### App Test Relocation

`src/app/mod.rs` is large mostly because its test module starts around the middle
of the file. Moving tests to `src/app/tests.rs` or integration tests would make
the file easier to scan, but the current AGENTS.md instruction says tests go in
`#[cfg(test)] mod tests` at the bottom of the source file.

Defer this cleanup unless the project rule is changed.

## Phase 0: Add Measurement Coverage

Goal: create a baseline before changing performance-sensitive paths.

Tests:

- No required functional test unless a new helper is introduced.

Benches:

- Add `highlight_typescript_400_lines` to cover a syntax path that can be slower
  than Rust.
- Add a preview render benchmark that exercises the selection path. If rendering
  through a full Ratatui widget is too awkward, benchmark the helper introduced
  in Phase 6 instead.

Implementation notes:

- Keep generated code samples small and deterministic.
- Use `black_box` for inputs and outputs.
- Record before/after numbers in the PR or commit notes, not in source comments.

Validation:

- `cargo test`
- `cargo bench --bench core_ops highlight`

Commit:

- `bench preview and typescript highlight baselines`

## Phase 1: O(1) Theme Lookup

Goal: remove per-token `HashMap` lookup and repeated `AnsiStyleSpec -> Style`
conversion from the highlight hot path.

Tests:

- Add tests in `src/syntax/theme.rs` proving that every `SemanticToken::ALL`
  token has a style.
- Add a config override test proving token overrides still land at the same
  token index.
- Add a parse/round-trip guard if `SemanticToken` gains an index method.

Implementation:

- Add `SemanticToken::COUNT` and `SemanticToken::index()` or use
  `#[repr(usize)]` with an explicit method.
- Change `SyntaxTheme` from `HashMap<SemanticToken, AnsiStyleSpec>` to an array
  or fixed-size vector indexed by `SemanticToken`.
- Consider caching `Style` directly in `SyntaxTheme` so `style_for` is a single
  indexed copy.
- Keep `from_config` parsing behavior and warning behavior unchanged.

Scope cache:

- Do not add a new hash dependency in the same commit unless the benchmark shows
  the scope cache itself is a measurable bottleneck.
- If needed later, prefer `rustc-hash` or an equivalent small fast hash map and
  isolate the dependency addition in its own commit.

Validation:

- `cargo test`
- `cargo bench --bench core_ops highlight_rust_400_lines`
- `cargo bench --bench core_ops highlight_typescript_400_lines`

Commit:

- `optimize syntax theme lookup`

## Phase 2: Width-Aware Markdown Reflow

Goal: make rendered Markdown reflow on preview-width changes without reloading
or re-highlighting ordinary text previews.

Tests:

- Add an app-level test for width change on a text preview: generation should
  not increment and cached content should remain valid.
- Add an app-level or preview-controller test for width change on rendered
  Markdown: a reload or reflow should be scheduled after draw observes the new
  width.
- Add a split-drag Markdown case if the helper design makes it cheap.

Implementation:

- Track previous preview content width in `PreviewController`.
- During draw, compare the new width with the previous width and store a
  width-changed flag.
- After draw in the event loop, process the flag with access to `preview_tx`.
- Only reload/reflow when the preview is visible and the current/selected preview
  is rendered Markdown.
- Avoid calling `trigger_preview_load` directly from `Event::Resize` with stale
  width data.
- Preserve current cache behavior for text previews.

Possible helper:

```rust
fn should_reflow_for_width_change(&self) -> bool
```

This helper should be small enough to test directly.

Validation:

- `cargo test`
- Manual smoke test: open a long Markdown paragraph, resize the preview pane,
  and confirm wrapping changes. Open a Rust file and confirm width-only resize
  does not schedule a new highlight.

Commit:

- `reflow rendered markdown on preview width changes`

## Phase 3: StyledSpan Struct Conversion

Goal: replace the tuple alias with a named type without changing allocation
strategy yet.

Tests:

- Update existing tests to use a helper such as `StyledSpan::new`.
- Add tests for any accessor methods introduced on `StyledSpan`.
- Keep selection extraction tests green, especially CJK and multi-span cases.

Implementation:

- Replace `pub type StyledSpan = (String, Style)` with:

```rust
pub struct StyledSpan {
    pub text: String,
    pub style: Style,
}
```

- Add `StyledSpan::new(text, style)` and `StyledSpan::plain(text)` if that
  reduces repetitive construction.
- Update highlighter, loader, Markdown renderer, preview state, preview view,
  and tests.
- Avoid changing storage layout to `Arc<str>` or ranges in this phase.

Validation:

- `cargo test`

Commit:

- `make styled spans explicit`

## Phase 4: Range-Backed Preview Spans

Goal: eliminate per-token string allocation in syntax-highlighted previews.

Tests:

- Add tests proving selected text extraction works across span boundaries with
  range-backed spans.
- Add tests for long-line fallback and parse-error fallback.
- Add tests for Markdown fenced code if the new storage type is shared there.

Implementation options:

- Introduce a `PreviewContent` owner that stores `Arc<str>` source text plus
  spans that reference source ranges.
- Or flatten content into `Vec<SpanRef>` plus `line_offsets: Vec<usize>`.
- Keep non-file generated previews, such as hex dumps and directory previews,
  able to own strings without awkward source reconstruction.

Recommendation:

- Do not force every preview kind into the range-backed path immediately.
- Start with syntax-highlighted text files, then decide whether Markdown,
  directory previews, and binary previews should use the same representation.

Validation:

- `cargo test`
- `cargo bench --bench core_ops highlight_rust_400_lines`
- `cargo bench --bench core_ops highlight_typescript_400_lines`
- Inspect allocation counts with a profiler if available.

Commit:

- `store highlighted text spans by source range`

## Phase 5: Preview Load Options

Goal: make preview loading extensible without growing the parameter list.

Tests:

- Add `PreviewLoadOptions::default` or constructor tests.
- Update loader tests to pass options and preserve existing behavior.

Implementation:

- Add:

```rust
pub struct PreviewLoadOptions<'a> {
    pub max_file_size_kb: u64,
    pub syntax_highlight: bool,
    pub render_markdown: bool,
    pub preview_width: usize,
    pub image_preview: bool,
    pub repo_root: Option<&'a Path>,
    pub git_diff_hint: GitDiffHint,
}
```

- Change `load_preview(path, options)` and update call sites.
- Remove the clippy `fn_params_excessive_bools` allow after the signature is
  simplified.

Validation:

- `cargo test`

Commit:

- `group preview load options`

## Phase 6: Preview Render Selection Fast Path

Goal: remove per-character Ratatui writes from selected lines.

Tests:

- Add rendering tests in `src/render/preview_view.rs` using a small `Buffer`.
- Cover no selection, partial single-span selection, selection crossing span
  boundaries, CJK characters, and zero-width combining characters if supported.

Implementation:

- Extract a helper that renders one styled run within a width budget.
- For selection, split a span into at most three display segments: before,
  selected, after.
- Use bulk `set_string` for each segment where possible.
- Keep the current zero-width combining behavior correct.
- Reuse a `String` for line-number gutter formatting only if profiling or a
  focused bench shows it matters.

Validation:

- `cargo test`
- Run the selection render benchmark from Phase 0 or add it here.

Commit:

- `speed up preview selection rendering`

## Phase 7: TreeView Render Refactor

Goal: make `TreeView::render` easier to scan without behavior changes.

Tests:

- Keep existing tree render tests green.
- Add no new tests unless the extraction exposes a meaningful pure helper.

Implementation:

- Extract row mode into a small enum outside the render function.
- Extract row span construction into a helper.
- Extract row background fill into a helper.
- Do not change compaction, filter-mode guide calculation, hover, cursor, or
  find-highlight behavior in the same commit.

Validation:

- `cargo test`

Commit:

- `split tree row rendering helpers`

## Phase 8: Optional Public API Cleanup

Goal: decide whether `preview::highlight` should remain public.

Decision points:

- If the crate treats `preview::highlight` as public API, keep it as a facade.
- If the crate does not guarantee this module, remove it in a breaking/internal
  cleanup and update benches to import `croot::syntax::engine`.

Tests:

- Move or preserve the wrapper tests. Do not lose language coverage.

Validation:

- `cargo test`
- `cargo bench --bench core_ops highlight_rust_400_lines`

Commit:

- `remove preview highlight facade` or skip this phase.

## Deferred Work

These ideas are valid but should not be first-wave work:

- Semantic token macro generation. It reduces maintenance cost but is not needed
  for the hot-path theme lookup.
- Preview kind plugin/registry abstraction. The current closed enum is simpler
  unless new preview kinds are actively planned.
- App test relocation. Useful for readability, but blocked by current local
  test-placement rules.
- Incremental or viewport-only syntax highlighting. Potentially high impact, but
  it changes loading semantics and should follow the simpler storage and lookup
  optimizations.
- LRU preview cache. Useful for rapid cursor backtracking, but it adds memory
  policy decisions and should be measured first.

## Recommended Execution Order

1. Phase 0: add benchmark baselines.
2. Phase 1: make theme lookup O(1).
3. Phase 2: fix width-aware Markdown reflow while preserving text cache behavior.
4. Phase 3: convert `StyledSpan` to a struct.
5. Phase 6: speed up selected-line rendering.
6. Phase 5: group preview load options.
7. Phase 7: refactor `TreeView::render`.
8. Phase 4: range-backed highlighted spans, as a dedicated performance project.
9. Phase 8: decide whether to keep or remove the highlight facade.

The main dependency is that Phase 4 should follow Phase 3. Phase 2 can be done
before or after the syntax hot-path changes because it fixes cache semantics
rather than token rendering.
