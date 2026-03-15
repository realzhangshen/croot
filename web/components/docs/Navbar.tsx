'use client'

import { useState } from 'react'
import Link from 'next/link'
import { usePathname } from 'next/navigation'
import SearchDialog from './SearchDialog'

export default function Navbar() {
  const pathname = usePathname()
  const [searchOpen, setSearchOpen] = useState(false)

  const navLinks = [
    { text: 'Docs', href: '/docs/guide/getting-started/', match: /\/(guide|advanced)\// },
    { text: 'Features', href: '/docs/features/git-integration/', match: /\/features\// },
  ]

  return (
    <>
      <nav className="navbar">
        <div className="navbar-inner">
          <Link href="/" className="navbar-brand">
            <img src="/favicon.svg" alt="" width={24} height={24} />
            <span>croot</span>
          </Link>

          <div className="navbar-links">
            {navLinks.map((link) => (
              <Link
                key={link.text}
                href={link.href}
                className={`nav-link ${link.match.test(pathname) ? 'active' : ''}`}
              >
                {link.text}
              </Link>
            ))}
          </div>

          <div className="navbar-right">
            <button
              onClick={() => setSearchOpen(true)}
              className="search-trigger"
              aria-label="Search documentation"
            >
              <svg width={16} height={16} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={2} strokeLinecap="round" strokeLinejoin="round">
                <circle cx="11" cy="11" r="8" />
                <line x1="21" y1="21" x2="16.65" y2="16.65" />
              </svg>
              <span className="search-text">Search</span>
              <kbd>&#8984;K</kbd>
            </button>

            <a
              href="https://github.com/realzhangshen/croot"
              target="_blank"
              rel="noopener"
              className="github-link"
              aria-label="GitHub"
            >
              <svg viewBox="0 0 16 16" width={20} height={20}>
                <path
                  d="M8 0C3.58 0 0 3.58 0 8c0 3.54 2.29 6.53 5.47 7.59.4.07.55-.17.55-.38 0-.19-.01-.82-.01-1.49-2.01.37-2.53-.49-2.69-.94-.09-.23-.48-.94-.82-1.13-.28-.15-.68-.52-.01-.53.63-.01 1.08.58 1.23.82.72 1.21 1.87.87 2.33.66.07-.52.28-.87.51-1.07-1.78-.2-3.64-.89-3.64-3.95 0-.87.31-1.59.82-2.15-.08-.2-.36-1.02.08-2.12 0 0 .67-.21 2.2.82.64-.18 1.32-.27 2-.27.68 0 1.36.09 2 .27 1.53-1.04 2.2-.82 2.2-.82.44 1.1.16 1.92.08 2.12.51.56.82 1.27.82 2.15 0 3.07-1.87 3.75-3.65 3.95.29.25.54.73.54 1.48 0 1.07-.01 1.93-.01 2.2 0 .21.15.46.55.38A8.013 8.013 0 0016 8c0-4.42-3.58-8-8-8z"
                  fill="currentColor"
                />
              </svg>
            </a>
          </div>
        </div>
      </nav>

      <SearchDialog open={searchOpen} onClose={() => setSearchOpen(false)} />

      <style jsx>{`
        .navbar {
          position: sticky;
          top: 0;
          z-index: 50;
          background: rgba(255, 255, 255, 0.96);
          backdrop-filter: blur(8px);
          -webkit-backdrop-filter: blur(8px);
          border-bottom: 1px solid var(--croot-border);
        }
        .navbar-inner {
          max-width: var(--croot-max-width);
          margin: 0 auto;
          display: flex;
          align-items: center;
          gap: 24px;
          padding: 0 24px;
          height: 56px;
        }
        .navbar-brand {
          display: flex;
          align-items: center;
          gap: 8px;
          text-decoration: none;
          color: var(--croot-text);
          font-weight: 700;
          font-size: 1rem;
        }
        .navbar-links {
          display: flex;
          gap: 4px;
        }
        .nav-link {
          padding: 6px 12px;
          font-size: 0.9rem;
          color: var(--croot-text-secondary);
          text-decoration: none;
          border-radius: 4px;
          transition: color var(--croot-dur-fast) var(--croot-ease);
          position: relative;
        }
        .nav-link:hover {
          color: var(--croot-accent-orange);
        }
        .nav-link.active {
          color: var(--croot-accent-orange);
        }
        .nav-link.active::after {
          content: '';
          position: absolute;
          bottom: -16px;
          left: 0;
          right: 0;
          height: 2px;
          background: var(--croot-accent-orange);
        }
        .navbar-right {
          margin-left: auto;
          display: flex;
          align-items: center;
          gap: 12px;
        }
        .search-trigger {
          display: flex;
          align-items: center;
          gap: 8px;
          padding: 6px 12px;
          border: 1px solid var(--croot-border);
          border-radius: 6px;
          background: var(--croot-bg);
          color: var(--croot-text-muted);
          font-size: 0.85rem;
          cursor: pointer;
          transition: border-color var(--croot-dur-fast) var(--croot-ease);
        }
        .search-trigger:hover {
          border-color: var(--croot-border-strong);
        }
        .search-trigger kbd {
          font-family: var(--croot-font-sans);
          font-size: 0.75rem;
          padding: 1px 5px;
          border: 1px solid var(--croot-border);
          border-radius: 4px;
          background: var(--croot-bg-surface);
        }
        .search-text {
          display: none;
        }
        @media (min-width: 640px) {
          .search-text { display: inline; }
        }
        .github-link {
          color: var(--croot-text-secondary);
          display: flex;
          align-items: center;
          transition: color var(--croot-dur-fast) var(--croot-ease);
        }
        .github-link:hover {
          color: var(--croot-text);
        }
      `}</style>
    </>
  )
}
