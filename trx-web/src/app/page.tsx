import { Hero } from "@/components/hero";
import { Features } from "@/components/features";
import { InstallSteps } from "@/components/install";
import { Footer } from "@/components/layout/Footer";
import TrxDemoWrapper from "@/components/TrxDemoWrapper";

const LEATHER_NOISE =
  "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='256' height='256'%3E%3Cfilter id='n'%3E%3CfeTurbulence type='fractalNoise' baseFrequency='0.75' numOctaves='4' stitchTiles='stitch'/%3E%3CfeColorMatrix type='saturate' values='0'/%3E%3C/filter%3E%3Crect width='256' height='256' filter='url(%23n)' opacity='0.15'/%3E%3C/svg%3E";

export default function Home() {
  return (
    <div
      style={{ background: "#111111", backgroundImage: `url("${LEATHER_NOISE}")`, backgroundSize: "256px 256px" }}
      className="min-h-screen overflow-x-hidden"
    >
      <Hero />
      <div className="container mx-auto px-4 relative z-10">
        <TrxDemoWrapper />
      </div>
      <Features />
      <InstallSteps />
      <Footer />
    </div>
  );
}
