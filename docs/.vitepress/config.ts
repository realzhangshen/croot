import { defineConfig } from 'vitepress'
import tailwindcss from '@tailwindcss/vite'

export default defineConfig({
  title: 'croot',
  description: 'The VS Code sidebar for your terminal',
  base: '/croot/',

  head: [
    ['link', { rel: 'icon', href: '/croot/favicon.svg', type: 'image/svg+xml' }],
    ['link', { rel: 'preconnect', href: 'https://fonts.googleapis.com' }],
    ['link', { rel: 'preconnect', href: 'https://fonts.gstatic.com', crossorigin: '' }],
    ['link', { rel: 'stylesheet', href: 'https://fonts.googleapis.com/css2?family=JetBrains+Mono:wght@400;500;700&display=swap' }],
    ['meta', { name: 'theme-color', content: '#0d1117' }],
  ],

  themeConfig: {
    logo: '/favicon.svg',
    siteTitle: 'croot',

    nav: [
      { text: 'Guide', link: '/guide/getting-started' },
      { text: 'Features', link: '/features/git-integration' },
    ],

    sidebar: [
      {
        text: 'Guide',
        items: [
          { text: 'Getting Started', link: '/guide/getting-started' },
          { text: 'Installation', link: '/guide/installation' },
          { text: 'Configuration', link: '/guide/configuration' },
          { text: 'Keybindings', link: '/guide/keybindings' },
        ],
      },
      {
        text: 'Features',
        items: [
          { text: 'Git Integration', link: '/features/git-integration' },
          { text: 'File Preview', link: '/features/file-preview' },
          { text: 'Fuzzy Search', link: '/features/fuzzy-search' },
          { text: 'Mouse Support', link: '/features/mouse-support' },
          { text: 'File Operations', link: '/features/file-operations' },
          { text: 'Context Menus', link: '/features/context-menus' },
        ],
      },
      {
        text: 'Advanced',
        items: [
          { text: 'Pair with cmux', link: '/advanced/cmux-workflow' },
          { text: 'Development', link: '/advanced/development' },
        ],
      },
    ],

    socialLinks: [
      { icon: 'github', link: 'https://github.com/realzhangshen/croot' },
    ],

    search: {
      provider: 'local',
    },

    editLink: {
      pattern: 'https://github.com/realzhangshen/croot/edit/main/docs/:path',
    },

    footer: {
      message: 'Built with Rust & <a href="https://ratatui.rs">Ratatui</a>',
      copyright: 'MIT License',
    },
  },

  vite: {
    plugins: [
      tailwindcss(),
    ],
  },
})
