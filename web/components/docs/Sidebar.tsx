"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import { sidebarSections } from "@/lib/sidebar";

export function Sidebar() {
  const pathname = usePathname();

  return (
    <aside className="hidden min-[960px]:block w-[220px] shrink-0 sticky top-14 h-[calc(100vh-56px)] overflow-y-auto py-6 pl-2 border-r border-border">
      <nav className="space-y-8">
        {sidebarSections.map((section) => (
          <div key={section.label}>
            <h4 className="text-xs font-normal text-text-muted tracking-[0.6px] mb-1">
              {section.label}
            </h4>
            <ul>
              {section.links.map((link) => {
                const href = `/docs/${link.slug}`;
                const isActive = pathname === href;
                return (
                  <li key={link.slug}>
                    <Link
                      href={href}
                      className={`block h-[29px] leading-[29px] text-base transition-colors rounded-sm hover:bg-bg-elevated/70 ${
                        isActive
                          ? "text-sidebar-active font-medium"
                          : "text-text"
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
