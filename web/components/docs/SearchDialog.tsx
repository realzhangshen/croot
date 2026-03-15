"use client";

import { useEffect, useState, useCallback, useRef } from "react";
import { useRouter } from "next/navigation";
import { Search, FileText } from "lucide-react";

interface SearchEntry {
  title: string;
  slug: string;
  section: string;
  content: string;
}

export function SearchDialog({
  open,
  onClose,
}: {
  open: boolean;
  onClose: () => void;
}) {
  const [query, setQuery] = useState("");
  const [results, setResults] = useState<SearchEntry[]>([]);
  const [data, setData] = useState<SearchEntry[]>([]);
  const [selected, setSelected] = useState(0);
  const inputRef = useRef<HTMLInputElement>(null);
  const timerRef = useRef<ReturnType<typeof setTimeout>>(undefined);
  const router = useRouter();

  // Lazy-load search data only when dialog opens
  const dataLoaded = useRef(false);
  useEffect(() => {
    if (!open || dataLoaded.current) return;
    dataLoaded.current = true;
    const basePath =
      process.env.NODE_ENV === "production" ? "/croot" : "";
    fetch(`${basePath}/search-data.json`)
      .then((r) => r.json())
      .then(setData)
      .catch(() => {});
  }, [open]);

  useEffect(() => {
    if (open) {
      setQuery("");
      setResults([]);
      setSelected(0);
      setTimeout(() => inputRef.current?.focus(), 50);
    }
  }, [open]);

  const search = useCallback(
    (q: string) => {
      setQuery(q);
      clearTimeout(timerRef.current);
      if (!q.trim()) {
        setResults([]);
        setSelected(0);
        return;
      }
      timerRef.current = setTimeout(() => {
        const lower = q.toLowerCase();
        const matched = data.filter(
          (entry) =>
            entry.title.toLowerCase().includes(lower) ||
            entry.content.toLowerCase().includes(lower)
        );
        setResults(matched.slice(0, 10));
        setSelected(0);
      }, 150);
    },
    [data]
  );

  const navigate = useCallback(
    (slug: string) => {
      router.push(`/docs/${slug}`);
      onClose();
    },
    [router, onClose]
  );

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "ArrowDown") {
      e.preventDefault();
      setSelected((s) => Math.min(s + 1, results.length - 1));
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      setSelected((s) => Math.max(s - 1, 0));
    } else if (e.key === "Enter" && results[selected]) {
      navigate(results[selected].slug);
    } else if (e.key === "Escape") {
      onClose();
    }
  };

  if (!open) return null;

  return (
    <div className="fixed inset-0 z-[100] search-backdrop" onClick={onClose}>
      <div className="mx-auto max-w-xl mt-[20vh] px-4" onClick={(e) => e.stopPropagation()}>
        <div className="bg-bg-surface border border-border rounded-lg shadow-2xl overflow-hidden">
          <div className="flex items-center gap-3 px-4 py-3 border-b border-border">
            <Search size={16} className="text-text-muted shrink-0" />
            <input
              ref={inputRef}
              type="text"
              value={query}
              onChange={(e) => search(e.target.value)}
              onKeyDown={handleKeyDown}
              placeholder="Search documentation..."
              className="flex-1 bg-transparent text-sm text-text placeholder:text-text-muted outline-none"
            />
            <kbd className="text-[10px] text-text-muted bg-bg-elevated px-1.5 py-0.5 rounded border border-border">
              ESC
            </kbd>
          </div>
          {results.length > 0 && (
            <ul className="max-h-80 overflow-y-auto py-2">
              {results.map((result, i) => (
                <li key={result.slug}>
                  <button
                    onClick={() => navigate(result.slug)}
                    className={`w-full text-left px-4 py-2.5 flex items-center gap-3 text-sm transition-colors ${
                      i === selected
                        ? "bg-bg-elevated text-text"
                        : "text-text-secondary hover:bg-bg-elevated"
                    }`}
                  >
                    <FileText size={14} className="shrink-0 text-text-muted" />
                    <div>
                      <p className="font-medium">{result.title}</p>
                      <p className="text-xs text-text-muted">
                        {result.section}
                      </p>
                    </div>
                  </button>
                </li>
              ))}
            </ul>
          )}
          {query && results.length === 0 && (
            <p className="px-4 py-8 text-sm text-text-muted text-center">
              No results for &ldquo;{query}&rdquo;
            </p>
          )}
        </div>
      </div>
    </div>
  );
}
