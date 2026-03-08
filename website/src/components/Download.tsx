import { useState, useEffect } from "react";
import { motion } from "framer-motion";
import { Download as DownloadIcon, Terminal, Apple, Monitor, ExternalLink, Loader2 } from "lucide-react";

type Platform = "linux" | "macos" | "windows";

function detectPlatform(): Platform {
  const ua = navigator.userAgent.toLowerCase();
  if (ua.includes("mac")) return "macos";
  if (ua.includes("win")) return "windows";
  return "linux";
}

function formatBytes(bytes: number): string {
  const mb = bytes / (1024 * 1024);
  return `${mb.toFixed(1)} MB`;
}

const platforms: { id: Platform; label: string; icon: typeof Monitor }[] = [
  { id: "linux", label: "Linux", icon: Terminal },
  { id: "macos", label: "macOS", icon: Apple },
  { id: "windows", label: "Windows", icon: Monitor },
];

const REPO = "dasunNimantha/bolt";

const ASSET_PATTERNS: Record<Platform, RegExp> = {
  linux: /^bolt-linux-x86_64$/,
  macos: /^bolt-macos-aarch64$/,
  windows: /^bolt-windows-x86_64\.exe$/,
};

interface Asset {
  url: string;
  filename: string;
  size: string;
}

interface ReleaseInfo {
  tag: string;
  name: string;
  assets: Record<Platform, Asset>;
}

export function Download() {
  const [platform, setPlatform] = useState<Platform>("linux");
  const [release, setRelease] = useState<ReleaseInfo | null>(null);

  useEffect(() => {
    setPlatform(detectPlatform());

    fetch(`https://api.github.com/repos/${REPO}/releases`)
      .then((r) => r.json())
      .then((releases: { tag_name: string; name: string; assets: { name: string; browser_download_url: string; size: number }[] }[]) => {
        const latest = releases[0];
        if (!latest) return;

        const assets = {} as Record<Platform, Asset>;
        for (const [plat, pattern] of Object.entries(ASSET_PATTERNS) as [Platform, RegExp][]) {
          const match = latest.assets.find((a) => pattern.test(a.name));
          if (match) {
            assets[plat] = {
              url: match.browser_download_url,
              filename: match.name,
              size: formatBytes(match.size),
            };
          }
        }

        setRelease({
          tag: latest.tag_name,
          name: latest.name || latest.tag_name,
          assets,
        });
      })
      .catch(() => {});
  }, []);

  const dl = release?.assets[platform];

  return (
    <section id="download" className="py-16 md:py-24 relative">
      <div className="absolute inset-0 -z-10">
        <div className="absolute top-0 left-0 right-0 h-px bg-gradient-to-r from-transparent via-white/[0.06] to-transparent" />
        <div className="absolute bottom-[20%] left-1/2 -translate-x-1/2 w-[700px] h-[500px] rounded-full bg-bolt/[0.06] blur-[150px] animate-pulse-glow" />
      </div>

      <div className="max-w-3xl mx-auto px-6">
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true, margin: "-100px" }}
          transition={{ duration: 0.6 }}
          className="text-center"
        >
          <h2 className="text-3xl md:text-5xl font-bold tracking-tight mb-4">
            Get Bolt
          </h2>
          <p className="text-text-secondary text-lg mb-10">
            Free, open source, no account required.
          </p>

          {/* Platform tabs */}
          <div className="inline-flex items-center gap-1 p-1.5 rounded-2xl glass border border-white/[0.06] mb-10">
            {platforms.map((p) => (
              <button
                key={p.id}
                onClick={() => setPlatform(p.id)}
                className={`flex items-center gap-2 px-5 py-2.5 rounded-xl text-sm font-medium transition-all duration-300 ${
                  platform === p.id
                    ? "bg-bolt text-black shadow-lg shadow-bolt/20"
                    : "text-text-secondary hover:text-text-primary hover:bg-white/[0.04]"
                }`}
              >
                <p.icon className="w-4 h-4" />
                {p.label}
              </button>
            ))}
          </div>

          {/* Download button */}
          <div>
            {!release ? (
              <div className="flex items-center justify-center gap-2 text-text-muted text-sm py-4">
                <Loader2 className="w-4 h-4 animate-spin" />
                Loading latest release…
              </div>
            ) : dl ? (
              <>
                <motion.a
                  href={dl.url}
                  whileHover={{ scale: 1.03 }}
                  whileTap={{ scale: 0.98 }}
                  className="group relative inline-flex items-center gap-3 px-10 py-4.5 bg-bolt text-black font-semibold text-lg rounded-2xl shadow-xl shadow-bolt/25 transition-shadow hover:shadow-2xl hover:shadow-bolt/35"
                >
                  <DownloadIcon className="w-5 h-5 transition-transform group-hover:-translate-y-0.5" />
                  Download for {platforms.find((p) => p.id === platform)?.label}
                  <span className="absolute inset-0 rounded-2xl bg-white/0 group-hover:bg-white/10 transition-colors" />
                </motion.a>
                <p className="text-xs text-text-muted mt-4 font-mono">
                  {dl.filename} &middot; {dl.size} &middot; {release.name}
                </p>
              </>
            ) : (
              <p className="text-sm text-text-muted">
                No binary available for this platform yet.
              </p>
            )}
            <a
              href={`https://github.com/${REPO}/releases`}
              target="_blank"
              rel="noopener noreferrer"
              className="inline-flex items-center gap-1.5 text-xs text-text-muted hover:text-bolt mt-2 transition-colors"
            >
              All releases <ExternalLink className="w-3 h-3" />
            </a>
          </div>

          {/* Build from source */}
          <div className="mt-14 p-6 rounded-2xl glass-card border border-white/[0.05] text-left noise relative overflow-hidden">
            <h3 className="relative z-10 text-sm font-semibold mb-4 text-text-secondary">
              Or build from source
            </h3>
            <div className="relative z-10 bg-black/30 rounded-xl p-5 font-mono text-sm text-text-secondary space-y-1 overflow-x-auto border border-white/[0.04]">
              <div>
                <span className="text-bolt/60 select-none">$ </span>
                <span className="text-text-primary">
                  git clone https://github.com/{REPO}.git
                </span>
              </div>
              <div>
                <span className="text-bolt/60 select-none">$ </span>
                <span className="text-text-primary">cd bolt</span>
              </div>
              <div>
                <span className="text-bolt/60 select-none">$ </span>
                <span className="text-text-primary">
                  cargo build --workspace --release
                </span>
              </div>
            </div>
          </div>
        </motion.div>
      </div>
    </section>
  );
}
