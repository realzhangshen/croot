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
    <div className="inline-flex items-center gap-3 bg-bg-elevated border border-border rounded-lg px-4 py-2.5 font-mono text-sm">
      <span className="text-text-muted select-none">$</span>
      <code className="text-text">{command}</code>
      <button
        onClick={copy}
        className="text-text-muted hover:text-text transition-colors"
        aria-label="Copy install command"
      >
        {copied ? <Check size={14} /> : <Copy size={14} />}
      </button>
    </div>
  );
}
