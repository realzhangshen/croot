import { Sidebar } from "@/components/docs/Sidebar";
import dynamic from "next/dynamic";

const MobileNav = dynamic(
  () => import("@/components/docs/MobileNav").then((m) => ({ default: m.MobileNav }))
);

export default function DocsLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <div className="flex-1 bg-bg-surface">
      <div className="flex items-center gap-2 border-b border-border bg-bg-surface px-4 py-3 min-[960px]:hidden">
        <MobileNav />
        <span className="text-sm font-medium text-text-muted">Documentation</span>
      </div>
      <div className="mx-auto flex max-w-[1520px]">
        <Sidebar />
        <div className="min-w-0 flex-1 px-4 sm:px-6 lg:px-10">{children}</div>
      </div>
    </div>
  );
}
