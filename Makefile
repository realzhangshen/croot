.PHONY: ci fmt check clippy clippy-pedantic clippy-fix test test-all-features fix install-hooks release

PEDANTIC_ALLOW = \
	-A clippy::module_name_repetitions \
	-A clippy::must_use_candidate \
	-A clippy::missing_errors_doc \
	-A clippy::missing_panics_doc \
	-A clippy::match_same_arms \
	-A clippy::too_many_lines \
	-A clippy::cast_possible_truncation \
	-A clippy::cast_sign_loss \
	-A clippy::cast_precision_loss \
	-A clippy::manual_let_else \
	-A clippy::similar_names \
	-A clippy::uninlined_format_args \
	-A clippy::return_self_not_must_use \
	-A clippy::if_same_then_else \
	-A clippy::useless_format \
	-A clippy::derivable_impls \
	-A clippy::map_unwrap_or \
	-A clippy::needless_raw_string_hashes

## Run all CI checks (mirrors pre-push hook)
ci: fmt check clippy test test-all-features
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

## Advisory pedantic lint sweep (non-blocking style feedback)
clippy-pedantic:
	cargo clippy --quiet -- -W clippy::pedantic $(PEDANTIC_ALLOW)

## Apply machine-fixable Clippy suggestions
clippy-fix:
	cargo clippy --fix --allow-dirty --allow-staged -- -D warnings

## Run tests (default features)
test:
	cargo test --quiet

## Run tests across feature matrix: default, none, image-preview.
## Use this to catch breakage in feature combinations before pushing.
test-all-features:
	cargo test --quiet --no-default-features
	cargo test --quiet --features image-preview

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
