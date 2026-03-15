import { Sidebar } from "@/components/docs/Sidebar";
import { MobileNav } from "@/components/docs/MobileNav";

export default function DocsLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <div className="mx-auto max-w-7xl px-4 sm:px-6">
      <div className="flex items-center gap-2 min-[960px]:hidden py-3 border-b border-border">
        <MobileNav />
        <span className="text-sm text-text-muted">Documentation</span>
      </div>
      <div className="flex gap-0">
        <Sidebar />
        <div className="flex-1 min-w-0">{children}</div>
      </div>
    </div>
  );
}
