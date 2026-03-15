export function TerminalDemo() {
  return (
    <section className="py-14 sm:py-20">
      <div className="mx-auto max-w-5xl px-4 sm:px-6">
        <div className="overflow-hidden rounded-[28px] border border-border bg-bg-surface shadow-[var(--shadow-dialog)]">
          {/* Title bar */}
          <div className="flex items-center gap-2 border-b border-border bg-bg-elevated px-4 py-3">
            <span className="w-3 h-3 rounded-full bg-[#ff5f57]" />
            <span className="w-3 h-3 rounded-full bg-[#febc2e]" />
            <span className="w-3 h-3 rounded-full bg-[#28c840]" />
            <span className="ml-2 text-xs text-text-muted font-mono">
              croot ~/projects/my-app
            </span>
          </div>
          {/* Terminal content */}
          <pre className="overflow-x-auto bg-[#181714] p-6 text-xs leading-relaxed text-[#efe7d9] sm:p-8 sm:text-sm">
            {`  my-app/                       │  # Getting Started
  ├── src/                      │
  │   ├── main.rs          M    │  Welcome to **my-app**!
  │   ├── config.rs         ●   │
  │   ├── lib.rs                │  ## Installation
  │   └── utils/                │
  │       ├── fs.rs        M    │  \`\`\`bash
  │       └── path.rs           │  cargo install my-app
  ├── tests/                    │  \`\`\`
  │   └── integration.rs   A    │
  ├── Cargo.toml           M    │  ## Usage
  ├── README.md            ●    │
  └── .gitignore                │  Run \`my-app\` in any directory.`}
          </pre>
        </div>
      </div>
    </section>
  );
}
