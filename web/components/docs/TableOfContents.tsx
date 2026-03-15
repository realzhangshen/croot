"use client";

import { useEffect, useState } from "react";
import type { TocEntry } from "@/lib/docs";

export function TableOfContents({ entries }: { entries: TocEntry[] }) {
  const [activeId, setActiveId] = useState("");

  useEffect(() => {
    const observer = new IntersectionObserver(
      (obs) => {
        for (const entry of obs) {
          if (entry.isIntersecting) {
            setActiveId(entry.target.id);
          }
        }
      },
      { rootMargin: "-80px 0px -60% 0px", threshold: 0 }
    );

    for (const entry of entries) {
      const el = document.getElementById(entry.id);
      if (el) observer.observe(el);
    }

    return () => observer.disconnect();
  }, [entries]);

  if (entries.length === 0) return null;

  return (
    <aside className="hidden min-[1200px]:block w-[200px] shrink-0 sticky top-14 h-[calc(100vh-56px)] overflow-y-auto py-8 pl-6">
      <p className="text-[11px] font-semibold uppercase tracking-wider text-text-muted mb-3">
        On this page
      </p>
      <ul className="space-y-1">
        {entries.map((entry) => (
          <li key={entry.id}>
            <a
              href={`#${entry.id}`}
              className={`block text-xs leading-relaxed transition-colors ${
                entry.level === 3 ? "pl-3" : ""
              } ${
                activeId === entry.id
                  ? "text-text font-medium"
                  : "text-text-muted hover:text-text"
              }`}
            >
              {entry.text}
            </a>
          </li>
        ))}
      </ul>
    </aside>
  );
}
