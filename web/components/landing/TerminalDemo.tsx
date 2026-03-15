export function TerminalDemo() {
  return (
    <section className="py-16 sm:py-24">
      <div className="mx-auto max-w-4xl px-4 sm:px-6">
        <div className="rounded-lg border border-border overflow-hidden shadow-lg">
          {/* Title bar */}
          <div className="flex items-center gap-2 px-4 py-3 bg-bg-elevated border-b border-border">
            <span className="w-3 h-3 rounded-full bg-[#ff5f57]" />
            <span className="w-3 h-3 rounded-full bg-[#febc2e]" />
            <span className="w-3 h-3 rounded-full bg-[#28c840]" />
            <span className="ml-2 text-xs text-text-muted font-mono">
              croot ~/projects/my-app
            </span>
          </div>
          {/* Terminal content */}
          <pre className="bg-[#1a1a2e] text-[#e0e0e0] p-6 text-xs sm:text-sm leading-relaxed font-mono overflow-x-auto">
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
