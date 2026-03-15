import Link from "next/link";
import { ArrowRight } from "lucide-react";
import { InstallCommand } from "./InstallCommand";

export function Hero() {
  return (
    <section className="py-24 sm:py-32 lg:py-40">
      <div className="mx-auto max-w-7xl px-4 sm:px-6 text-center">
        <h1 className="animate-fade-up text-4xl sm:text-5xl lg:text-6xl font-bold tracking-tight text-text leading-tight">
          The VS Code sidebar
          <br />
          for your terminal
        </h1>
        <p className="animate-fade-up-delay-1 mt-6 text-lg sm:text-xl text-text-secondary max-w-2xl mx-auto leading-relaxed">
          A modern file tree explorer built with Rust. Git status, file preview,
          fuzzy search, and mouse support — all in your terminal.
        </p>
        <div className="animate-fade-up-delay-2 mt-10">
          <InstallCommand />
        </div>
        <div className="animate-fade-up-delay-3 mt-8 flex items-center justify-center gap-4 flex-wrap">
          <Link
            href="/docs/guide/getting-started"
            className="inline-flex items-center gap-2 bg-accent text-accent-fg px-5 py-2.5 rounded-lg text-sm font-medium hover:opacity-90 transition-opacity"
          >
            Get Started
            <ArrowRight size={16} />
          </Link>
          <a
            href="https://github.com/dxmq/croot"
            target="_blank"
            rel="noopener noreferrer"
            className="inline-flex items-center gap-2 border border-border px-5 py-2.5 rounded-lg text-sm font-medium text-text hover:bg-bg-elevated transition-colors"
          >
            GitHub
          </a>
        </div>
      </div>
    </section>
  );
}
