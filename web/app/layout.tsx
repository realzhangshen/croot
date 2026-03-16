import type { Metadata } from "next";
import { Instrument_Sans } from "next/font/google";
import { GeistMono } from "geist/font/mono";
import { ThemeProvider } from "@/components/shared/ThemeProvider";
import { Navbar } from "@/components/shared/Navbar";
import { Footer } from "@/components/shared/Footer";
import { AntiFlashScript } from "@/components/shared/AntiFlashScript";
import "@/tailwind.css";

const instrumentSans = Instrument_Sans({
  subsets: ["latin"],
  variable: "--font-instrument-sans",
  display: "swap",
});

export const metadata: Metadata = {
  title: "croot — Terminal File Explorer",
  description:
    "A modern terminal-based file tree explorer built with Rust & Ratatui. Git integration, file preview, fuzzy search, and mouse support.",
  icons: { icon: "/favicon.svg" },
};

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html lang="en" suppressHydrationWarning>
      <head>
        <AntiFlashScript />
      </head>
      <body
        className={`${instrumentSans.variable} ${GeistMono.variable} font-sans antialiased`}
      >
        <ThemeProvider>
          <div className="min-h-screen flex flex-col">
            <Navbar />
            <main className="flex-1 flex flex-col">{children}</main>
            <Footer />
          </div>
        </ThemeProvider>
      </body>
    </html>
  );
}
