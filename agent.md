# croot — Agent Instructions

## Project
Lightweight terminal file tree sidebar, built with Rust + Ratatui.

## Commands
- `make ci` — run all checks (fmt, check, clippy, test)
- `cargo test --quiet` — run tests
- `cargo clippy --quiet -- -D warnings` — lint
- `cargo fmt` — format code

## Conventions
- Rust edition 2021, MSRV 1.88
- `unsafe` code is forbidden
- Clippy pedantic enabled
- Error handling: `anyhow`
- Tests: inline `#[cfg(test)] mod tests` at bottom of each source file

## TDD (Required)
All development follows TDD:
1. Write a failing test first
2. Write minimum code to pass
3. Refactor while tests stay green

Every feature and bug fix starts with a test. Run `cargo test` before completing any change.
