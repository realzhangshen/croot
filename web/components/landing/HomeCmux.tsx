'use client'

export default function HomeCmux() {
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
          Workflow
        </p>
        <h2
          style={{
            fontSize: 'clamp(1.5rem, 3vw, 2rem)',
            fontWeight: 700,
            marginBottom: 12,
            color: 'var(--croot-text)',
          }}
        >
          Pair with cmux
        </h2>

        <div className="cmux-grid">
          <div>
            <p style={{ color: 'var(--croot-text-secondary)', marginBottom: 16, fontSize: '0.95rem', lineHeight: 1.7 }}>
              croot works great alongside{' '}
              <a
                href="https://github.com/manaflow-ai/cmux"
                style={{ color: 'var(--croot-text)', textDecoration: 'underline', textUnderlineOffset: 2 }}
              >
                cmux
              </a>{' '}
              for a full vibe coding setup in the terminal — file tree on one
              side, editor and shell on the other.
            </p>
            <p style={{ color: 'var(--croot-text-secondary)', marginBottom: 16, fontSize: '0.95rem', lineHeight: 1.7 }}>
              Together they give you a VS Code-like workspace that lives
              entirely in your terminal. No Electron, no GUI, just fast tools
              that compose well.
            </p>
            <a
              href="https://github.com/manaflow-ai/cmux"
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
                marginTop: 8,
              }}
            >
              Learn about cmux
            </a>
          </div>

          <div
            style={{
              background: 'var(--croot-bg-surface)',
              border: '1px solid var(--croot-border)',
              borderRadius: 'var(--croot-radius-md)',
              padding: 24,
              fontFamily: 'var(--croot-font-mono)',
              fontSize: '0.8rem',
              color: 'var(--croot-text-secondary)',
              lineHeight: 1.6,
              boxShadow: 'var(--croot-shadow)',
            }}
          >
            <pre style={{ margin: 0 }}>
              <span className="dim">┌──────────────┬────────────────────────┐</span>{'\n'}
              <span className="dim">│</span> <span className="label">croot</span>{'        '}<span className="dim">│</span> <span className="label">$EDITOR</span>{'                '}<span className="dim">│</span>{'\n'}
              <span className="dim">│</span>{'              '}<span className="dim">│</span>{'                        '}<span className="dim">│</span>{'\n'}
              <span className="dim">│</span>{'  src/        '}<span className="dim">│</span>{'  fn main() {           '}<span className="dim">│</span>{'\n'}
              <span className="dim">│</span>{'    main.rs   '}<span className="dim">│</span>{'      println!("hello");'}<span className="dim">│</span>{'\n'}
              <span className="dim">│</span>{'    lib.rs    '}<span className="dim">│</span>{'  }                     '}<span className="dim">│</span>{'\n'}
              <span className="dim">│</span>{'  Cargo.toml  '}<span className="dim">│</span>{'                        '}<span className="dim">│</span>{'\n'}
              <span className="dim">│</span>{'  README.md   '}<span className="dim">│────────────────────────┤</span>{'\n'}
              <span className="dim">│</span>{'              '}<span className="dim">│</span> <span className="label">$ </span>cargo run{'              '}<span className="dim">│</span>{'\n'}
              <span className="dim">│</span>{'              '}<span className="dim">│</span>{'                        '}<span className="dim">│</span>{'\n'}
              <span className="dim">└──────────────┴────────────────────────┘</span>
            </pre>
          </div>
        </div>
      </div>

      <style jsx>{`
        .cmux-grid {
          display: grid;
          grid-template-columns: 1fr 1fr;
          gap: 48px;
          align-items: center;
        }
        .dim { color: var(--croot-border-strong); }
        .label { color: var(--croot-text); }
        @media (max-width: 768px) {
          .cmux-grid {
            grid-template-columns: 1fr;
            gap: 32px;
          }
          section { padding: 60px 16px !important; }
        }
      `}</style>
    </section>
  )
}
