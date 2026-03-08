import { motion } from "framer-motion";
import { Download, Zap, Github } from "lucide-react";

export function Hero() {
  return (
    <section className="relative pt-28 pb-16 md:pt-40 md:pb-24 overflow-hidden">
      {/* Layered background */}
      <div className="absolute inset-0 -z-10">
        <div className="absolute inset-0 grid-bg" />
        <div className="absolute top-[15%] left-1/2 -translate-x-1/2 w-[900px] h-[600px] rounded-full bg-bolt/[0.07] blur-[150px] animate-pulse-glow" />
        <div className="absolute top-[10%] left-[20%] w-[400px] h-[400px] rounded-full bg-purple-500/[0.03] blur-[120px]" />
        <div className="absolute top-[30%] right-[15%] w-[300px] h-[300px] rounded-full bg-blue-500/[0.03] blur-[100px]" />
        <div className="absolute top-0 left-0 right-0 h-px bg-gradient-to-r from-transparent via-bolt/20 to-transparent" />
      </div>

      <div className="max-w-6xl mx-auto px-6">
        <motion.div
          initial={{ opacity: 0, y: 30 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.7, ease: [0.22, 1, 0.36, 1] }}
          className="text-center max-w-3xl mx-auto"
        >
          <motion.div
            initial={{ opacity: 0, scale: 0.9 }}
            animate={{ opacity: 1, scale: 1 }}
            transition={{ duration: 0.5, delay: 0.1 }}
            className="inline-flex items-center gap-2 px-4 py-1.5 rounded-full glass border border-bolt/20 text-bolt text-sm font-medium mb-8"
          >
            <Zap className="w-3.5 h-3.5" />
            Open Source &middot; Built with Rust
          </motion.div>

          <h1 className="text-5xl sm:text-6xl md:text-[5.5rem] font-black tracking-tight leading-[1.05] mb-6">
            Downloads at
            <br />
            <span className="relative">
              <span className="text-bolt">full speed.</span>
              <span className="absolute -inset-x-4 -inset-y-2 bg-bolt/[0.06] blur-2xl rounded-3xl -z-10" />
            </span>
          </h1>

          <p className="text-lg md:text-xl text-text-secondary leading-relaxed max-w-2xl mx-auto mb-12">
            Multi-threaded download manager that splits files into parallel
            segments for maximum throughput. Pause, resume, schedule — all from
            a clean native UI.
          </p>

          <div className="flex flex-col sm:flex-row items-center justify-center gap-4">
            <motion.a
              href="#download"
              whileHover={{ scale: 1.03 }}
              whileTap={{ scale: 0.98 }}
              aria-label="Download Bolt download manager"
              className="group relative flex items-center gap-2.5 px-8 py-4 bg-bolt text-black font-semibold rounded-2xl shadow-xl shadow-bolt/25 transition-shadow hover:shadow-2xl hover:shadow-bolt/30"
            >
              <Download className="w-5 h-5 transition-transform group-hover:-translate-y-0.5" />
              Download Bolt
              <span className="absolute inset-0 rounded-2xl bg-white/0 group-hover:bg-white/10 transition-colors" />
            </motion.a>
            <motion.a
              href="https://github.com/dasunNimantha/bolt"
              target="_blank"
              rel="noopener noreferrer"
              whileHover={{ scale: 1.03 }}
              whileTap={{ scale: 0.98 }}
              aria-label="View Bolt source code on GitHub"
              className="flex items-center gap-2.5 px-8 py-4 glass-card border border-white/[0.08] text-text-primary font-semibold rounded-2xl hover:border-white/[0.15] transition-all"
            >
              <Github className="w-5 h-5" />
              View Source
            </motion.a>
          </div>
        </motion.div>

        {/* App preview */}
        <motion.div
          initial={{ opacity: 0, y: 60, scale: 0.92 }}
          animate={{ opacity: 1, y: 0, scale: 1 }}
          transition={{ duration: 1, delay: 0.3, ease: [0.22, 1, 0.36, 1] }}
          className="relative mt-20 md:mt-28 max-w-4xl mx-auto"
        >
          <div className="absolute -inset-8 rounded-3xl bg-gradient-to-b from-bolt/[0.08] via-bolt/[0.03] to-transparent blur-2xl -z-10" />
          <div className="absolute -inset-px rounded-2xl bg-gradient-to-b from-white/[0.08] to-transparent -z-[1]" />

          <div className="relative rounded-2xl border border-white/[0.06] bg-surface-card overflow-hidden shadow-2xl shadow-black/50 noise">
            {/* Title bar */}
            <div className="relative z-10 flex items-center gap-2 px-5 py-3.5 bg-surface-raised/80 border-b border-white/[0.06]">
              <div className="flex gap-2">
                <div className="w-3 h-3 rounded-full bg-red-400/70 shadow-sm shadow-red-400/30" />
                <div className="w-3 h-3 rounded-full bg-yellow-400/70 shadow-sm shadow-yellow-400/30" />
                <div className="w-3 h-3 rounded-full bg-green-400/70 shadow-sm shadow-green-400/30" />
              </div>
              <span className="ml-3 text-xs text-text-muted font-mono tracking-wide">
                Bolt Download Manager
              </span>
            </div>
            {/* Mock download list */}
            <div className="relative z-10 p-5 space-y-2.5">
              <MockDownload
                name="ubuntu-24.04-desktop-amd64.iso"
                size="4.7 GB"
                progress={73}
                speed="42.5 MB/s"
                status="downloading"
                segments={8}
                delay={0.5}
              />
              <MockDownload
                name="rustup-init.exe"
                size="12.3 MB"
                progress={100}
                speed=""
                status="completed"
                segments={4}
                delay={0.7}
              />
              <MockDownload
                name="node-v22.0.0-linux-x64.tar.xz"
                size="28.6 MB"
                progress={45}
                speed="18.2 MB/s"
                status="downloading"
                segments={6}
                delay={0.9}
              />
            </div>
          </div>
        </motion.div>
      </div>
    </section>
  );
}

