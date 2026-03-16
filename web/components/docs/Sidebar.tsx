"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import { sidebarSections } from "@/lib/sidebar";

export function Sidebar() {
  const pathname = usePathname();

  return (
    <aside className="sticky top-[52px] hidden h-[calc(100vh-52px)] w-[236px] shrink-0 overflow-y-auto border-r border-border bg-bg-sidebar min-[960px]:block">
      <nav className="space-y-8 px-4 py-6">
        {sidebarSections.map((section) => (
          <div key={section.label}>
            <h4 className="mb-2 px-2.5 text-[12px] font-semibold tracking-[0.02em] text-text-muted">
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
                      aria-current={isActive ? "page" : undefined}
                      className={`block rounded-lg px-2.5 py-1.5 text-[15px] leading-6 transition-colors ${
                        isActive
                          ? "font-semibold text-sidebar-active"
                          : "font-medium text-text-secondary hover:bg-white/55 hover:text-text dark:hover:bg-white/[0.04]"
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
