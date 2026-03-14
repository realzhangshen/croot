# Pair with cmux

croot works great alongside [cmux](https://github.com/manaflow-ai/cmux) for a full terminal-based development workspace.

## What is cmux?

cmux is a terminal multiplexer that provides a VS Code-like pane layout in your terminal. Pair it with croot and you get:

- File tree on the left (croot)
- Editor in the top right
- Shell in the bottom right

```
┌──────────────┬────────────────────────┐
│ croot        │ $EDITOR                │
│              │                        │
│  src/        │  fn main() {           │
│    main.rs   │      println!("hello");│
│    lib.rs    │  }                     │
│  Cargo.toml  │                        │
│  README.md   ├────────────────────────┤
│              │ $ cargo run              │
│              │                        │
└──────────────┴────────────────────────┘
```

## Setup

1. Install cmux: see [cmux README](https://github.com/manaflow-ai/cmux)
2. Install croot: `brew install realzhangshen/croot/croot`
3. Launch cmux with croot in the sidebar pane

Together they give you a VS Code-like workspace that lives entirely in your terminal. No Electron, no GUI, just fast tools that compose well.
