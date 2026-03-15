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
            <nav className="p-4 space-y-6">
              {sidebarSections.map((section) => (
                <div key={section.label}>
                  <h4 className="text-[11px] font-semibold uppercase tracking-wider text-text-muted mb-2">
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
                            className={`block px-3 py-1.5 text-sm rounded-sm transition-colors ${
                              isActive
                                ? "text-text font-medium bg-bg-elevated"
                                : "text-text-secondary hover:text-text"
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
