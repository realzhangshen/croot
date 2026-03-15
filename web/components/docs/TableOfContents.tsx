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
    <aside className="hidden w-[220px] shrink-0 min-[1200px]:block">
      <div className="sticky top-[72px] py-10 pl-8">
        <p className="mb-3 text-[12px] font-medium tracking-[0.02em] text-text-muted">
          On this page
        </p>
        <ul className="space-y-1">
          {entries.map((entry) => (
            <li key={entry.id}>
              <a
                href={`#${entry.id}`}
                className={`block text-[13px] leading-6 transition-colors ${
                  entry.level === 3 ? "pl-3" : ""
                } ${
                  activeId === entry.id
                    ? "font-medium text-text"
                    : "text-text-muted hover:text-text"
                }`}
              >
                {entry.text}
              </a>
            </li>
          ))}
        </ul>
      </div>
    </aside>
  );
}
