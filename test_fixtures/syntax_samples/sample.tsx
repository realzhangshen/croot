/**
 * TSX sample file exercising JSX elements, components, hooks,
 * generics, and all semantic token types for React/TSX highlighting.
 */

import React, {
  useState,
  useEffect,
  useCallback,
  useMemo,
  useRef,
  type FC,
  type ReactNode,
  type CSSProperties,
} from "react";

// ── Types & Interfaces ──────────────────────────────────────────────

interface Theme {
  primary: string;
  secondary: string;
  background: string;
  text: string;
  fontFamily: string;
  borderRadius: number;
}

interface Column<T> {
  key: keyof T & string;
  label: string;
  width?: number | string;
  render?: (value: T[keyof T], row: T) => ReactNode;
  sortable?: boolean;
}

type SortDirection = "asc" | "desc" | null;
type FilterFn<T> = (item: T, query: string) => boolean;

interface DataTableProps<T extends Record<string, unknown>> {
  data: T[];
  columns: Column<T>[];
  title?: string;
  loading?: boolean;
  pageSize?: number;
  onRowClick?: (row: T, index: number) => void;
  filterFn?: FilterFn<T>;
  emptyMessage?: string;
  className?: string;
  style?: CSSProperties;
}

// ── Constants ────────────────────────────────────────────────────────

const DEFAULT_PAGE_SIZE = 20;
const DEBOUNCE_MS = 300;

const defaultTheme: Theme = {
  primary: "#6366f1",
  secondary: "#8b5cf6",
  background: "#ffffff",
  text: "#1e293b",
  fontFamily: '"Inter", system-ui, sans-serif',
  borderRadius: 8,
};

// ── Custom Hooks ─────────────────────────────────────────────────────

function useDebounce<T>(value: T, delay: number = DEBOUNCE_MS): T {
  const [debounced, setDebounced] = useState(value);

  useEffect(() => {
    const timer = setTimeout(() => setDebounced(value), delay);
    return () => clearTimeout(timer);
  }, [value, delay]);

  return debounced;
}

function usePagination(totalItems: number, pageSize: number) {
  const [page, setPage] = useState(1);
  const totalPages = Math.ceil(totalItems / pageSize);

  const goTo = useCallback(
    (p: number) => setPage(Math.max(1, Math.min(p, totalPages))),
    [totalPages],
  );

  return {
    page,
    totalPages,
    goTo,
    next: () => goTo(page + 1),
    prev: () => goTo(page - 1),
    hasNext: page < totalPages,
    hasPrev: page > 1,
    startIndex: (page - 1) * pageSize,
    endIndex: Math.min(page * pageSize, totalItems),
  };
}

// ── Utility Components ───────────────────────────────────────────────

const Spinner: FC<{ size?: number; color?: string }> = ({
  size = 24,
  color = defaultTheme.primary,
}) => (
  <svg
    width={size}
    height={size}
    viewBox="0 0 24 24"
    fill="none"
    xmlns="http://www.w3.org/2000/svg"
    aria-label="Loading"
    role="status"
  >
    <circle
      cx={12}
      cy={12}
      r={10}
      stroke={color}
      strokeWidth={3}
      strokeDasharray="31.4 31.4"
      strokeLinecap="round"
    >
      <animateTransform
        attributeName="transform"
        type="rotate"
        values="0 12 12;360 12 12"
        dur="1s"
        repeatCount="indefinite"
      />
    </circle>
  </svg>
);

const Badge: FC<{
  children: ReactNode;
  variant?: "info" | "success" | "warning" | "error";
}> = ({ children, variant = "info" }) => {
  const colors: Record<string, { bg: string; fg: string }> = {
    info: { bg: "#dbeafe", fg: "#1e40af" },
    success: { bg: "#dcfce7", fg: "#166534" },
    warning: { bg: "#fef9c3", fg: "#854d0e" },
    error: { bg: "#fee2e2", fg: "#991b1b" },
  };

  const { bg, fg } = colors[variant] ?? colors.info;

  return (
    <span
      style={{
        backgroundColor: bg,
        color: fg,
        padding: "2px 8px",
        borderRadius: 4,
        fontSize: 12,
        fontWeight: 600,
      }}
    >
      {children}
    </span>
  );
};

// ── Main Component ───────────────────────────────────────────────────

