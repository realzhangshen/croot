'use client'

import { useState } from 'react'

const tabs = [
  { id: 'homebrew', label: 'Homebrew' },
  { id: 'source', label: 'From Source' },
  { id: 'binary', label: 'Pre-built Binary' },
] as const

type TabId = (typeof tabs)[number]['id']

const copyTexts: Record<TabId, string> = {
  homebrew: 'brew install realzhangshen/croot/croot',
  source: 'git clone https://github.com/realzhangshen/croot.git\ncd croot\ncargo build --release',
  binary: '# Download from GitHub Releases for your platform\ncurl -fsSL https://github.com/realzhangshen/croot/releases/latest\n# Then extract and move to your PATH',
}

export default function HomeInstall() {
  const [activeTab, setActiveTab] = useState<TabId>('homebrew')
  const [copiedTab, setCopiedTab] = useState<TabId | null>(null)

  function copyCode(tabId: TabId) {
    navigator.clipboard.writeText(copyTexts[tabId]).then(() => {
      setCopiedTab(tabId)
      setTimeout(() => setCopiedTab(null), 2000)
    })
  }

  return (
    <section
      style={{
        padding: '80px 24px',
        background: 'var(--croot-bg)',
        fontFamily: 'var(--croot-font-sans)',
      }}
    >
      <div style={{ maxWidth: 'var(--croot-max-width)', margin: '0 auto' }}>
        <p className="section-label">Installation</p>
        <h2 className="section-title">Get started in seconds</h2>
        <p
          style={{
            color: 'var(--croot-text-secondary)',
            maxWidth: 600,
            marginBottom: 48,
            lineHeight: 1.6,
          }}
        >
          Install croot with your preferred method.
        </p>

        <div style={{ display: 'flex', gap: '0.55rem', flexWrap: 'wrap', maxWidth: 640, marginBottom: 4 }}>
          {tabs.map((tab) => (
            <button
              key={tab.id}
              onClick={() => setActiveTab(tab.id)}
              className="install-tab"
              style={{
                padding: '0.35rem 0.65rem',
                borderRadius: 'var(--croot-radius-pill)',
                border: '1px solid',
                borderColor: activeTab === tab.id ? 'var(--croot-accent)' : 'var(--croot-border)',
                fontSize: 13,
                lineHeight: 1.5,
                color: activeTab === tab.id ? 'var(--croot-accent-white)' : 'var(--croot-text-secondary)',
                background: activeTab === tab.id ? 'var(--croot-accent)' : 'transparent',
                cursor: 'pointer',
                fontFamily: 'var(--croot-font-sans)',
                transition: 'color var(--croot-dur-fast) var(--croot-ease), background-color var(--croot-dur-fast) var(--croot-ease), border-color var(--croot-dur-fast) var(--croot-ease)',
              }}
            >
              {tab.label}
            </button>
          ))}
        </div>

        <div style={{ maxWidth: 640 }}>
          {activeTab === 'homebrew' && (
            <InstallBlock
              onCopy={() => copyCode('homebrew')}
              copied={copiedTab === 'homebrew'}
            >
              <pre>brew install realzhangshen/croot/croot</pre>
            </InstallBlock>
          )}

          {activeTab === 'source' && (
            <InstallBlock
              onCopy={() => copyCode('source')}
              copied={copiedTab === 'source'}
            >
              <pre>
                <span className="comment"># Requires Rust 1.88+</span>{'\n'}
                git clone https://github.com/realzhangshen/croot.git{'\n'}
                cd croot{'\n'}
                cargo build --release{'\n\n'}
                <span className="comment"># Binary is at target/release/croot</span>
              </pre>
            </InstallBlock>
          )}

          {activeTab === 'binary' && (
            <>
              <InstallBlock
                onCopy={() => copyCode('binary')}
                copied={copiedTab === 'binary'}
              >
                <pre>
                  <span className="comment"># Download from GitHub Releases for your platform</span>{'\n'}
                  <span className="comment"># Available targets:</span>{'\n'}
                  <span className="comment">#   aarch64-apple-darwin    (Apple Silicon)</span>{'\n'}
                  <span className="comment">#   x86_64-apple-darwin     (Intel Mac)</span>{'\n'}
                  <span className="comment">#   x86_64-unknown-linux-gnu</span>{'\n'}
                  <span className="comment">#   aarch64-unknown-linux-gnu</span>{'\n\n'}
                  <span className="comment"># Example (macOS Apple Silicon):</span>{'\n'}
                  TAG=v0.4.0{'\n'}
                  curl -fsSL &quot;https://github.com/realzhangshen/croot/releases/download/${'{'}TAG{'}'}/croot-${'{'}TAG{'}'}-aarch64-apple-darwin.tar.gz&quot; | tar xz{'\n'}
                  sudo mv croot /usr/local/bin/
                </pre>
              </InstallBlock>
              <p style={{ marginTop: 12, fontSize: '0.85rem', color: 'var(--croot-text-secondary)' }}>
                Download binaries from the{' '}
                <a
                  href="https://github.com/realzhangshen/croot/releases"
                  style={{ color: 'var(--croot-text)', textDecoration: 'underline', textUnderlineOffset: 2 }}
                >
                  Releases page
                </a>
                .
              </p>
            </>
          )}
        </div>
      </div>

      <style jsx>{`
        .section-label {
          display: inline-flex;
          align-items: center;
          gap: 0.35rem;
          padding: 0.35rem 0.7rem;
          border: 1px solid var(--croot-border);
          border-radius: var(--croot-radius-pill);
          font-size: 13px;
          font-weight: 500;
          color: var(--croot-text-muted);
          background: var(--croot-bg-hover);
          margin-bottom: 12px;
        }
        .section-title {
          font-size: clamp(1.5rem, 3vw, 2rem);
          font-weight: 700;
          margin-bottom: 12px;
          color: var(--croot-text);
        }
        .comment {
          color: var(--croot-text-muted);
        }
        pre {
          padding: 16px;
          font-size: 0.875rem;
          overflow-x: auto;
          line-height: 1.7;
          font-family: var(--croot-font-mono);
          color: var(--croot-text);
          margin: 0;
        }
      `}</style>
    </section>
  )
}

