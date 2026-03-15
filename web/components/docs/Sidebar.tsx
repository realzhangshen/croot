"use client";

import { useState } from "react";
import { ChevronRight } from "lucide-react";
import Link from "next/link";
import { usePathname } from "next/navigation";
import { sidebarSections } from "@/lib/sidebar";

export function Sidebar() {
  const pathname = usePathname();
  const [expanded, setExpanded] = useState<Record<string, boolean>>(() =>
    Object.fromEntries(sidebarSections.map((s) => [s.label, true]))
  );

  const toggle = (label: string) =>
    setExpanded((prev) => ({ ...prev, [label]: !prev[label] }));

  return (
    <aside className="hidden min-[960px]:block w-64 shrink-0 sticky top-14 h-[calc(100vh-56px)] overflow-y-auto py-8 pr-6">
      <nav className="space-y-4">
        {sidebarSections.map((section, i) => {
          const isExpanded = expanded[section.label] ?? true;
          return (
            <div
              key={section.label}
              className={i > 0 ? "border-t border-border pt-4" : ""}
            >
              <button
                onClick={() => toggle(section.label)}
                className="flex items-center gap-1 w-full text-left text-[13px] font-medium text-text-muted hover:text-text transition-colors mb-1 px-2"
              >
                <ChevronRight
                  size={14}
                  className={`shrink-0 transition-transform duration-150 ${
                    isExpanded ? "rotate-90" : ""
                  }`}
                />
                {section.label}
              </button>
              {isExpanded && (
                <ul className="space-y-0.5">
                  {section.links.map((link) => {
                    const href = `/docs/${link.slug}`;
                    const isActive = pathname === href;
                    return (
                      <li key={link.slug}>
                        <Link
                          href={href}
                          className={`block px-2 py-1 text-sm rounded-md transition-colors ${
                            isActive
                              ? "text-text font-medium bg-bg-elevated"
                              : "text-text-secondary hover:text-text hover:bg-bg-elevated"
                          }`}
                        >
                          {link.title}
                        </Link>
                      </li>
                    );
                  })}
                </ul>
              )}
            </div>
          );
        })}
      </nav>
    </aside>
  );
}
