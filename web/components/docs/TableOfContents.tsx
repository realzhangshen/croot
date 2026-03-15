'use client'

import { useEffect, useState } from 'react'
import type { DocHeading } from '@/lib/docs'

export default function TableOfContents({ headings }: { headings: DocHeading[] }) {
  const [activeId, setActiveId] = useState<string>('')

  useEffect(() => {
    if (headings.length === 0) return

    const observer = new IntersectionObserver(
      (entries) => {
        for (const entry of entries) {
          if (entry.isIntersecting) {
            setActiveId(entry.target.id)
          }
        }
      },
      { rootMargin: '0px 0px -80% 0px', threshold: 0 }
    )

    for (const h of headings) {
      const el = document.getElementById(h.id)
      if (el) observer.observe(el)
    }

    return () => observer.disconnect()
  }, [headings])

  if (headings.length === 0) return null

  return (
    <aside className="toc">
      <div className="toc-title">ON THIS PAGE</div>
      <ul>
        {headings.map((h) => (
          <li key={h.id} style={{ paddingLeft: h.level === 3 ? 12 : 0 }}>
            <a
              href={`#${h.id}`}
              className={`toc-link ${activeId === h.id ? 'active' : ''}`}
              onClick={(e) => {
                e.preventDefault()
                document.getElementById(h.id)?.scrollIntoView({ behavior: 'smooth' })
                setActiveId(h.id)
              }}
            >
              {h.text}
            </a>
          </li>
        ))}
      </ul>

      <style jsx>{`
        .toc {
          width: 200px;
          flex-shrink: 0;
          position: sticky;
          top: 80px;
          max-height: calc(100vh - 100px);
          overflow-y: auto;
          padding: 0 16px;
          display: none;
        }
        @media (min-width: 1200px) {
          .toc { display: block; }
        }
        .toc-title {
          font-size: 11px;
          font-weight: 600;
          letter-spacing: 0.08em;
          color: var(--croot-text-muted);
          margin-bottom: 12px;
        }
        ul {
          list-style: none;
          margin: 0;
          padding: 0;
        }
        li {
          margin: 0;
        }
      `}</style>

      <style jsx global>{`
        .toc-link {
          display: block;
          padding: 4px 0;
          font-size: 0.8rem;
          color: var(--croot-text-muted);
          text-decoration: none;
          transition: color var(--croot-dur-fast) var(--croot-ease);
          line-height: 1.4;
        }
        .toc-link:hover {
          color: var(--croot-text);
        }
        .toc-link.active {
          color: var(--croot-accent-orange);
        }
      `}</style>
    </aside>
  )
}
