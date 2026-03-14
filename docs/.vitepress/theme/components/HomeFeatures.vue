<template>
  <section class="features-section">
    <div class="section-inner">
      <p class="section-label">Features</p>
      <h2 class="section-title">Everything you need in a file explorer</h2>
      <p class="section-desc">A fast, keyboard-driven file tree with git awareness, syntax previews, and mouse support. No Electron required.</p>

      <div class="features-grid">
        <div v-for="feature in features" :key="feature.title" class="feature-card">
          <div class="feature-icon" v-html="feature.icon"></div>
          <h3>{{ feature.title }}</h3>
          <p v-html="feature.desc"></p>
        </div>
      </div>
    </div>
  </section>
</template>

<script setup>
const features = [
  {
    title: 'Git Status Integration',
    desc: 'See modified, staged, untracked, and conflicted files at a glance. Status propagates to parent directories so you always know where changes live.',
    icon: '<svg viewBox="0 0 24 24"><circle cx="12" cy="12" r="3"/><path d="M6.3 20.3a2.4 2.4 0 0 0 3.4 0L12 18l2.3 2.3a2.4 2.4 0 0 0 3.4 0l.3-.3a2.4 2.4 0 0 0 0-3.4L15.7 14 18 11.7a2.4 2.4 0 0 0 0-3.4l-.3-.3a2.4 2.4 0 0 0-3.4 0L12 10.3 9.7 8a2.4 2.4 0 0 0-3.4 0l-.3.3a2.4 2.4 0 0 0 0 3.4L8.3 14 6 16.3a2.4 2.4 0 0 0 0 3.4z"/></svg>',
  },
  {
    title: 'Syntax-Highlighted Preview',
    desc: 'Split-pane preview with syntax highlighting for 150+ languages, Markdown rendering, binary hex dumps, and a resizable divider.',
    icon: '<svg viewBox="0 0 24 24"><polyline points="16 18 22 12 16 6"/><polyline points="8 6 2 12 8 18"/></svg>',
  },
  {
    title: 'Fuzzy Search',
    desc: 'Hit <code>/</code> to fuzzy-filter the tree by filename. Navigate matches with Tab and Shift+Tab. The match count updates in real time.',
    icon: '<svg viewBox="0 0 24 24"><circle cx="11" cy="11" r="8"/><line x1="21" y1="21" x2="16.65" y2="16.65"/></svg>',
  },
  {
    title: 'Full Mouse Support',
    desc: 'Click to select, double-click to expand, right-click for context menus, drag to resize panes, and scroll naturally. Or stay on the keyboard.',
    icon: '<svg viewBox="0 0 24 24"><path d="M4 4h6v6H4z"/><path d="M14 4h6v6h-6z"/><path d="M4 14h6v6H4z"/><path d="M14 14h6v6h-6z"/></svg>',
  },
  {
    title: 'File Operations',
    desc: 'Create, rename, and delete files and directories. Multi-select with <code>v</code> for bulk operations. All without leaving the terminal.',
    icon: '<svg viewBox="0 0 24 24"><path d="M14.5 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7.5L14.5 2z"/><polyline points="14 2 14 8 20 8"/><line x1="12" y1="18" x2="12" y2="12"/><line x1="9" y1="15" x2="15" y2="15"/></svg>',
  },
  {
    title: 'Real-Time File Watching',
    desc: 'The tree auto-refreshes when files change on disk. Git status updates automatically. A debounced watcher keeps things fast.',
    icon: '<svg viewBox="0 0 24 24"><path d="M12 2v4"/><path d="M12 18v4"/><path d="M4.93 4.93l2.83 2.83"/><path d="M16.24 16.24l2.83 2.83"/><path d="M2 12h4"/><path d="M18 12h4"/><path d="M4.93 19.07l2.83-2.83"/><path d="M16.24 7.76l2.83-2.83"/></svg>',
  },
  {
    title: 'Nerd Font Icons',
    desc: '100+ file type icons render when a Nerd Font is installed. Directories, Rust, JavaScript, Markdown — each gets its own glyph.',
    icon: '<svg viewBox="0 0 24 24"><path d="M4 20h16a2 2 0 0 0 2-2V8a2 2 0 0 0-2-2h-7.93a2 2 0 0 1-1.66-.9l-.82-1.2A2 2 0 0 0 7.93 3H4a2 2 0 0 0-2 2v13c0 1.1.9 2 2 2Z"/></svg>',
  },
  {
    title: 'Context Menus',
    desc: 'Right-click any node for contextual actions: open in editor, copy path, reveal in Finder, rename, delete, and more.',
    icon: '<svg viewBox="0 0 24 24"><rect x="3" y="3" width="18" height="18" rx="2"/><path d="M3 9h18"/><path d="M9 21V9"/></svg>',
  },
  {
    title: 'Configurable',
    desc: 'TOML config for hidden files, exclusion patterns, compact folders, preview settings, and custom editor commands. Manage it with <code>croot&nbsp;config</code>.',
    icon: '<svg viewBox="0 0 24 24"><path d="M12.22 2h-.44a2 2 0 0 0-2 2v.18a2 2 0 0 1-1 1.73l-.43.25a2 2 0 0 1-2 0l-.15-.08a2 2 0 0 0-2.73.73l-.22.38a2 2 0 0 0 .73 2.73l.15.1a2 2 0 0 1 1 1.72v.51a2 2 0 0 1-1 1.74l-.15.09a2 2 0 0 0-.73 2.73l.22.38a2 2 0 0 0 2.73.73l.15-.08a2 2 0 0 1 2 0l.43.25a2 2 0 0 1 1 1.73V20a2 2 0 0 0 2 2h.44a2 2 0 0 0 2-2v-.18a2 2 0 0 1 1-1.73l.43-.25a2 2 0 0 1 2 0l.15.08a2 2 0 0 0 2.73-.73l.22-.39a2 2 0 0 0-.73-2.73l-.15-.08a2 2 0 0 1-1-1.74v-.5a2 2 0 0 1 1-1.74l.15-.09a2 2 0 0 0 .73-2.73l-.22-.38a2 2 0 0 0-2.73-.73l-.15.08a2 2 0 0 1-2 0l-.43-.25a2 2 0 0 1-1-1.73V4a2 2 0 0 0-2-2z"/><circle cx="12" cy="12" r="3"/></svg>',
  },
]
</script>