function DataTable<T extends Record<string, unknown>>({
  data,
  columns,
  title,
  loading = false,
  pageSize = DEFAULT_PAGE_SIZE,
  onRowClick,
  filterFn,
  emptyMessage = "No data available",
  className,
  style,
}: DataTableProps<T>): JSX.Element {
  const [query, setQuery] = useState("");
  const [sortKey, setSortKey] = useState<string | null>(null);
  const [sortDir, setSortDir] = useState<SortDirection>(null);
  const tableRef = useRef<HTMLTableElement>(null);

  const debouncedQuery = useDebounce(query);

  // Filter
  const filtered = useMemo(() => {
    if (!debouncedQuery || !filterFn) return data;
    return data.filter((item) => filterFn(item, debouncedQuery));
  }, [data, debouncedQuery, filterFn]);

  // Sort
  const sorted = useMemo(() => {
    if (!sortKey || !sortDir) return filtered;
    return [...filtered].sort((a, b) => {
      const av = a[sortKey];
      const bv = b[sortKey];
      if (av === bv) return 0;
      const cmp = av != null && bv != null && av < bv ? -1 : 1;
      return sortDir === "asc" ? cmp : -cmp;
    });
  }, [filtered, sortKey, sortDir]);

  // Pagination
  const { page, totalPages, next, prev, hasNext, hasPrev, startIndex, endIndex } =
    usePagination(sorted.length, pageSize);
  const pageData = sorted.slice(startIndex, endIndex);

  const handleSort = useCallback(
    (key: string) => {
      if (sortKey === key) {
        setSortDir((d) => (d === "asc" ? "desc" : d === "desc" ? null : "asc"));
        if (sortDir === "desc") setSortKey(null);
      } else {
        setSortKey(key);
        setSortDir("asc");
      }
    },
    [sortKey, sortDir],
  );

  const sortIndicator = (key: string): string => {
    if (sortKey !== key) return "";
    return sortDir === "asc" ? " \u2191" : " \u2193";
  };

  if (loading) {
    return (
      <div style={{ display: "flex", justifyContent: "center", padding: 40 }}>
        <Spinner size={32} />
      </div>
    );
  }

  return (
    <div className={className} style={{ fontFamily: defaultTheme.fontFamily, ...style }}>
      {/* Header */}
      {(title || filterFn) && (
        <div style={{ display: "flex", justifyContent: "space-between", marginBottom: 12 }}>
          {title && <h2 style={{ margin: 0, color: defaultTheme.text }}>{title}</h2>}
          {filterFn && (
            <input
              type="search"
              placeholder="Search\u2026"
              value={query}
              onChange={(e) => setQuery(e.target.value)}
              style={{
                padding: "6px 12px",
                border: "1px solid #e2e8f0",
                borderRadius: defaultTheme.borderRadius,
                outline: "none",
              }}
              aria-label="Filter table"
            />
          )}
        </div>
      )}

      {/* Table */}
      <table
        ref={tableRef}
        style={{
          width: "100%",
          borderCollapse: "collapse",
          borderRadius: defaultTheme.borderRadius,
          overflow: "hidden",
        }}
      >
        <thead>
          <tr style={{ backgroundColor: "#f8fafc" }}>
            {columns.map((col) => (
              <th
                key={col.key}
                onClick={() => col.sortable && handleSort(col.key)}
                style={{
                  padding: "10px 14px",
                  textAlign: "left",
                  cursor: col.sortable ? "pointer" : "default",
                  userSelect: "none",
                  width: col.width,
                  borderBottom: `2px solid ${defaultTheme.primary}`,
                }}
              >
                {col.label}
                {col.sortable && sortIndicator(col.key)}
              </th>
            ))}
          </tr>
        </thead>
        <tbody>
          {pageData.length === 0 ? (
            <tr>
              <td
                colSpan={columns.length}
                style={{ textAlign: "center", padding: 24, color: "#94a3b8" }}
              >
                {emptyMessage}
              </td>
            </tr>
          ) : (
            pageData.map((row, idx) => (
              <tr
                key={String(row["id"] ?? idx)}
                onClick={() => onRowClick?.(row, startIndex + idx)}
                style={{
                  cursor: onRowClick ? "pointer" : "default",
                  borderBottom: "1px solid #f1f5f9",
                }}
              >
                {columns.map((col) => (
                  <td key={col.key} style={{ padding: "8px 14px" }}>
                    {col.render
                      ? col.render(row[col.key], row)
                      : String(row[col.key] ?? "")}
                  </td>
                ))}
              </tr>
            ))
          )}
        </tbody>
      </table>

      {/* Pagination */}
      {totalPages > 1 && (
        <div style={{ display: "flex", justifyContent: "center", gap: 8, marginTop: 12 }}>
          <button onClick={prev} disabled={!hasPrev}>
            &laquo; Prev
          </button>
          <span>
            Page {page} of {totalPages}
          </span>
          <button onClick={next} disabled={!hasNext}>
            Next &raquo;
          </button>
        </div>
      )}
    </div>
  );
}

// ── Usage Example ────────────────────────────────────────────────────

interface User {
  id: number;
  name: string;
  email: string;
  role: "admin" | "user" | "viewer";
  active: boolean;
}

const App: FC = () => {
  const [users, setUsers] = useState<User[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    const controller = new AbortController();

    (async () => {
      try {
        const res = await fetch("/api/users", { signal: controller.signal });
        const data: User[] = await res.json();
        setUsers(data);
      } catch (err) {
        if (err instanceof DOMException && err.name === "AbortError") return;
        console.error("Failed to load users:", err);
      } finally {
        setLoading(false);
      }
    })();

    return () => controller.abort();
  }, []);

  const columns: Column<User>[] = [
    { key: "id", label: "#", width: 50, sortable: true },
    { key: "name", label: "Name", sortable: true },
    { key: "email", label: "Email" },
    {
      key: "role",
      label: "Role",
      sortable: true,
      render: (val) => <Badge variant={val === "admin" ? "warning" : "info"}>{String(val)}</Badge>,
    },
    {
      key: "active",
      label: "Status",
      render: (val) => (
        <Badge variant={val ? "success" : "error"}>
          {val ? "Active" : "Inactive"}
        </Badge>
      ),
    },
  ];

  return (
    <DataTable<User>
      data={users}
      columns={columns}
      title="User Management"
      loading={loading}
      pageSize={15}
      onRowClick={(user) => console.log("Selected:", user.name)}
      filterFn={(user, q) =>
        user.name.toLowerCase().includes(q.toLowerCase()) ||
        user.email.toLowerCase().includes(q.toLowerCase())
      }
    />
  );
};

export default App;
export { DataTable, Badge, Spinner, useDebounce, usePagination };
export type { DataTableProps, Column, Theme };
