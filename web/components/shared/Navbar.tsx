"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import { Github, Search } from "lucide-react";
import { ThemeToggle } from "./ThemeToggle";
import { useState, useEffect, useCallback } from "react";
import dynamic from "next/dynamic";

const SearchDialog = dynamic(
  () => import("../docs/SearchDialog").then((m) => ({ default: m.SearchDialog })),
  { ssr: false }
);

export function Navbar() {
  const pathname = usePathname();
  const isDocsActive = pathname.startsWith("/docs");

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
      <nav className="sticky top-0 z-50 border-b border-border bg-bg-navbar backdrop-blur-xl">
        <div className="mx-auto flex h-[52px] max-w-[1480px] items-center justify-between px-4 sm:px-6 lg:px-8">
          <div className="flex items-center gap-6 sm:gap-8">
            <Link
              href="/"
              className="flex items-center gap-2 text-[1.02rem] font-semibold tracking-[-0.04em] text-text"
            >
              <span className="inline-flex h-5 w-5 items-center justify-center rounded-[6px] border border-border bg-bg-elevated text-[11px] font-semibold text-text shadow-[var(--shadow-soft)]">
                c
              </span>
              <span>croot</span>
            </Link>
            <div className="hidden h-full items-center gap-1 sm:flex">
              <Link
                href="/docs/guide/getting-started"
                className={`relative flex h-full items-center px-3 text-[14px] font-medium transition-colors ${
                  isDocsActive
                    ? "text-sidebar-active"
                    : "text-text-secondary hover:text-text"
                }`}
              >
                Docs
                {isDocsActive && (
                  <span className="absolute inset-x-0 bottom-0 h-0.5 bg-sidebar-active" />
                )}
              </Link>
            </div>
          </div>
          <div className="flex items-center gap-1.5">
            <button
              onClick={() => setSearchOpen(true)}
              className="hidden h-9 min-w-[182px] items-center justify-between gap-3 rounded-xl border border-border bg-bg px-3.5 text-[13px] text-text-muted transition-colors hover:border-border-strong hover:text-text sm:flex"
            >
              <span className="flex items-center gap-2">
                <Search size={14} />
                <span>Search docs...</span>
              </span>
              <kbd className="rounded-md border border-border bg-bg-elevated px-1.5 py-0.5 text-[10px] text-text-muted">
                ⌘K
              </kbd>
            </button>
            <button
              onClick={() => setSearchOpen(true)}
              className="flex h-9 w-9 items-center justify-center rounded-xl border border-border bg-bg text-text-muted transition-colors hover:border-border-strong hover:text-text sm:hidden"
              aria-label="Search"
            >
              <Search size={15} />
            </button>
            <ThemeToggle />
            <a
              href="https://github.com/realzhangshen/croot"
              target="_blank"
              rel="noopener noreferrer"
              className="flex h-9 w-9 items-center justify-center rounded-xl text-text-muted transition-colors hover:bg-bg-elevated hover:text-text"
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
