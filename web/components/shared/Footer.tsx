import Link from "next/link";

export function Footer() {
  return (
    <footer className="border-t border-border py-8 mt-16">
      <div className="mx-auto max-w-7xl px-4 sm:px-6 flex flex-col sm:flex-row items-center justify-between gap-4 text-sm text-text-muted">
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
