"use client";

import Link from "next/link";
import { Github, Search } from "lucide-react";
import { ThemeToggle } from "./ThemeToggle";
import { useState, useEffect, useCallback } from "react";
import { SearchDialog } from "../docs/SearchDialog";

export function Navbar() {
  const [searchOpen, setSearchOpen] = useState(false);

  const handleClose = useCallback(() => setSearchOpen(false), []);

  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key === "k") {
        e.preventDefault();
        setSearchOpen((o) => !o);
      }
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, []);

  return (
    <>
      <nav className="sticky top-0 z-50 h-14 border-b border-border bg-bg-surface/80 backdrop-blur-md">
        <div className="mx-auto max-w-7xl h-full px-4 sm:px-6 flex items-center justify-between">
          <div className="flex items-center gap-6">
            <Link
              href="/"
              className="text-text font-semibold text-lg tracking-tight"
            >
              croot
            </Link>
            <div className="hidden sm:flex items-center gap-4">
              <Link
                href="/docs"
                className="text-sm text-text-secondary hover:text-text transition-colors"
              >
                Docs
              </Link>
              <Link
                href="/docs/guide/getting-started"
                className="text-sm text-text-secondary hover:text-text transition-colors"
              >
                Quickstart
              </Link>
            </div>
          </div>
          <div className="flex items-center gap-1">
            <button
              onClick={() => setSearchOpen(true)}
              className="flex items-center gap-2 px-3 py-1.5 rounded-sm text-sm text-text-muted hover:text-text hover:bg-bg-elevated border border-border transition-colors"
            >
              <Search size={14} />
              <span className="hidden sm:inline">Search</span>
              <kbd className="hidden sm:inline text-xs text-text-muted bg-bg-elevated px-1.5 py-0.5 rounded border border-border ml-2">
                ⌘K
              </kbd>
            </button>
            <ThemeToggle />
            <a
              href="https://github.com/dxmq/croot"
              target="_blank"
              rel="noopener noreferrer"
              className="p-2 rounded-sm text-text-muted hover:text-text hover:bg-bg-elevated transition-colors"
              aria-label="GitHub"
            >
              <Github size={18} />
            </a>
          </div>
        </div>
      </nav>
      <SearchDialog open={searchOpen} onClose={handleClose} />
    </>
  );
}
