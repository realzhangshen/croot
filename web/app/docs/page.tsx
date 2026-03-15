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
    <div className="mx-auto max-w-[760px] py-10 lg:py-12">
      <p className="text-[12px] font-medium tracking-[0.02em] text-text-muted">
        Get Started
      </p>
      <h1 className="mt-2 text-[clamp(3rem,5.2vw,4rem)] font-semibold tracking-[-0.05em] text-text">
        croot Documentation
      </h1>
      <p className="mt-5 max-w-2xl text-[17px] leading-8 text-text-secondary">
        Explore installation, key workflows, and the features that make croot
        feel closer to an editor sidebar than a typical terminal file browser.
      </p>
      <div className="mt-10 space-y-10">
        {sidebarSections.map((section) => {
          const Icon = sectionIcons[section.label] || BookOpen;
          return (
            <div key={section.label}>
              <div className="mb-3 flex items-center gap-2">
                <Icon size={16} className="text-text-muted" />
                <h2 className="text-[12px] font-medium tracking-[0.02em] text-text-muted">
                  {section.label}
                </h2>
              </div>
              <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
                {section.links.map((link) => (
                  <Link
                    key={link.slug}
                    href={`/docs/${link.slug}`}
                    className="group block rounded-2xl border border-border bg-bg-elevated/45 p-4 transition-colors hover:border-border-strong hover:bg-bg-elevated"
                  >
                    <h3 className="text-[15px] font-medium tracking-[-0.02em] text-text">
                      {link.title}
                    </h3>
                    <p className="mt-1 text-sm text-text-muted group-hover:text-text-secondary">
                      Open this section
                    </p>
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
