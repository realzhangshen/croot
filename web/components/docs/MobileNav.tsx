"use client";

import { useState } from "react";
import { Menu, X, ChevronRight } from "lucide-react";
import Link from "next/link";
import { usePathname } from "next/navigation";
import { sidebarSections } from "@/lib/sidebar";

export function MobileNav() {
  const [open, setOpen] = useState(false);
  const pathname = usePathname();
  const [expanded, setExpanded] = useState<Record<string, boolean>>(() =>
    Object.fromEntries(sidebarSections.map((s) => [s.label, true]))
  );

  const toggle = (label: string) =>
    setExpanded((prev) => ({ ...prev, [label]: !prev[label] }));

  return (
    <div className="min-[960px]:hidden">
      <button
        onClick={() => setOpen(true)}
        className="p-2 text-text-muted hover:text-text transition-colors"
        aria-label="Open navigation"
      >
        <Menu size={20} />
      </button>

      {open && (
        <div className="fixed inset-0 z-50">
          <div
            className="absolute inset-0 bg-black/40 backdrop-blur-sm"
            onClick={() => setOpen(false)}
          />
          <div className="absolute top-0 left-0 bottom-0 w-72 bg-bg-surface border-r border-border overflow-y-auto">
            <div className="flex items-center justify-between p-4 border-b border-border">
              <span className="font-semibold text-text">Navigation</span>
              <button
                onClick={() => setOpen(false)}
                className="p-1 text-text-muted hover:text-text"
                aria-label="Close navigation"
              >
                <X size={18} />
              </button>
            </div>
            <nav className="p-4 space-y-4">
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
                                onClick={() => setOpen(false)}
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
          </div>
        </div>
      )}
    </div>
  );
}
