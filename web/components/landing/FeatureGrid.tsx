import {
  GitBranch,
  Eye,
  Search,
  FolderOpen,
  Mouse,
  TerminalSquare,
} from "lucide-react";

const features = [
  {
    icon: GitBranch,
    title: "Git Integration",
    description:
      "See file status at a glance — staged, modified, untracked. Color-coded indicators propagate through the tree.",
  },
  {
    icon: Eye,
    title: "File Preview",
    description:
      "Split-pane preview with syntax highlighting for 150+ languages, Markdown rendering, and hex dumps.",
  },
  {
    icon: Search,
    title: "Fuzzy Search",
    description:
      "Four search modes: local filter, filename search with fd, content search with rg, and in-tree find.",
  },
  {
    icon: FolderOpen,
    title: "File Operations",
    description:
      "Create, rename, delete files and directories. Open in your editor or external app with one key.",
  },
  {
    icon: Mouse,
    title: "Mouse Support",
    description:
      "Click, scroll, drag the divider, double-click to open. Full mouse support that just works.",
  },
  {
    icon: TerminalSquare,
    title: "cmux Pairing",
    description:
      "Pair with cmux for a VS Code-like layout — file tree on the left, editor and shell on the right.",
  },
];

export function FeatureGrid() {
  return (
    <section className="py-16 sm:py-20">
      <div className="mx-auto max-w-7xl px-4 sm:px-6">
        <div className="mb-8 text-center">
          <p className="text-[12px] font-medium tracking-[0.02em] text-text-muted">
            Core workflows
          </p>
          <h2 className="mt-2 text-3xl font-semibold tracking-[-0.05em] text-text sm:text-4xl">
            Built for daily terminal work
          </h2>
        </div>
        <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3">
          {features.map((feature) => (
            <div
              key={feature.title}
              className="group rounded-[24px] border border-border bg-bg-surface p-6 transition-all hover:-translate-y-0.5 hover:border-border-strong hover:bg-white/70 dark:hover:bg-white/[0.02]"
            >
              <div className="inline-flex rounded-2xl border border-border bg-bg-elevated p-2.5">
                <feature.icon
                  size={18}
                  className="text-text-muted transition-colors group-hover:text-text"
                />
              </div>
              <h3 className="mt-4 text-[17px] font-semibold tracking-[-0.03em] text-text">
                {feature.title}
              </h3>
              <p className="mt-2 text-sm leading-7 text-text-secondary">
                {feature.description}
              </p>
            </div>
          ))}
        </div>
      </div>
    </section>
  );
}
