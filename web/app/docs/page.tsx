'use client'

import Link from 'next/link'
import { sidebar } from '@/lib/sidebar'

export default function DocsLandingPage() {
  return (
    <div style={{ maxWidth: 768, margin: '0 auto' }}>
      <h1
        style={{
          fontSize: '2rem',
          fontWeight: 700,
          letterSpacing: '-0.02em',
          marginBottom: 8,
        }}
      >
        Documentation
      </h1>
      <p
        style={{
          color: 'var(--croot-text-secondary)',
          fontSize: '1.1rem',
          marginBottom: 40,
        }}
      >
        Everything you need to navigate, preview, and manage your projects from
        the terminal.
      </p>

      <div className="cards-grid">
        {sidebar.map((group) => (
          <div key={group.text} className="card">
            <div className="card-header">
              <h2 className="card-title">{group.text}</h2>
              <p className="card-desc">{group.description}</p>
            </div>
            <ul className="card-links">
              {group.items.map((item) => (
                <li key={item.link}>
                  <Link href={`${item.link}/`} className="card-link">
                    {item.text}
                  </Link>
                </li>
              ))}
            </ul>
          </div>
        ))}
      </div>

      <style jsx>{`
        .cards-grid {
          display: grid;
          grid-template-columns: repeat(1, 1fr);
          gap: 16px;
        }
        @media (min-width: 640px) {
          .cards-grid {
            grid-template-columns: repeat(2, 1fr);
          }
        }
        @media (min-width: 960px) {
          .cards-grid {
            grid-template-columns: repeat(3, 1fr);
          }
        }
        .card {
          background: var(--croot-bg-surface);
          border: 1px solid var(--croot-border);
          border-radius: var(--croot-radius-sm);
          overflow: hidden;
          transition: border-color var(--croot-dur-fast) var(--croot-ease),
            box-shadow var(--croot-dur-fast) var(--croot-ease);
        }
        .card:hover {
          border-color: var(--croot-border-strong);
          box-shadow: var(--croot-shadow);
        }
        .card-header {
          padding: 16px;
          background: var(--croot-bg-elevated);
          border-bottom: 1px solid var(--croot-border);
        }
        .card-title {
          font-size: 1rem;
          font-weight: 700;
          margin: 0 0 4px;
        }
        .card-desc {
          font-size: 0.85rem;
          color: var(--croot-text-secondary);
          margin: 0;
          line-height: 1.4;
        }
        .card-links {
          list-style: none;
          margin: 0;
          padding: 12px 16px;
        }
        .card-links li {
          margin: 0;
        }
      `}</style>

      <style jsx global>{`
        .card-link {
          display: block;
          padding: 6px 0;
          font-size: 0.9rem;
          color: var(--croot-text-secondary);
          text-decoration: none;
          transition: color var(--croot-dur-fast) var(--croot-ease);
        }
        .card-link:hover {
          color: var(--croot-accent-orange);
        }
      `}</style>
    </div>
  )
}
