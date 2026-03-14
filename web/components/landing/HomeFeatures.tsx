'use client'

import type { ReactNode } from 'react'

interface Feature {
  title: string
  desc: ReactNode
  icon: ReactNode
}

const features: Feature[] = [
  {
    title: 'Git Status Integration',
    desc: 'See modified, staged, untracked, and conflicted files at a glance. Status propagates to parent directories so you always know where changes live.',
    icon: <svg viewBox="0 0 24 24"><circle cx="12" cy="12" r="3"/><path d="M6.3 20.3a2.4 2.4 0 0 0 3.4 0L12 18l2.3 2.3a2.4 2.4 0 0 0 3.4 0l.3-.3a2.4 2.4 0 0 0 0-3.4L15.7 14 18 11.7a2.4 2.4 0 0 0 0-3.4l-.3-.3a2.4 2.4 0 0 0-3.4 0L12 10.3 9.7 8a2.4 2.4 0 0 0-3.4 0l-.3.3a2.4 2.4 0 0 0 0 3.4L8.3 14 6 16.3a2.4 2.4 0 0 0 0 3.4z"/></svg>,
  },
  {
    title: 'Syntax-Highlighted Preview',
    desc: 'Split-pane preview with syntax highlighting for 150+ languages, Markdown rendering, binary hex dumps, and a resizable divider.',
    icon: <svg viewBox="0 0 24 24"><polyline points="16 18 22 12 16 6"/><polyline points="8 6 2 12 8 18"/></svg>,
  },
  {
    title: 'Fuzzy Search',
    desc: <>Hit <code>/</code> to fuzzy-filter the tree by filename. Navigate matches with Tab and Shift+Tab. The match count updates in real time.</>,
    icon: <svg viewBox="0 0 24 24"><circle cx="11" cy="11" r="8"/><line x1="21" y1="21" x2="16.65" y2="16.65"/></svg>,
  },
  {
    title: 'Full Mouse Support',
    desc: 'Click to select, double-click to expand, right-click for context menus, drag to resize panes, and scroll naturally. Or stay on the keyboard.',
    icon: <svg viewBox="0 0 24 24"><path d="M4 4h6v6H4z"/><path d="M14 4h6v6h-6z"/><path d="M4 14h6v6H4z"/><path d="M14 14h6v6h-6z"/></svg>,
  },
  {
    title: 'File Operations',
    desc: <>Create, rename, and delete files and directories. Multi-select with <code>v</code> for bulk operations. All without leaving the terminal.</>,
    icon: <svg viewBox="0 0 24 24"><path d="M14.5 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7.5L14.5 2z"/><polyline points="14 2 14 8 20 8"/><line x1="12" y1="18" x2="12" y2="12"/><line x1="9" y1="15" x2="15" y2="15"/></svg>,
  },
  {
    title: 'Real-Time File Watching',
    desc: 'The tree auto-refreshes when files change on disk. Git status updates automatically. A debounced watcher keeps things fast.',
    icon: <svg viewBox="0 0 24 24"><path d="M12 2v4"/><path d="M12 18v4"/><path d="M4.93 4.93l2.83 2.83"/><path d="M16.24 16.24l2.83 2.83"/><path d="M2 12h4"/><path d="M18 12h4"/><path d="M4.93 19.07l2.83-2.83"/><path d="M16.24 7.76l2.83-2.83"/></svg>,
  },
  {
    title: 'Nerd Font Icons',
    desc: '100+ file type icons render when a Nerd Font is installed. Directories, Rust, JavaScript, Markdown — each gets its own glyph.',
    icon: <svg viewBox="0 0 24 24"><path d="M4 20h16a2 2 0 0 0 2-2V8a2 2 0 0 0-2-2h-7.93a2 2 0 0 1-1.66-.9l-.82-1.2A2 2 0 0 0 7.93 3H4a2 2 0 0 0-2 2v13c0 1.1.9 2 2 2Z"/></svg>,
  },
  {
    title: 'Context Menus',
    desc: 'Right-click any node for contextual actions: open in editor, copy path, reveal in Finder, rename, delete, and more.',
    icon: <svg viewBox="0 0 24 24"><rect x="3" y="3" width="18" height="18" rx="2"/><path d="M3 9h18"/><path d="M9 21V9"/></svg>,
  },
  {
    title: 'Configurable',
    desc: <>TOML config for hidden files, exclusion patterns, compact folders, preview settings, and custom editor commands. Manage it with <code>croot&nbsp;config</code>.</>,
    icon: <svg viewBox="0 0 24 24"><path d="M12.22 2h-.44a2 2 0 0 0-2 2v.18a2 2 0 0 1-1 1.73l-.43.25a2 2 0 0 1-2 0l-.15-.08a2 2 0 0 0-2.73.73l-.22.38a2 2 0 0 0 .73 2.73l.15.1a2 2 0 0 1 1 1.72v.51a2 2 0 0 1-1 1.74l-.15.09a2 2 0 0 0-.73 2.73l.22.38a2 2 0 0 0 2.73.73l.15-.08a2 2 0 0 1 2 0l.43.25a2 2 0 0 1 1 1.73V20a2 2 0 0 0 2 2h.44a2 2 0 0 0 2-2v-.18a2 2 0 0 1 1-1.73l.43-.25a2 2 0 0 1 2 0l.15.08a2 2 0 0 0 2.73-.73l.22-.39a2 2 0 0 0-.73-2.73l-.15-.08a2 2 0 0 1-1-1.74v-.5a2 2 0 0 1 1-1.74l.15-.09a2 2 0 0 0 .73-2.73l-.22-.38a2 2 0 0 0-2.73-.73l-.15.08a2 2 0 0 1-2 0l-.43-.25a2 2 0 0 1-1-1.73V4a2 2 0 0 0-2-2z"/><circle cx="12" cy="12" r="3"/></svg>,
  },
]

