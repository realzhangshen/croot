.PHONY: ci fmt check clippy test fix install-hooks release

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

## Create and push a release tag (usage: make release VERSION=0.5.0)
release:
	@test -n "$(VERSION)" || (echo "Usage: make release VERSION=0.5.0" && exit 1)
	@grep -q 'version = "$(VERSION)"' Cargo.toml || (echo "Error: Cargo.toml version != $(VERSION)" && exit 1)
	@grep -q '\[$(VERSION)\]' CHANGELOG.md || (echo "Error: CHANGELOG.md missing [$(VERSION)] entry" && exit 1)
	@git diff --quiet || (echo "Error: working tree is dirty" && exit 1)
	git tag v$(VERSION)
	git push && git push --tags
	@echo "Released v$(VERSION)"
