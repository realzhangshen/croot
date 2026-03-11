.PHONY: ci fmt check clippy test fix install-hooks

## Run all CI checks (mirrors pre-push hook)
ci: fmt check clippy test
	@echo ""
	@echo "All checks passed."

## Check formatting
fmt:
	cargo fmt --check

## Compile check
check:
	cargo check --quiet

## Lint with clippy
clippy:
	cargo clippy --quiet -- -D warnings

## Run tests
test:
	cargo test --quiet

## Auto-format code
fix:
	cargo fmt

## Set up git hooks
install-hooks:
	git config core.hooksPath .githooks
	@echo "Git hooks installed (.githooks/)"