<style scoped>
.features-section {
  padding: 80px 24px;
  background: var(--croot-bg);
  font-family: var(--croot-font-sans);
}

.section-inner {
  max-width: var(--croot-max-width);
  margin: 0 auto;
}

.section-label {
  font-family: var(--croot-font-mono);
  font-size: 0.8rem;
  text-transform: uppercase;
  letter-spacing: 0.1em;
  color: var(--croot-accent);
  margin-bottom: 8px;
}

.section-title {
  font-size: clamp(1.5rem, 3vw, 2rem);
  font-weight: 700;
  margin-bottom: 12px;
  color: var(--croot-text);
}

.section-desc {
  color: var(--croot-text-muted);
  max-width: 600px;
  margin-bottom: 48px;
  line-height: 1.6;
}

.features-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
  gap: 20px;
}

.feature-card {
  background: var(--croot-bg-surface);
  border: 1px solid var(--croot-border);
  border-radius: var(--croot-radius);
  padding: 28px;
  transition: border-color 0.2s, box-shadow 0.2s;
}

.feature-card:hover {
  border-color: var(--croot-accent);
  box-shadow: 0 0 0 1px var(--croot-accent-glow), 0 8px 24px rgba(0, 0, 0, 0.2);
}

.feature-icon {
  font-size: 1.1rem;
  margin-bottom: 12px;
  display: flex;
  align-items: center;
  justify-content: center;
  width: 40px;
  height: 40px;
  background: var(--croot-accent-glow);
  border-radius: 8px;
  color: var(--croot-accent);
}

.feature-icon :deep(svg) {
  width: 20px;
  height: 20px;
  stroke: currentColor;
  fill: none;
  stroke-width: 2;
  stroke-linecap: round;
  stroke-linejoin: round;
}

.feature-card h3 {
  font-size: 1.05rem;
  margin-bottom: 8px;
  color: var(--croot-text);
}

.feature-card p {
  color: var(--croot-text-muted);
  font-size: 0.9rem;
  line-height: 1.5;
}

.feature-card :deep(code) {
  font-family: var(--croot-font-mono);
  font-size: 0.85em;
  background: var(--croot-bg-overlay);
  padding: 2px 6px;
  border-radius: 4px;
}

@media (max-width: 640px) {
  .features-section {
    padding: 60px 16px;
  }
  .features-grid {
    grid-template-columns: 1fr;
  }
}
</style>
