'use client'

import { useState, useEffect } from 'react'

export default function HomeDemo() {
  const [loaded, setLoaded] = useState(false)

  useEffect(() => {
    const img = new Image()
    img.src = '/croot/demo.gif'
    img.onload = () => setLoaded(true)
  }, [])

  return (
    <section
      style={{
        background: 'var(--croot-bg)',
        padding: '80px 24px',
        fontFamily: 'var(--croot-font-sans)',
      }}
    >
      <div style={{ maxWidth: 'var(--croot-max-width)', margin: '0 auto', textAlign: 'center' }}>
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
          Demo
        </p>
        <h2
          style={{
            fontSize: 'clamp(1.5rem, 3vw, 2rem)',
            fontWeight: 700,
            marginBottom: 12,
            color: 'var(--croot-text)',
          }}
        >
          See it in action
        </h2>
        <p
          style={{
            color: 'var(--croot-text-secondary)',
            maxWidth: 600,
            margin: '0 auto 48px',
            lineHeight: 1.6,
          }}
        >
          Navigate your project, preview files, and manage everything from the
          terminal.
        </p>

        <div
          style={{
            maxWidth: 840,
            margin: '0 auto',
            borderRadius: 'var(--croot-radius-md)',
            overflow: 'hidden',
            border: '1px solid var(--croot-border)',
            boxShadow: 'var(--croot-shadow-elevated)',
          }}
        >
          <div
            style={{
              background: 'var(--croot-bg-elevated)',
              padding: '12px 16px',
              display: 'flex',
              alignItems: 'center',
              gap: 8,
              borderBottom: '1px solid var(--croot-border)',
            }}
          >
            <div style={{ width: 12, height: 12, borderRadius: '50%', background: '#ff5f56' }} />
            <div style={{ width: 12, height: 12, borderRadius: '50%', background: '#ffbd2e' }} />
            <div style={{ width: 12, height: 12, borderRadius: '50%', background: '#27c93f' }} />
            <span
              style={{
                flex: 1,
                textAlign: 'center',
                fontFamily: 'var(--croot-font-mono)',
                fontSize: '0.8rem',
                color: 'var(--croot-text-muted)',
                marginRight: 44,
              }}
            >
              croot ~/project
            </span>
          </div>
          <div
            style={{
              background: '#0d1117',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              minHeight: 400,
            }}
          >
            {loaded ? (
              <img
                src="/croot/demo.gif"
                alt="croot demo showing file tree navigation, syntax preview, and git status"
                style={{ width: '100%', display: 'block' }}
              />
            ) : (
              <div
                style={{
                  color: '#8b949e',
                  fontFamily: 'var(--croot-font-mono)',
                  fontSize: '0.85rem',
                  textAlign: 'center',
                  padding: 40,
                }}
              >
                <p>
                  $ croot{' '}
                  <span
                    style={{
                      display: 'inline-block',
                      width: 8,
                      height: '1.2em',
                      background: '#3fb950',
                      verticalAlign: 'text-bottom',
                      animation: 'blink 1s step-end infinite',
                    }}
                  />
                </p>
                <p style={{ marginTop: 16 }}>
                  Demo GIF is auto-generated on each release.
                </p>
                <p>Clone the repo and try it yourself!</p>
              </div>
            )}
          </div>
        </div>
      </div>
    </section>
  )
}
