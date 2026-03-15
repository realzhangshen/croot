'use client'

import Link from 'next/link'
import { usePathname } from 'next/navigation'
import { sidebar } from '@/lib/sidebar'

export default function Sidebar() {
  const pathname = usePathname()

  return (
    <aside className="sidebar">
      <nav>
        {sidebar.map((group) => (
          <div key={group.text} className="sidebar-group">
            <div className="group-title">{group.text}</div>
            <ul>
              {group.items.map((item) => {
                const href = `${item.link}/`
                const isActive = pathname === href || pathname === item.link
                return (
                  <li key={item.link}>
                    <Link
                      href={href}
                      className={`sidebar-link ${isActive ? 'active' : ''}`}
                    >
                      {item.text}
                    </Link>
                  </li>
                )
              })}
            </ul>
          </div>
        ))}
      </nav>

      <style jsx>{`
        .sidebar {
          width: 256px;
          flex-shrink: 0;
          padding: 24px 0 24px 24px;
          border-right: 1px solid var(--croot-border);
          position: sticky;
          top: 56px;
          height: calc(100vh - 56px);
          overflow-y: auto;
          display: none;
        }
        @media (min-width: 960px) {
          .sidebar { display: block; }
        }
        .sidebar-group {
          margin-bottom: 28px;
        }
        .group-title {
          text-transform: uppercase;
          font-size: 11px;
          font-weight: 600;
          letter-spacing: 0.08em;
          color: var(--croot-text-muted);
          margin-bottom: 8px;
          padding: 0 8px;
        }
        ul {
          list-style: none;
          margin: 0;
          padding: 0;
        }
      `}</style>

      <style jsx global>{`
        .sidebar-link {
          display: block;
          padding: 6px 8px;
          font-size: 0.9rem;
          color: var(--croot-text-secondary);
          text-decoration: none;
          border-radius: 6px;
          border-left: 2px solid transparent;
          transition: color var(--croot-dur-fast) var(--croot-ease),
                      background var(--croot-dur-fast) var(--croot-ease);
        }
        .sidebar-link:hover {
          color: var(--croot-accent-orange);
          background: var(--croot-bg-hover);
        }
        .sidebar-link.active {
          color: var(--croot-accent-orange);
          font-weight: 600;
          border-left-color: var(--croot-accent-orange);
        }
      `}</style>
    </aside>
  )
}
