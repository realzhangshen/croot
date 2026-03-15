"use client";

import { useState } from "react";
import { Menu, X } from "lucide-react";
import Link from "next/link";
import { usePathname } from "next/navigation";
import { sidebarSections } from "@/lib/sidebar";

export function MobileNav() {
  const [open, setOpen] = useState(false);
  const pathname = usePathname();

  return (
    <div className="min-[960px]:hidden">
      <button
        onClick={() => setOpen(true)}
        className="flex h-9 w-9 items-center justify-center rounded-xl border border-border bg-bg text-text-muted transition-colors hover:border-border-strong hover:text-text"
        aria-label="Open navigation"
      >
        <Menu size={20} />
      </button>

      {open && (
        <div className="fixed inset-0 z-50">
          <div
            className="absolute inset-0 bg-black/40"
            onClick={() => setOpen(false)}
          />
          <div className="absolute bottom-0 left-0 top-0 w-72 overflow-y-auto border-r border-border bg-bg-sidebar shadow-[var(--shadow-dialog)]">
            <div className="flex items-center justify-between p-4 border-b border-border">
              <span className="text-sm font-semibold tracking-[-0.02em] text-text">
                Documentation
              </span>
              <button
                onClick={() => setOpen(false)}
                className="flex h-8 w-8 items-center justify-center rounded-lg text-text-muted transition-colors hover:bg-bg-elevated hover:text-text"
                aria-label="Close navigation"
              >
                <X size={18} />
              </button>
            </div>
            <nav className="space-y-8 p-4">
              {sidebarSections.map((section) => (
                <div key={section.label}>
                  <h4 className="mb-2 px-2.5 text-[12px] font-medium tracking-[0.02em] text-text-muted">
                    {section.label}
                  </h4>
                  <ul className="space-y-0.5">
                    {section.links.map((link) => {
                      const href = `/docs/${link.slug}`;
                      const isActive = pathname === href;
                      return (
                        <li key={link.slug}>
                          <Link
                            href={href}
                            onClick={() => setOpen(false)}
                            aria-current={isActive ? "page" : undefined}
                            className={`block rounded-lg px-2.5 py-1.5 text-[15px] leading-6 transition-colors ${
                              isActive
                                ? "font-medium text-sidebar-active"
                                : "text-text-secondary hover:bg-white/55 hover:text-text dark:hover:bg-white/[0.04]"
                            }`}
                          >
                            {link.title}
                          </Link>
                        </li>
                      );
                    })}
                  </ul>
                </div>
              ))}
            </nav>
          </div>
        </div>
      )}
    </div>
  );
}
