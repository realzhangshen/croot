export interface SidebarItem {
  text: string
  link: string
}

export interface SidebarGroup {
  text: string
  items: SidebarItem[]
}

export const sidebar: SidebarGroup[] = [
  {
    text: 'Get Started',
    items: [
      { text: 'Quickstart', link: '/docs/guide/getting-started' },
      { text: 'Installation', link: '/docs/guide/installation' },
      { text: 'Configuration', link: '/docs/guide/configuration' },
      { text: 'Keybindings', link: '/docs/guide/keybindings' },
    ],
  },
  {
    text: 'Features',
    items: [
      { text: 'Git Integration', link: '/docs/features/git-integration' },
      { text: 'File Preview', link: '/docs/features/file-preview' },
      { text: 'Fuzzy Search', link: '/docs/features/fuzzy-search' },
      { text: 'Mouse Support', link: '/docs/features/mouse-support' },
      { text: 'File Operations', link: '/docs/features/file-operations' },
      { text: 'Context Menus', link: '/docs/features/context-menus' },
    ],
  },
  {
    text: 'Workflow',
    items: [
      { text: 'Pair with cmux', link: '/docs/advanced/cmux-workflow' },
      { text: 'Development', link: '/docs/advanced/development' },
    ],
  },
]
