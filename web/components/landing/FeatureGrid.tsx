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
    <section className="py-16 sm:py-24">
      <div className="mx-auto max-w-7xl px-4 sm:px-6">
        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
          {features.map((feature) => (
            <div
              key={feature.title}
              className="group p-6 rounded-lg border border-border hover:border-border-strong bg-bg-surface transition-colors"
            >
              <feature.icon
                size={20}
                className="text-text-muted group-hover:text-text transition-colors"
              />
              <h3 className="mt-3 text-base font-semibold text-text">
                {feature.title}
              </h3>
              <p className="mt-2 text-sm text-text-secondary leading-relaxed">
                {feature.description}
              </p>
            </div>
          ))}
        </div>
      </div>
    </section>
  );
}
