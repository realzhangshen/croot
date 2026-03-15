'use client'

import { useState, useEffect, useCallback, useRef } from 'react'

interface SearchEntry {
  slug: string
  title: string
  headings: string[]
  content: string
}

export default function SearchDialog({
  open,
  onClose,
}: {
  open: boolean
  onClose: () => void
}) {
  const [query, setQuery] = useState('')
  const [data, setData] = useState<SearchEntry[]>([])
  const [selectedIndex, setSelectedIndex] = useState(0)
  const inputRef = useRef<HTMLInputElement>(null)

  useEffect(() => {
    if (open && data.length === 0) {
      fetch('/search-data.json')
        .then((r) => r.json())
        .then(setData)
        .catch(() => {})
    }
    if (open) {
      setQuery('')
      setSelectedIndex(0)
      setTimeout(() => inputRef.current?.focus(), 50)
    }
  }, [open])

  useEffect(() => {
    function handleKeyDown(e: KeyboardEvent) {
      if ((e.metaKey || e.ctrlKey) && e.key === 'k') {
        e.preventDefault()
        if (open) onClose()
        else {
          // parent component handles opening
        }
      }
      if (e.key === 'Escape' && open) {
        e.preventDefault()
        onClose()
      }
    }
    window.addEventListener('keydown', handleKeyDown)
    return () => window.removeEventListener('keydown', handleKeyDown)
  }, [open, onClose])

  const results = query.trim()
    ? data.filter((entry) => {
        const q = query.toLowerCase()
        return (
          entry.title.toLowerCase().includes(q) ||
          entry.headings.some((h) => h.toLowerCase().includes(q)) ||
          entry.content.toLowerCase().includes(q)
        )
      })
    : []

  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      if (e.key === 'ArrowDown') {
        e.preventDefault()
        setSelectedIndex((i) => Math.min(i + 1, results.length - 1))
      } else if (e.key === 'ArrowUp') {
        e.preventDefault()
        setSelectedIndex((i) => Math.max(i - 1, 0))
      } else if (e.key === 'Enter' && results[selectedIndex]) {
        e.preventDefault()
        window.location.href = `/docs/${results[selectedIndex].slug}/`
        onClose()
      }
    },
    [results, selectedIndex, onClose]
  )

  useEffect(() => {
    setSelectedIndex(0)
  }, [query])

  if (!open) return null

  return (
    <div className="search-overlay" onClick={onClose}>
      <div className="search-dialog" onClick={(e) => e.stopPropagation()}>
        <div className="search-input-wrapper">
          <svg width={18} height={18} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={2} strokeLinecap="round" strokeLinejoin="round" style={{ color: 'var(--croot-text-muted)', flexShrink: 0 }}>
            <circle cx="11" cy="11" r="8" />
            <line x1="21" y1="21" x2="16.65" y2="16.65" />
          </svg>
          <input
            ref={inputRef}
            type="text"
            placeholder="Search documentation..."
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            onKeyDown={handleKeyDown}
            className="search-input"
          />
          <kbd className="search-kbd">ESC</kbd>
        </div>

        {query.trim() && (
          <div className="search-results">
            {results.length === 0 ? (
              <div className="search-empty">No results found.</div>
            ) : (
              results.map((entry, i) => (
                <a
                  key={entry.slug}
                  href={`/docs/${entry.slug}/`}
                  className={`search-result ${i === selectedIndex ? 'selected' : ''}`}
                  onClick={onClose}
                >
                  <div className="search-result-title">{entry.title}</div>
                  <div className="search-result-path">{entry.slug}</div>
                </a>
              ))
            )}
          </div>
        )}
      </div>

      <style jsx>{`
        .search-overlay {
          position: fixed;
          inset: 0;
          z-index: 100;
          background: rgba(0, 0, 0, 0.3);
          display: flex;
          align-items: flex-start;
          justify-content: center;
          padding-top: 15vh;
        }
        .search-dialog {
          width: 90%;
          max-width: 560px;
          background: var(--croot-bg-surface);
          border: 1px solid var(--croot-border);
          border-radius: var(--croot-radius-sm);
          box-shadow: var(--croot-shadow-elevated);
          overflow: hidden;
        }
        .search-input-wrapper {
          display: flex;
          align-items: center;
          gap: 10px;
          padding: 12px 16px;
          border-bottom: 1px solid var(--croot-border);
        }
        .search-input {
          flex: 1;
          border: none;
          background: transparent;
          font-size: 1rem;
          color: var(--croot-text);
          outline: none;
          font-family: var(--croot-font-sans);
        }
        .search-input::placeholder {
          color: var(--croot-text-muted);
        }
        .search-kbd {
          font-family: var(--croot-font-sans);
          font-size: 0.7rem;
          padding: 2px 6px;
          border: 1px solid var(--croot-border);
          border-radius: 4px;
          color: var(--croot-text-muted);
          background: var(--croot-bg);
          flex-shrink: 0;
        }
        .search-results {
          max-height: 400px;
          overflow-y: auto;
          padding: 8px;
        }
        .search-empty {
          padding: 24px 16px;
          text-align: center;
          color: var(--croot-text-muted);
          font-size: 0.9rem;
        }
        .search-result {
          display: block;
          padding: 10px 12px;
          border-radius: 6px;
          text-decoration: none;
          color: var(--croot-text);
          transition: background var(--croot-dur-fast) var(--croot-ease);
        }
        .search-result:hover,
        .search-result.selected {
          background: var(--croot-bg-hover);
        }
        .search-result-title {
          font-weight: 600;
          font-size: 0.9rem;
        }
        .search-result-path {
          font-size: 0.8rem;
          color: var(--croot-text-muted);
          font-family: var(--croot-font-mono);
          margin-top: 2px;
        }
      `}</style>
    </div>
  )
}
