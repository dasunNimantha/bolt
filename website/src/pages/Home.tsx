import { Navbar } from "../components/Navbar";
import { Hero } from "../components/Hero";
import { Features } from "../components/Features";
import { HowItWorks } from "../components/HowItWorks";
import { Download } from "../components/Download";
import { Footer } from "../components/Footer";
import { useSEO } from "../hooks/useSEO";

export function Home() {
  useSEO({
    title: "Bolt — Fast Multi-Threaded Download Manager",
    description:
      "Free, open-source download manager with multi-segment parallel downloads, pause & resume, scheduling, proxy support, and browser integration. Built with Rust for Linux, Windows, and macOS.",
    canonical: "https://boltdm.site/",
  });

  return (
    <div className="min-h-screen">
      <Navbar />
      <main>
        <Hero />
        <Features />
        <HowItWorks />
        <Download />
      </main>
      <Footer />
    </div>
  );
}