function MockDownload({
  name,
  size,
  progress,
  speed,
  status,
  segments,
  delay,
}: {
  name: string;
  size: string;
  progress: number;
  speed: string;
  status: "downloading" | "completed" | "queued";
  segments: number;
  delay: number;
}) {
  return (
    <div className="flex items-center gap-4 p-3.5 rounded-xl bg-white/[0.02] border border-white/[0.04] hover:bg-white/[0.04] hover:border-white/[0.07] transition-all duration-300">
      <div
        className={`w-10 h-10 rounded-xl flex items-center justify-center shrink-0 ${
          status === "completed"
            ? "bg-green-500/10 text-green-400 shadow-inner shadow-green-500/5"
            : status === "downloading"
              ? "bg-bolt/10 text-bolt shadow-inner shadow-bolt/5"
              : "bg-white/[0.04] text-text-muted"
        }`}
      >
        {status === "completed" ? (
          <svg className="w-4.5 h-4.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2.5} d="M5 13l4 4L19 7" />
          </svg>
        ) : (
          <Download className="w-4.5 h-4.5" />
        )}
      </div>

      <div className="flex-1 min-w-0">
        <div className="flex items-center justify-between mb-2">
          <span className="text-[13px] font-medium truncate mr-4">{name}</span>
          <span className="text-xs text-text-muted shrink-0 flex items-center gap-2">
            {size}
            {speed && (
              <span className="text-bolt font-mono font-medium">{speed}</span>
            )}
          </span>
        </div>
        <div className="flex items-center gap-3">
          <div className="flex-1 h-[5px] rounded-full bg-white/[0.04] overflow-hidden">
            <motion.div
              initial={{ width: 0 }}
              animate={{ width: `${progress}%` }}
              transition={{ duration: 1.8, delay, ease: "easeOut" }}
              className={`h-full rounded-full ${
                status === "completed"
                  ? "bg-gradient-to-r from-green-500 to-green-400"
                  : "bg-gradient-to-r from-bolt-dark via-bolt to-bolt/80"
              }`}
              style={{
                boxShadow:
                  status === "completed"
                    ? "0 0 12px rgba(74, 222, 128, 0.3)"
                    : "0 0 12px rgba(242, 191, 64, 0.25)",
              }}
            />
          </div>
          <span className="text-xs text-text-muted w-10 text-right font-mono">
            {progress}%
          </span>
          {status === "downloading" && (
            <span className="text-[10px] text-text-muted/60 font-mono bg-white/[0.03] px-1.5 py-0.5 rounded">
              {segments}×
            </span>
          )}
        </div>
      </div>
    </div>
  );
}
