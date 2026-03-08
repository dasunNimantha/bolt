import { motion } from "framer-motion";
import {
  Layers,
  Pause,
  Clock,
  Wifi,
  FileStack,
  Globe,
} from "lucide-react";

const features = [
  {
    icon: Layers,
    title: "Multi-Segment Downloads",
    desc: "Splits files into up to 8 parallel segments for maximum throughput on any connection.",
  },
  {
    icon: Pause,
    title: "Pause & Resume",
    desc: "Stop and continue downloads without losing progress. State persists across restarts.",
  },
  {
    icon: Globe,
    title: "Browser Integration",
    desc: "Extension intercepts downloads from Chrome, Edge, Firefox, Brave and more — with a confirmation popup.",
  },
  {
    icon: Clock,
    title: "Scheduling & Queue",
    desc: "Set daily time windows for downloads. Smart queue auto-starts when slots open.",
  },
  {
    icon: Wifi,
    title: "Auto-Resume",
    desc: "Detects network recovery and automatically retries failed downloads.",
  },
  {
    icon: FileStack,
    title: "Batch Downloads",
    desc: "Paste multiple URLs or import from a text file to queue downloads in bulk.",
  },
];

const container = {
  hidden: {},
  show: { transition: { staggerChildren: 0.06 } },
};

const item = {
  hidden: { opacity: 0, y: 24 },
  show: {
    opacity: 1,
    y: 0,
    transition: { duration: 0.5, ease: [0.22, 1, 0.36, 1] as const },
  },
};

export function Features() {
  return (
    <section id="features" className="py-16 md:py-24 relative">
      <div className="absolute inset-0 -z-10">
        <div className="absolute top-0 left-0 right-0 h-px bg-gradient-to-r from-transparent via-white/[0.06] to-transparent" />
        <div className="absolute top-1/2 left-1/4 w-[500px] h-[500px] rounded-full bg-bolt/[0.04] blur-[150px]" />
        <div className="absolute bottom-1/4 right-1/4 w-[400px] h-[400px] rounded-full bg-purple-500/[0.02] blur-[120px]" />
      </div>

      <div className="max-w-6xl mx-auto px-6">
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true, margin: "-100px" }}
          transition={{ duration: 0.6 }}
          className="text-center mb-16"
        >
          <h2 className="text-3xl md:text-5xl font-bold tracking-tight mb-4">
            Everything you need.
            <br />
            <span className="text-text-secondary">Nothing you don't.</span>
          </h2>
          <p className="text-text-secondary text-lg max-w-xl mx-auto">
            Built for power users who want full control over their downloads.
          </p>
        </motion.div>

        <motion.div
          variants={container}
          initial="hidden"
          whileInView="show"
          viewport={{ once: true, margin: "-50px" }}
          className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-5"
        >
          {features.map((f) => (
            <motion.div
              key={f.title}
              variants={item}
              className="group relative p-6 rounded-2xl glass-card border border-white/[0.05] hover:border-white/[0.1] transition-all duration-400 glow-border overflow-hidden"
            >
              <div className="absolute inset-0 shimmer pointer-events-none opacity-0 group-hover:opacity-100 transition-opacity duration-500" />
              <div className="relative z-10">
                <div className="w-11 h-11 rounded-xl bg-bolt/[0.08] border border-bolt/[0.1] flex items-center justify-center mb-4 group-hover:bg-bolt/[0.12] group-hover:border-bolt/[0.2] transition-all duration-300 group-hover:shadow-lg group-hover:shadow-bolt/10">
                  <f.icon className="w-5 h-5 text-bolt" />
                </div>
                <h3 className="text-[15px] font-semibold mb-2">{f.title}</h3>
                <p className="text-sm text-text-secondary leading-relaxed">
                  {f.desc}
                </p>
              </div>
            </motion.div>
          ))}
        </motion.div>
      </div>
    </section>
  );
}