export default function HomeFeatures() {
  return (
    <section
      style={{
        padding: '80px 24px',
        background: 'var(--croot-bg)',
        fontFamily: 'var(--croot-font-sans)',
      }}
    >
      <div style={{ maxWidth: 'var(--croot-max-width)', margin: '0 auto' }}>
        <p
          style={{
            display: 'inline-flex',
            alignItems: 'center',
            gap: '0.35rem',
            padding: '0.35rem 0.7rem',
            border: '1px solid var(--croot-border)',
            borderRadius: 'var(--croot-radius-pill)',
            fontSize: 13,
            fontWeight: 500,
            color: 'var(--croot-text-muted)',
            background: 'var(--croot-bg-hover)',
            marginBottom: 12,
          }}
        >
          Features
        </p>
        <h2
          style={{
            fontSize: 'clamp(1.5rem, 3vw, 2rem)',
            fontWeight: 700,
            marginBottom: 12,
            color: 'var(--croot-text)',
          }}
        >
          Everything you need in a file explorer
        </h2>
        <p
          style={{
            color: 'var(--croot-text-secondary)',
            maxWidth: 600,
            marginBottom: 48,
            lineHeight: 1.6,
          }}
        >
          A fast, keyboard-driven file tree with git awareness, syntax previews,
          and mouse support. No Electron required.
        </p>

        <div
          className="features-grid"
        >
          {features.map((feature, i) => (
            <div
              key={feature.title}
              className="feature-card"
              style={{
                animationDelay: `${i * 40}ms`,
              }}
            >
              <div className="feature-icon">
                {feature.icon}
              </div>
              <h3 style={{ fontSize: '1.05rem', marginBottom: 8, color: 'var(--croot-text)' }}>
                {feature.title}
              </h3>
              <p className="feature-desc">
                {feature.desc}
              </p>
            </div>
          ))}
        </div>
      </div>

      <style jsx>{`
        .features-grid {
          display: grid;
          grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
          gap: 20px;
        }
        .feature-card {
          background: var(--croot-bg-surface);
          border: 1px solid var(--croot-border);
          border-radius: var(--croot-radius-md);
          padding: 28px;
          box-shadow: var(--croot-shadow);
          transition: transform var(--croot-dur-fast) var(--croot-ease),
                      box-shadow var(--croot-dur-fast) var(--croot-ease);
          animation: fade-up var(--croot-dur-mid) var(--croot-ease) both;
        }
        .feature-card:hover {
          transform: translateY(-2px);
          box-shadow: var(--croot-shadow-elevated);
        }
        .feature-icon {
          margin-bottom: 12px;
          display: flex;
          align-items: center;
          justify-content: center;
          width: 40px;
          height: 40px;
          background: rgba(38, 37, 30, 0.04);
          border-radius: 8px;
          color: var(--croot-text-secondary);
        }
        .feature-desc {
          color: var(--croot-text-secondary);
          font-size: 0.9rem;
          line-height: 1.5;
        }
        @media (max-width: 640px) {
          .features-grid {
            grid-template-columns: 1fr;
          }
        }
      `}</style>

      <style jsx global>{`
        .feature-icon svg {
          width: 20px;
          height: 20px;
          stroke: currentColor;
          fill: none;
          stroke-width: 2;
          stroke-linecap: round;
          stroke-linejoin: round;
        }
        .feature-desc code {
          font-family: var(--croot-font-mono);
          font-size: 0.85em;
          background: var(--croot-bg-elevated);
          padding: 2px 6px;
          border-radius: 4px;
        }
      `}</style>
    </section>
  )
}
