import { Hero } from "@/components/landing/Hero";
import { FeatureGrid } from "@/components/landing/FeatureGrid";
import { TerminalDemo } from "@/components/landing/TerminalDemo";

export default function HomePage() {
  return (
    <>
      <Hero />
      <TerminalDemo />
      <FeatureGrid />
    </>
  );
}
