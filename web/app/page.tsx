import { Hero } from "@/components/landing/Hero";
import { FeatureGrid } from "@/components/landing/FeatureGrid";

export default function HomePage() {
  return (
    <div className="pb-16">
      <Hero />
      <FeatureGrid />
    </div>
  );
}
