# Installation

## Homebrew (macOS)

The recommended way to install croot on macOS:

```bash
brew install realzhangshen/croot/croot
```

This installs the latest release and makes `croot` available in your PATH.

## From Source

Requires [Rust](https://rustup.rs/) 1.90 or later.

```bash
git clone https://github.com/realzhangshen/croot.git
cd croot
cargo build --release
```

The binary will be at `target/release/croot`. Move it to a directory in your PATH:

```bash
sudo cp target/release/croot /usr/local/bin/
```

## Pre-built Binaries

Download pre-built binaries from the [GitHub Releases](https://github.com/realzhangshen/croot/releases) page.

Available targets:
- `aarch64-apple-darwin` — Apple Silicon Mac
- `x86_64-apple-darwin` — Intel Mac
- `x86_64-unknown-linux-gnu` — Linux x86_64
- `aarch64-unknown-linux-gnu` — Linux ARM64

Example for macOS Apple Silicon:

```bash
TAG=v0.5.1
curl -fsSL "https://github.com/realzhangshen/croot/releases/download/${TAG}/croot-${TAG}-aarch64-apple-darwin.tar.gz" | tar xz
sudo mv croot /usr/local/bin/
```

## Optional: Nerd Font

croot renders 100+ file type icons when a [Nerd Font](https://www.nerdfonts.com/) is installed. Without a Nerd Font, croot still works perfectly — it just uses plain text markers instead.

Recommended fonts:
- [JetBrains Mono Nerd Font](https://www.nerdfonts.com/font-downloads)
- [Fira Code Nerd Font](https://www.nerdfonts.com/font-downloads)

## Verify Installation

```bash
croot --version
```
