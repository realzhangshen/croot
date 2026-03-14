'use client'

import { useState } from 'react'

export default function CopyButton({ text }: { text: string }) {
  const [copied, setCopied] = useState(false)

  function handleCopy() {
    navigator.clipboard.writeText(text).then(() => {
      setCopied(true)
      setTimeout(() => setCopied(false), 2000)
    })
  }

  return (
    <button
      onClick={handleCopy}
      aria-label="Copy to clipboard"
      style={{
        background: 'var(--croot-bg-surface)',
        border: '1px solid var(--croot-border)',
        borderRadius: 6,
        padding: '4px 8px',
        cursor: 'pointer',
        color: copied ? 'var(--croot-text)' : 'var(--croot-text-muted)',
        fontSize: '0.75rem',
        fontFamily: 'var(--croot-font-mono)',
        transition: 'color var(--croot-dur-fast) var(--croot-ease), border-color var(--croot-dur-fast) var(--croot-ease)',
      }}
    >
      {copied ? 'Copied!' : 'Copy'}
    </button>
  )
}
