export interface SidebarLink {
  title: string;
  slug: string;
}

export interface SidebarSection {
  label: string;
  links: SidebarLink[];
}

export const sidebarSections: SidebarSection[] = [
  {
    label: "Guide",
    links: [
      { title: "Quickstart", slug: "guide/getting-started" },
      { title: "Installation", slug: "guide/installation" },
      { title: "Configuration", slug: "guide/configuration" },
      { title: "Keybindings", slug: "guide/keybindings" },
    ],
  },
  {
    label: "Features",
    links: [
      { title: "File Preview", slug: "features/file-preview" },
      { title: "Fuzzy Search", slug: "features/fuzzy-search" },
      { title: "File Operations", slug: "features/file-operations" },
      { title: "Git Integration", slug: "features/git-integration" },
      { title: "Mouse Support", slug: "features/mouse-support" },
      { title: "Context Menus", slug: "features/context-menus" },
    ],
  },
  {
    label: "Advanced",
    links: [
      { title: "cmux Workflow", slug: "advanced/cmux-workflow" },
      { title: "Development", slug: "advanced/development" },
      { title: "Review Execution Plan", slug: "advanced/review-execution-plan" },
    ],
  },
];

export function getAllDocSlugs(): string[][] {
  return sidebarSections.flatMap((section) =>
    section.links.map((link) => link.slug.split("/"))
  );
}
