import Link from "next/link";
import { ArrowRight } from "lucide-react";
import { InstallCommand } from "./InstallCommand";

export function Hero() {
  return (
    <section className="relative overflow-hidden py-24 sm:py-28 lg:py-32">
      <div className="absolute inset-x-0 top-0 -z-10 h-[460px] bg-[radial-gradient(circle_at_top,_rgba(255,255,255,0.86),_transparent_70%)]" />
      <div className="absolute left-1/2 top-10 -z-10 h-[340px] w-[720px] -translate-x-1/2 rounded-full bg-white/50 blur-3xl dark:bg-white/[0.05]" />
      <div className="mx-auto max-w-6xl px-4 text-center sm:px-6">
        <div className="mx-auto max-w-4xl">
          <div className="animate-fade-up inline-flex items-center gap-2 rounded-full border border-border bg-bg-surface px-4 py-1.5 text-[12px] font-medium tracking-[0.01em] text-text-muted shadow-[var(--shadow-soft)]">
            <span className="h-1.5 w-1.5 rounded-full bg-sidebar-active" />
            Terminal-native file navigation, Cursor-inspired UI
          </div>
          <h1 className="animate-fade-up mt-6 text-5xl font-semibold leading-[0.94] tracking-[-0.06em] text-text sm:text-6xl lg:text-7xl">
            The editor-like sidebar
            <br />
            your terminal was missing
          </h1>
          <p className="animate-fade-up-delay-1 mx-auto mt-6 max-w-3xl text-[18px] leading-8 text-text-secondary sm:text-[19px]">
            croot brings a clean docs-grade interface to the command line with
            fast file traversal, Git awareness, previews, and search that feel
            closer to a modern IDE than a traditional TUI.
          </p>
          <div className="animate-fade-up-delay-2 mt-10">
            <InstallCommand />
          </div>
          <div className="animate-fade-up-delay-3 mt-8 flex flex-wrap items-center justify-center gap-4">
            <Link
              href="/docs/guide/getting-started"
              className="inline-flex items-center gap-2 rounded-full bg-accent px-5 py-2.5 text-sm font-medium text-accent-fg transition-opacity hover:opacity-90"
            >
              Get Started
              <ArrowRight size={16} />
            </Link>
            <a
              href="https://github.com/realzhangshen/croot"
              target="_blank"
              rel="noopener noreferrer"
              className="inline-flex items-center gap-2 rounded-full border border-border bg-bg-surface px-5 py-2.5 text-sm font-medium text-text transition-colors hover:border-border-strong hover:bg-bg-elevated"
            >
              GitHub
            </a>
          </div>
        </div>
      </div>
    </section>
  );
}
