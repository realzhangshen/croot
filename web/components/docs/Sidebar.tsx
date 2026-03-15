"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import { sidebarSections } from "@/lib/sidebar";

export function Sidebar() {
  const pathname = usePathname();

  return (
    <aside className="hidden min-[960px]:block w-64 shrink-0 sticky top-14 h-[calc(100vh-56px)] overflow-y-auto py-6 pr-6">
      <nav className="space-y-5">
        {sidebarSections.map((section) => (
          <div key={section.label}>
            <h4 className="text-[13px] font-normal text-text-muted mb-1.5 px-2">
              {section.label}
            </h4>
            <ul className="space-y-px">
              {section.links.map((link) => {
                const href = `/docs/${link.slug}`;
                const isActive = pathname === href;
                return (
                  <li key={link.slug}>
                    <Link
                      href={href}
                      className={`block px-2 py-1 text-[14px] transition-colors ${
                        isActive
                          ? "text-accent font-medium"
                          : "text-text hover:text-accent"
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