function InstallBlock({
  onCopy,
  copied,
  children,
}: {
  onCopy: () => void
  copied: boolean
  children: React.ReactNode
}) {
  return (
    <div
      style={{
        background: 'var(--croot-bg-surface)',
        border: '1px solid var(--croot-border)',
        borderRadius: 'var(--croot-radius-sm)',
        marginTop: 16,
        overflow: 'hidden',
      }}
    >
      <div
        style={{
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between',
          padding: '10px 16px',
          borderBottom: '1px solid var(--croot-border)',
          background: 'var(--croot-bg-elevated)',
        }}
      >
        <span
          style={{
            fontFamily: 'var(--croot-font-mono)',
            fontSize: '0.75rem',
            color: 'var(--croot-text-muted)',
            textTransform: 'uppercase',
            letterSpacing: '0.05em',
          }}
        >
          Terminal
        </span>
        <button
          onClick={onCopy}
          style={{
            background: 'none',
            border: '1px solid var(--croot-border)',
            color: copied ? 'var(--croot-text)' : 'var(--croot-text-muted)',
            padding: '4px 10px',
            borderRadius: 'var(--croot-radius-pill)',
            fontSize: '0.75rem',
            cursor: 'pointer',
            fontFamily: 'var(--croot-font-mono)',
            transition: 'border-color var(--croot-dur-fast) var(--croot-ease), color var(--croot-dur-fast) var(--croot-ease)',
          }}
        >
          {copied ? 'Copied!' : 'Copy'}
        </button>
      </div>
      {children}
    </div>
  )
}
