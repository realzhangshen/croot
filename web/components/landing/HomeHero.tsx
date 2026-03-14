'use client'

import { useState } from 'react'

const COMMAND = 'brew install realzhangshen/croot/croot'

export default function HomeHero() {
  const [copied, setCopied] = useState(false)

  function copyCommand() {
    navigator.clipboard.writeText(COMMAND).then(() => {
      setCopied(true)
      setTimeout(() => setCopied(false), 2000)
    })
  }

  return (
    <section
      className="text-center"
      style={{
        padding: '160px 24px 80px',
        background: 'var(--croot-bg)',
        fontFamily: 'var(--croot-font-sans)',
      }}
    >
      <div style={{ maxWidth: 'var(--croot-max-width)', margin: '0 auto' }}>
        <h1
          className="reveal"
          style={{
            fontSize: 'clamp(3rem, 8vw, 5rem)',
            fontWeight: 700,
            letterSpacing: '-0.03em',
            marginBottom: 16,
            color: 'var(--croot-text)',
          }}
        >
          croot
        </h1>

        <p
          className="reveal reveal-delay-1"
          style={{
            fontSize: 'clamp(1.1rem, 2.5vw, 1.35rem)',
            color: 'var(--croot-text-secondary)',
            maxWidth: 560,
            margin: '0 auto 40px',
            lineHeight: 1.6,
          }}
        >
          The VS Code sidebar for your terminal. Navigate files, preview code,
          and manage your project — all from the command line.
        </p>

        <div
          className="reveal reveal-delay-2"
          onClick={copyCommand}
          title="Click to copy"
          style={{
            display: 'inline-flex',
            alignItems: 'center',
            background: 'var(--croot-bg-surface)',
            border: '1px solid var(--croot-border)',
            borderRadius: 'var(--croot-radius-sm)',
            padding: '12px 20px',
            fontFamily: 'var(--croot-font-mono)',
            fontSize: '0.95rem',
            gap: 12,
            marginBottom: 32,
            cursor: 'pointer',
            transition: 'border-color var(--croot-dur-fast) var(--croot-ease)',
            color: 'var(--croot-text)',
          }}
        >
          <span style={{ color: 'var(--croot-text-muted)', userSelect: 'none' }}>$</span>
          <span>{COMMAND}</span>
          <button
            aria-label="Copy to clipboard"
            style={{
              background: 'none',
              border: 'none',
              cursor: 'pointer',
              padding: 4,
              display: 'flex',
              alignItems: 'center',
              color: copied ? 'var(--croot-text)' : 'var(--croot-text-muted)',
              transition: 'color var(--croot-dur-fast) var(--croot-ease)',
            }}
          >
            {copied ? (
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={2} strokeLinecap="round" strokeLinejoin="round" width={18} height={18}>
                <polyline points="20 6 9 17 4 12" />
              </svg>
            ) : (
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={2} strokeLinecap="round" strokeLinejoin="round" width={18} height={18}>
                <rect x={9} y={9} width={13} height={13} rx={2} />
                <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1" />
              </svg>
            )}
          </button>
        </div>

        <div
          className="reveal reveal-delay-3"
          style={{ display: 'flex', gap: 12, justifyContent: 'center', flexWrap: 'wrap' }}
        >
          <a
            href="/croot/docs/guide/getting-started/"
            style={{
              display: 'inline-flex',
              alignItems: 'center',
              gap: 8,
              padding: '0.72rem 1.15rem',
              borderRadius: 'var(--croot-radius-pill)',
              fontSize: '0.95rem',
              fontWeight: 600,
              textDecoration: 'none',
              lineHeight: 1,
              letterSpacing: '0.01em',
              background: 'var(--croot-accent)',
              color: 'var(--croot-accent-white)',
              border: '1px solid transparent',
              transition: 'transform var(--croot-dur-fast) var(--croot-ease), filter var(--croot-dur-fast) var(--croot-ease)',
            }}
          >
            Get Started
          </a>
          <a
            href="https://github.com/realzhangshen/croot"
            target="_blank"
            rel="noopener"
            style={{
              display: 'inline-flex',
              alignItems: 'center',
              gap: 8,
              padding: '0.72rem 1.15rem',
              borderRadius: 'var(--croot-radius-pill)',
              fontSize: '0.95rem',
              fontWeight: 600,
              textDecoration: 'none',
              lineHeight: 1,
              letterSpacing: '0.01em',
              background: 'transparent',
              color: 'var(--croot-text)',
              border: '1px solid var(--croot-border-strong)',
              transition: 'transform var(--croot-dur-fast) var(--croot-ease), border-color var(--croot-dur-fast) var(--croot-ease)',
            }}
          >
            <svg viewBox="0 0 16 16" width={18} height={18}>
              <path
                d="M8 0C3.58 0 0 3.58 0 8c0 3.54 2.29 6.53 5.47 7.59.4.07.55-.17.55-.38 0-.19-.01-.82-.01-1.49-2.01.37-2.53-.49-2.69-.94-.09-.23-.48-.94-.82-1.13-.28-.15-.68-.52-.01-.53.63-.01 1.08.58 1.23.82.72 1.21 1.87.87 2.33.66.07-.52.28-.87.51-1.07-1.78-.2-3.64-.89-3.64-3.95 0-.87.31-1.59.82-2.15-.08-.2-.36-1.02.08-2.12 0 0 .67-.21 2.2.82.64-.18 1.32-.27 2-.27.68 0 1.36.09 2 .27 1.53-1.04 2.2-.82 2.2-.82.44 1.1.16 1.92.08 2.12.51.56.82 1.27.82 2.15 0 3.07-1.87 3.75-3.65 3.95.29.25.54.73.54 1.48 0 1.07-.01 1.93-.01 2.2 0 .21.15.46.55.38A8.013 8.013 0 0016 8c0-4.42-3.58-8-8-8z"
                fill="currentColor"
              />
            </svg>
            View on GitHub
          </a>
        </div>
      </div>

      <style jsx>{`
        .reveal {
          animation: fade-up var(--croot-dur-mid) var(--croot-ease) both;
        }
        .reveal-delay-1 { animation-delay: 40ms; }
        .reveal-delay-2 { animation-delay: 80ms; }
        .reveal-delay-3 { animation-delay: 120ms; }
        @media (max-width: 640px) {
          section { padding: 120px 16px 60px !important; }
        }
      `}</style>
    </section>
  )
}
