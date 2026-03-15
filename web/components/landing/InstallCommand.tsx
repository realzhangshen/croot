"use client";

import { Check, Copy } from "lucide-react";
import { useState } from "react";

export function InstallCommand() {
  const [copied, setCopied] = useState(false);
  const command = "brew install realzhangshen/croot/croot";

  const copy = async () => {
    await navigator.clipboard.writeText(command);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <div className="inline-flex max-w-full items-center gap-3 rounded-2xl border border-border bg-bg-surface px-4 py-3 text-sm shadow-[var(--shadow-soft)]">
      <span className="select-none text-text-muted">$</span>
      <code className="truncate text-left text-text">{command}</code>
      <button
        onClick={copy}
        className="rounded-lg p-1 text-text-muted transition-colors hover:bg-bg-elevated hover:text-text"
        aria-label="Copy install command"
      >
        {copied ? <Check size={14} /> : <Copy size={14} />}
      </button>
    </div>
  );
}
