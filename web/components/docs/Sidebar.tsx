"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import { sidebarSections } from "@/lib/sidebar";

export function Sidebar() {
  const pathname = usePathname();

  return (
    <aside className="hidden min-[960px]:block w-64 shrink-0 sticky top-14 h-[calc(100vh-56px)] overflow-y-auto py-8 pr-6">
      <nav className="space-y-6">
        {sidebarSections.map((section) => (
          <div key={section.label}>
            <h4 className="text-[11px] font-semibold uppercase tracking-wider text-text-muted mb-2 px-3">
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
                      className={`block px-3 py-1.5 text-sm rounded-sm transition-colors ${
                        isActive
                          ? "text-text font-medium bg-bg-elevated border-l-2 border-accent"
                          : "text-text-secondary hover:text-text hover:bg-bg-elevated"
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
    </aside>
  );
}
