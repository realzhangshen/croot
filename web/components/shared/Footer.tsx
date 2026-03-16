import Link from "next/link";

export function Footer() {
  return (
    <footer className="border-t border-border bg-bg-surface py-8">
      <div className="mx-auto flex max-w-7xl flex-col items-center justify-between gap-4 px-4 text-sm text-text-muted sm:flex-row sm:px-6">
        <p>Built with Rust & Ratatui</p>
        <div className="flex items-center gap-6">
          <a
            href="https://github.com/realzhangshen/croot"
            target="_blank"
            rel="noopener noreferrer"
            className="hover:text-text transition-colors"
          >
            GitHub
          </a>
          <Link href="/docs" className="hover:text-text transition-colors">
            Docs
          </Link>
          <a
            href="https://github.com/realzhangshen/croot/blob/main/CHANGELOG.md"
            target="_blank"
            rel="noopener noreferrer"
            className="hover:text-text transition-colors"
          >
            Changelog
          </a>
          <span>MIT License</span>
        </div>
      </div>
    </footer>
  );
}
