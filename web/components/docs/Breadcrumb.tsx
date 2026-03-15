import Link from "next/link";
import { ChevronRight } from "lucide-react";

export function Breadcrumb({ slug }: { slug: string }) {
  const parts = slug.split("/");

  return (
    <nav className="flex items-center gap-1 text-sm text-text-muted mb-6">
      <Link href="/docs" className="hover:text-text transition-colors">
        Docs
      </Link>
      {parts.map((part, i) => (
        <span key={part} className="flex items-center gap-1">
          <ChevronRight size={14} />
          {i === parts.length - 1 ? (
            <span className="text-text capitalize">
              {part.replace(/-/g, " ")}
            </span>
          ) : (
            <span className="capitalize">{part}</span>
          )}
        </span>
      ))}
    </nav>
  );
}
