# Development Rules

## TDD (Test-Driven Development)

All development must follow the TDD workflow:

1. **Red** — Write a failing test first
2. **Green** — Write the minimum code to make it pass
3. **Refactor** — Clean up while keeping tests green

Rules:
- Every new feature and bug fix starts with a test
- Tests go in `#[cfg(test)] mod tests` at the bottom of the source file
- Run `cargo test` before considering any change complete
- Do not write implementation code without a corresponding test

## Auto Commit

When a milestone task is completed (e.g., finishing a plan's execution, completing a feature, fixing a bug), automatically commit the changes:

1. Run `cargo test` — only proceed if **all tests pass**
2. Stage relevant files and create a commit with a clear message
3. Do not wait for the user to ask — commit proactively upon task completion
