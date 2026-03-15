import Link from "next/link";
import { sidebarSections } from "@/lib/sidebar";
import {
  BookOpen,
  Layers,
  Settings,
} from "lucide-react";

const sectionIcons: Record<string, React.ElementType> = {
  Guide: BookOpen,
  Features: Layers,
  Advanced: Settings,
};

export default function DocsIndex() {
  return (
    <div className="py-8 max-w-3xl">
      <h1 className="text-3xl font-bold text-text mb-2">Documentation</h1>
      <p className="text-text-secondary mb-8">
        Everything you need to know about croot.
      </p>
      <div className="space-y-8">
        {sidebarSections.map((section) => {
          const Icon = sectionIcons[section.label] || BookOpen;
          return (
            <div key={section.label}>
              <div className="flex items-center gap-2 mb-3">
                <Icon size={16} className="text-text-muted" />
                <h2 className="text-sm font-semibold uppercase tracking-wider text-text-muted">
                  {section.label}
                </h2>
              </div>
              <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
                {section.links.map((link) => (
                  <Link
                    key={link.slug}
                    href={`/docs/${link.slug}`}
                    className="group block p-4 rounded-lg border border-border hover:border-border-strong bg-bg-surface transition-colors"
                  >
                    <h3 className="text-sm font-medium text-text group-hover:underline">
                      {link.title}
                    </h3>
                  </Link>
                ))}
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
}
