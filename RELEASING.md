# Releasing croot

## Overview

croot uses a **push-based release pipeline**: push a `v*.*.*` tag and CI
handles everything else.

```
git tag v0.5.0 && git push --tags
        │
        ▼
  ┌─────────────┐
  │ validate-tag │  rejects non-semver tags
  └──────┬──────┘
         ▼
  ┌─────────────┐  macOS arm64 / x86_64
  │    build     │  Linux x86_64 / aarch64
  └──────┬──────┘
         ▼
  ┌─────────────┐
  │   release    │  creates GitHub Release with 4 tarballs
  └──────┬──────┘
         ▼
  ┌────────────────┐
  │ update-homebrew │  pushes formula to realzhangshen/homebrew-croot
  └────────────────┘
```

Fully automated after the tag push — manual steps are only pre-tag
preparation.

---

## Pre-Release Checklist

### Step 1: Ensure all changes are merged and CI is green

```bash
make ci        # local CI: fmt, check, clippy, test
git push       # push to main, wait for CI to pass
```

### Step 2: Update version in Cargo.toml

Change `version = "X.Y.Z"` in `Cargo.toml`, then regenerate the lockfile:

```bash
cargo check    # updates Cargo.lock
```

### Step 3: Update CHANGELOG.md

1. Move items from `[Unreleased]` to a new `[X.Y.Z] - YYYY-MM-DD` section.
2. Add a comparison link at the bottom:
   ```
   [X.Y.Z]: https://github.com/realzhangshen/croot/compare/vPREV...vX.Y.Z
   ```
3. Update the `[Unreleased]` link:
   ```
   [Unreleased]: https://github.com/realzhangshen/croot/compare/vX.Y.Z...HEAD
   ```

### Step 4: Commit the release

```bash
git add Cargo.toml Cargo.lock CHANGELOG.md
git commit -m "Release vX.Y.Z"
```

### Step 5: Create and push the tag

```bash
git tag vX.Y.Z
git push && git push --tags
```

---

## What Happens Automatically

After you push the tag, the `.github/workflows/release.yml` pipeline runs
four jobs in sequence:

### 1. validate-tag

Rejects any tag that doesn't match `^v[0-9]+\.[0-9]+\.[0-9]+$`. Tags like
`0.4.0` (missing `v` prefix) or `v0.4.0-beta` (pre-release suffix) will
fail here.

### 2. build

Cross-compiles `croot` for four targets using `cargo build --release`:

| Target                        | Runner         |
| ----------------------------- | -------------- |
| `aarch64-apple-darwin`        | `macos-14`     |
| `x86_64-apple-darwin`         | `macos-14`     |
| `x86_64-unknown-linux-gnu`    | `ubuntu-latest`|
| `aarch64-unknown-linux-gnu`   | `ubuntu-latest`|

Each binary is packaged as `croot-vX.Y.Z-<target>.tar.gz` and uploaded as a
build artifact.

### 3. release

Downloads all four artifacts and creates a GitHub Release using
`softprops/action-gh-release@v2` with auto-generated release notes. All
`.tar.gz` files are attached as release assets.

### 4. update-homebrew

Downloads the two macOS tarballs from the newly created release, computes
their SHA-256 checksums, and pushes an updated `Formula/croot.rb` to
`realzhangshen/homebrew-croot` via the GitHub API. Uses the
`HOMEBREW_TAP_TOKEN` repository secret for authentication.

---

## Post-Release Verification

1. **Check GitHub Actions** — all four jobs should be green.
2. **Verify the GitHub Release** — the release page should list all four
   `croot-vX.Y.Z-*.tar.gz` binaries.
3. **Test Homebrew install:**
   ```bash
   brew update && brew upgrade croot
   ```
4. **Verify version:**
   ```bash
   croot --version
   ```

---

## Pitfalls & Lessons Learned

### 1. Forgot to create git tag (v0.2.2)

The release commit existed on `main` but no tag was pushed, so CI never
triggered. The version was bumped in `Cargo.toml` and `CHANGELOG.md` had a
`[0.2.2]` section, but without the tag the release was never built.

**Fix:** Always run `git push --tags` after `git tag`.

### 2. Version bump without CHANGELOG update (v0.4.1)

`Cargo.toml` was bumped to `0.4.1` but `CHANGELOG.md` was left with an
empty `[Unreleased]` section and no `[0.4.1]` entry. The release went out
with no user-facing notes.

**Fix:** Always update `CHANGELOG.md` in the same commit as the version
bump.

### 3. CHANGELOG link maintenance

The comparison links at the bottom of `CHANGELOG.md` must be updated for
every release. Stale links produce 404 pages on GitHub. Both the new version
link and the `[Unreleased]` link need updating.

### 4. Tag format must be exact

The CI trigger pattern is `v*.*.*` and the validation regex requires
`^v[0-9]+\.[0-9]+\.[0-9]+$`. Only tags like `v0.4.0` will trigger the
pipeline. These will **not** work:
- `0.4.0` (missing `v` prefix)
- `v0.4.0-beta` (pre-release suffix)
- `V0.4.0` (uppercase)

### 5. HOMEBREW_TAP_TOKEN expiry

The `HOMEBREW_TAP_TOKEN` secret must be a valid GitHub PAT with
`contents:write` permission on `realzhangshen/homebrew-croot`. If it
expires, the `update-homebrew` job will fail silently (the release itself
still succeeds). Check token expiry periodically.

---

## Quick Reference

Copy-paste block — replace `X.Y.Z` with your version and `PREV` with the
previous version:

```bash
# 1. Local CI
make ci

# 2. Bump version
sed -i '' 's/^version = ".*"/version = "X.Y.Z"/' Cargo.toml
cargo check

# 3. Update CHANGELOG.md
#    - Move [Unreleased] items to [X.Y.Z] - YYYY-MM-DD
#    - Add: [X.Y.Z]: https://github.com/realzhangshen/croot/compare/vPREV...vX.Y.Z
#    - Update: [Unreleased]: https://github.com/realzhangshen/croot/compare/vX.Y.Z...HEAD

# 4. Commit & tag
git add Cargo.toml Cargo.lock CHANGELOG.md
git commit -m "Release vX.Y.Z"
git tag vX.Y.Z
git push && git push --tags

# 5. Verify
#    - GitHub Actions: all green
#    - Release page: 4 binaries attached
#    - brew update && brew upgrade croot
#    - croot --version
```
