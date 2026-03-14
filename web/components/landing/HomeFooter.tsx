'use client'

export default function HomeFooter() {
  return (
    <footer
      style={{
        padding: '40px 24px',
        background: 'var(--croot-bg)',
        fontFamily: 'var(--croot-font-sans)',
      }}
    >
      <div
        style={{
          maxWidth: 'var(--croot-max-width)',
          margin: '0 auto',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between',
          flexWrap: 'wrap',
          gap: 16,
        }}
      >
        <div style={{ fontSize: '0.85rem', color: 'var(--croot-text-muted)' }}>
          Built with Rust &amp;{' '}
          <a
            href="https://ratatui.rs"
            className="footer-link"
          >
            Ratatui
          </a>
        </div>
        <ul
          style={{
            display: 'flex',
            gap: 20,
            listStyle: 'none',
            margin: 0,
            padding: 0,
          }}
        >
          <li>
            <a href="https://github.com/realzhangshen/croot" className="footer-link">
              GitHub
            </a>
          </li>
          <li>
            <a
              href="https://github.com/realzhangshen/croot/blob/main/CHANGELOG.md"
              className="footer-link"
            >
              Changelog
            </a>
          </li>
          <li>
            <a
              href="https://github.com/realzhangshen/croot/blob/main/LICENSE"
              className="footer-link"
            >
              MIT License
            </a>
          </li>
        </ul>
      </div>

      <style jsx>{`
        .footer-link {
          color: var(--croot-text-muted);
          font-size: 0.85rem;
          text-decoration: none;
          transition: color var(--croot-dur-fast) var(--croot-ease);
        }
        .footer-link:hover {
          color: var(--croot-text);
        }
      `}</style>
    </footer>
  )
}
