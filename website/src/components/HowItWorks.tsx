import { motion } from "framer-motion";
import { Globe, ArrowRight, Cpu, Zap, ChevronRight } from "lucide-react";

const steps = [
  {
    icon: Globe,
    num: "01",
    title: "Install the Extension",
    desc: "Add the Bolt extension to your browser. Works with Chrome, Edge, Firefox, Brave, Vivaldi and other Chromium-based browsers.",
    color: "from-blue-500/20 to-blue-500/5",
  },
  {
    icon: ArrowRight,
    num: "02",
    title: "Click Any Download",
    desc: "When you click a download link, Bolt catches it and shows a confirmation popup with file details.",
    color: "from-purple-500/20 to-purple-500/5",
  },
  {
    icon: Cpu,
    num: "03",
    title: "Multi-Segment Magic",
    desc: "Bolt splits the file into parallel segments, downloading each simultaneously for maximum speed.",
    color: "from-bolt/20 to-bolt/5",
  },
  {
    icon: Zap,
    num: "04",
    title: "Done. Fast.",
    desc: "Your file downloads at full speed with automatic resume, retry, and progress tracking.",
    color: "from-green-500/20 to-green-500/5",
  },
];

export function HowItWorks() {
  return (
    <section id="how-it-works" className="py-16 md:py-24 relative">
      <div className="absolute inset-0 -z-10">
        <div className="absolute top-0 left-0 right-0 h-px bg-gradient-to-r from-transparent via-white/[0.06] to-transparent" />
        <div className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-[700px] h-[500px] rounded-full bg-bolt/[0.04] blur-[150px]" />
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
            How it works
          </h2>
          <p className="text-text-secondary text-lg max-w-xl mx-auto">
            From browser click to completed download in seconds.
          </p>
        </motion.div>

        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-5">
          {steps.map((step, i) => (
            <motion.div
              key={step.num}
              initial={{ opacity: 0, y: 30 }}
              whileInView={{ opacity: 1, y: 0 }}
              viewport={{ once: true }}
              transition={{
                duration: 0.5,
                delay: i * 0.1,
                ease: [0.22, 1, 0.36, 1],
              }}
              className="relative p-6 rounded-2xl glass-card border border-white/[0.05] overflow-hidden group hover:border-white/[0.1] transition-all duration-300"
            >
              <div className={`absolute inset-0 bg-gradient-to-b ${step.color} opacity-0 group-hover:opacity-100 transition-opacity duration-500`} />
              <span className="text-6xl font-black text-white/[0.03] absolute top-2 right-4 select-none group-hover:text-white/[0.06] transition-colors">
                {step.num}
              </span>
              <div className="relative z-10">
                <div className="w-10 h-10 rounded-xl bg-bolt/[0.08] border border-bolt/[0.1] flex items-center justify-center mb-4">
                  <step.icon className="w-5 h-5 text-bolt" />
                </div>
                <h3 className="text-[15px] font-semibold mb-2">{step.title}</h3>
                <p className="text-sm text-text-secondary leading-relaxed">
                  {step.desc}
                </p>
              </div>
            </motion.div>
          ))}
        </div>

        {/* Architecture diagram */}
        <motion.div
          initial={{ opacity: 0, y: 30 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 0.6, delay: 0.3 }}
          className="mt-16 p-6 md:p-8 rounded-2xl glass-card border border-white/[0.05] noise relative overflow-hidden"
        >
          <h3 className="relative z-10 text-lg font-semibold mb-8 text-center">
            Architecture
          </h3>
          <div className="relative z-10 flex flex-col md:flex-row items-center justify-center gap-3 md:gap-0">
            {[
              { label: "Browser Extension", sub: "Intercepts downloads" },
              { label: "Native Messaging Host", sub: "stdin/stdout bridge" },
              { label: "Bolt IPC Server", sub: "localhost:9817" },
              { label: "Download Engine", sub: "Multi-segment worker" },
            ].map((block, i) => (
              <div key={i} className="flex items-center gap-3">
                <div className="px-5 py-4 rounded-xl bg-white/[0.03] border border-white/[0.06] text-center min-w-[175px] hover:bg-white/[0.05] hover:border-white/[0.1] transition-all duration-300">
                  <div className="text-sm font-medium">{block.label}</div>
                  <div className="text-xs text-text-muted mt-1 font-mono">
                    {block.sub}
                  </div>
                </div>
                {i < 3 && (
                  <ChevronRight className="w-4 h-4 text-bolt/60 shrink-0 hidden md:block" />
                )}
              </div>
            ))}
          </div>
        </motion.div>
      </div>
    </section>
  );
}
