import { useState, useEffect } from "react";
import { Navbar } from "../components/Navbar";
import { Footer } from "../components/Footer";
import { useSEO } from "../hooks/useSEO";
import {
  Terminal,
  Globe,
  Play,
  Settings,
  FolderTree,
  BookOpen,
  Copy,
  Check,
} from "lucide-react";

const REPO = "dasunNimantha/bolt";

function CopyButton({ text }: { text: string }) {
  const [copied, setCopied] = useState(false);
  return (
    <button
      onClick={() => {
        navigator.clipboard.writeText(text);
        setCopied(true);
        setTimeout(() => setCopied(false), 2000);
      }}
      className="p-1 rounded-md hover:bg-white/[0.06] text-text-muted hover:text-text-primary transition-all opacity-0 group-hover/code:opacity-100"
    >
      {copied ? <Check className="w-3.5 h-3.5 text-green-400" /> : <Copy className="w-3.5 h-3.5" />}
    </button>
  );
}

function CodeBlock({ children }: { children: string }) {
  const raw = children.replace(/^\$ /gm, "");
  return (
    <div className="group/code relative rounded-xl border border-white/[0.1] overflow-hidden my-4 shadow-xl shadow-black/30 bg-gradient-to-b from-white/[0.04] to-white/[0.01] backdrop-blur-xl">
      <div className="flex items-center justify-between px-4 py-2.5 border-b border-white/[0.07] bg-white/[0.03]">
        <div className="flex items-center gap-1.5">
          <div className="w-2.5 h-2.5 rounded-full bg-red-400/40" />
          <div className="w-2.5 h-2.5 rounded-full bg-yellow-400/40" />
          <div className="w-2.5 h-2.5 rounded-full bg-green-400/40" />
        </div>
        <CopyButton text={raw} />
      </div>
      <pre className="p-4 text-[13px] leading-[1.8] font-mono overflow-x-auto bg-black/20">
        {children.split("\n").map((line, i) => (
          <div key={i} className={line.startsWith("#") ? "text-text-muted/40" : ""}>
            {line.startsWith("$ ") ? (
              <>
                <span className="text-bolt select-none">$ </span>
                <span className="text-text-primary">{line.slice(2)}</span>
              </>
            ) : (
              <span className="text-text-primary/80">{line}</span>
            )}
          </div>
        ))}
      </pre>
    </div>
  );
}

function Callout({ children }: { children: React.ReactNode }) {
  return (
    <div className="flex gap-3 p-4 rounded-xl bg-bolt/[0.03] border border-bolt/[0.1] my-4 backdrop-blur-sm shadow-md shadow-bolt/[0.03]">
      <span className="text-bolt text-sm mt-px shrink-0">💡</span>
      <div className="text-sm text-text-secondary leading-relaxed">{children}</div>
    </div>
  );
}

interface NavSection {
  id: string;
  icon: typeof Terminal;
  title: string;
}

const nav: NavSection[] = [
  { id: "installation", icon: Terminal, title: "Installation" },
  { id: "browser-extension", icon: Globe, title: "Browser Extension" },
  { id: "usage", icon: Play, title: "Usage" },
  { id: "configuration", icon: Settings, title: "Configuration" },
  { id: "architecture", icon: FolderTree, title: "Architecture" },
];

export function DocsPage() {
  const [active, setActive] = useState("installation");

  useSEO({
    title: "Documentation — Bolt Download Manager",
    description:
      "Installation guide, browser extension setup, configuration, and architecture overview for Bolt download manager.",
    canonical: "https://boltdm.site/docs",
  });

  useEffect(() => {
    window.scrollTo(0, 0);
  }, []);

  useEffect(() => {
    const observer = new IntersectionObserver(
      (entries) => {
        for (const entry of entries) {
          if (entry.isIntersecting) setActive(entry.target.id);
        }
      },
      { rootMargin: "-100px 0px -65% 0px" }
    );
    nav.forEach((s) => {
      const el = document.getElementById(s.id);
      if (el) observer.observe(el);
    });
    return () => observer.disconnect();
  }, []);

  return (
    <div className="min-h-screen">
      <Navbar />

      <main className="max-w-[1100px] mx-auto px-6 pt-28 pb-20">
        {/* Header */}
        <div className="mb-12">
          <div className="flex items-center gap-2 text-bolt text-sm font-medium mb-3">
            <BookOpen className="w-4 h-4" />
            Documentation
          </div>
          <h1 className="text-3xl md:text-4xl font-bold tracking-tight">
            Getting started with Bolt
          </h1>
          <p className="text-text-secondary mt-2 max-w-xl">
            Installation, browser setup, and everything else you need.
          </p>
        </div>

        <div className="flex gap-16">
          {/* Sidebar */}
          <aside className="hidden lg:block w-48 shrink-0">
            <nav className="sticky top-28 space-y-0.5">
              {nav.map((s) => (
                <a
                  key={s.id}
                  href={`#${s.id}`}
                  className={`flex items-center gap-2.5 px-3 py-2 rounded-lg text-[13px] transition-all duration-200 ${
                    active === s.id
                      ? "text-bolt bg-bolt/[0.06] font-medium"
                      : "text-text-muted hover:text-text-secondary"
                  }`}
                >
                  <s.icon className="w-3.5 h-3.5 shrink-0" />
                  {s.title}
                </a>
              ))}
            </nav>
          </aside>

          {/* Content */}
          <article className="flex-1 min-w-0">
            {/* Installation */}
            <section id="installation" className="scroll-mt-28 mb-16">
              <H2>Installation</H2>
              <P>
                Grab a pre-built binary from the <a href="/#download" className="text-bolt hover:underline">download page</a>,
                or build from source with <strong>Rust 1.88+</strong>.
              </P>

              <H3>Prerequisites</H3>
              <P>
                macOS and Windows need no extra dependencies. Linux requires a few system packages:
              </P>

              <Tabs
                tabs={[
                  {
                    label: "Debian / Ubuntu",
                    content: (
                      <CodeBlock>{`$ sudo apt install pkg-config libssl-dev libfontconfig-dev libgtk-3-dev libayatana-appindicator3-dev`}</CodeBlock>
                    ),
                  },
                  {
                    label: "Fedora",
                    content: (
                      <CodeBlock>{`$ sudo dnf install pkg-config openssl-devel fontconfig-devel gtk3-devel libayatana-appindicator-gtk3-devel`}</CodeBlock>
                    ),
                  },
                  {
                    label: "Arch",
                    content: (
                      <CodeBlock>{`$ sudo pacman -S pkg-config openssl fontconfig gtk3 libayatana-appindicator`}</CodeBlock>
                    ),
                  },
                ]}
              />

              <H3>Build from source</H3>
              <CodeBlock>{`$ git clone https://github.com/${REPO}.git\n$ cd bolt\n$ cargo build --workspace --release\n$ ./target/release/bolt`}</CodeBlock>
              <Callout>
                On Windows, the binary is at <code className="text-text-primary text-xs">.\\target\\release\\bolt.exe</code>
              </Callout>
            </section>

            <Divider />

            {/* Browser Extension */}
            <section id="browser-extension" className="scroll-mt-28 mb-16">
              <H2>Browser Extension</H2>
              <P>
                The extension intercepts downloads and sends them to the Bolt desktop app.
                Works with Chrome, Edge, Brave, Vivaldi, Firefox, and other Chromium browsers.
              </P>

              <H3>Step 1 — Install the extension</H3>

              <Tabs
                tabs={[
                  {
                    label: "Chrome / Chromium",
                    content: (
                      <>
                        <P>Install from the Chrome Web Store (coming soon), or load manually:</P>
                        <ol className="space-y-2 ml-1 my-3">
                          <Step n={1}>Open <code className="text-text-primary text-xs bg-white/[0.04] px-1.5 py-0.5 rounded">chrome://extensions</code> in your browser</Step>
                          <Step n={2}>Enable <strong className="text-text-primary">Developer Mode</strong> (top-right toggle)</Step>
                          <Step n={3}>Click <strong className="text-text-primary">Load unpacked</strong> → select the <code className="text-text-primary text-xs bg-white/[0.04] px-1.5 py-0.5 rounded">extension/</code> folder</Step>
                        </ol>
                      </>
                    ),
                  },
                  {
                    label: "Firefox",
                    content: (
                      <>
                        <P>Install from Firefox Add-ons (coming soon), or load temporarily:</P>
                        <ol className="space-y-2 ml-1 my-3">
                          <Step n={1}>Open <code className="text-text-primary text-xs bg-white/[0.04] px-1.5 py-0.5 rounded">about:debugging#/runtime/this-firefox</code></Step>
                          <Step n={2}>Click <strong className="text-text-primary">Load Temporary Add-on</strong></Step>
                          <Step n={3}>Select <code className="text-text-primary text-xs bg-white/[0.04] px-1.5 py-0.5 rounded">extension/manifest.json</code></Step>
                        </ol>
                        <Callout>
                          Temporary add-ons are removed when Firefox restarts. For permanent installs, use Firefox Add-ons or sign the extension via <code className="text-text-primary text-xs">web-ext sign</code>.
                        </Callout>
                      </>
                    ),
                  },
                ]}
              />

              <H3>Step 2 — Build the native messaging host</H3>
              <CodeBlock>{`$ cargo build --release -p bolt-nmh`}</CodeBlock>
              <P>
                Bolt auto-installs the native messaging manifest when it starts — no manual config needed.
              </P>

              <H3>Step 3 — Verify the connection</H3>
              <P>
                Click the Bolt icon in your browser toolbar. The popup should show{" "}
                <span className="text-green-400 font-medium">Connected to Bolt</span>.
              </P>
              <Callout>
                If Bolt isn't running, downloads automatically fall back to your browser's default downloader. Nothing is lost.
              </Callout>

              <H3>Cookie forwarding</H3>
              <P>
                For downloads behind a login, enable <strong>Forward Cookies</strong> in the extension popup.
                This grants the cookies permission at runtime — it's optional and can be revoked anytime.
              </P>
            </section>

            <Divider />

            {/* Usage */}
            <section id="usage" className="scroll-mt-28 mb-16">
              <H2>Usage</H2>

              <H3>Adding downloads</H3>
              <ul className="space-y-2 my-3">
                <Li>Paste a URL into the input bar and click <strong className="text-text-primary">Add</strong></Li>
                <Li>For bulk downloads, paste multiple URLs (one per line) or import a <code className="text-text-primary text-xs bg-white/[0.04] px-1.5 py-0.5 rounded">.txt</code> file</Li>
                <Li>With the browser extension, clicking any download link sends it straight to Bolt</Li>
              </ul>

              <H3>Controls</H3>
              <ul className="space-y-2 my-3">
                <Li><strong className="text-text-primary">Play / Pause</strong> — start or pause a download</Li>
                <Li><strong className="text-text-primary">Cancel</strong> — stop and remove from active queue</Li>
                <Li>Completed files can be <strong className="text-text-primary">opened</strong> or their <strong className="text-text-primary">folder revealed</strong></Li>
              </ul>

              <H3>Background mode</H3>
              <P>
                Closing the window minimizes Bolt to the system tray — downloads keep running.
                Right-click the tray icon to show the window or quit.
              </P>

              <H3>Smart queue</H3>
              <P>
                When a download finishes, the next queued item starts automatically.
                Failed segments retry on their own. If the network drops, Bolt pauses and resumes when you're back online.
              </P>
            </section>

            <Divider />

            {/* Configuration */}
            <section id="configuration" className="scroll-mt-28 mb-16">
              <H2>Configuration</H2>
              <P>Open settings via the gear icon in the header.</P>

              <div className="grid grid-cols-1 sm:grid-cols-2 gap-3 my-5">
                {[
                  { label: "Download directory", desc: "Where files are saved" },
                  { label: "Speed limit", desc: "Bandwidth cap in KB/s (0 = unlimited)" },
                  { label: "Concurrent downloads", desc: "1–10 simultaneous downloads" },
                  { label: "Segments per file", desc: "Up to 8 parallel segments" },
                  { label: "Scheduling", desc: "Daily time window for auto-start" },
                  { label: "Theme", desc: "Dark, Light, or System (auto)" },
                ].map((item) => (
                  <div key={item.label} className="p-3.5 rounded-xl bg-white/[0.02] border border-white/[0.08] backdrop-blur-sm hover:bg-white/[0.04] hover:border-white/[0.12] transition-all duration-300">
                    <div className="text-[13px] font-medium text-text-primary">{item.label}</div>
                    <div className="text-xs text-text-muted mt-0.5">{item.desc}</div>
                  </div>
                ))}
              </div>

              <H3>Storage location</H3>
              <Tabs
                tabs={[
                  { label: "Linux", content: <P><code className="text-text-primary text-xs bg-white/[0.04] px-1.5 py-0.5 rounded">~/.config/bolt/</code></P> },
                  { label: "macOS", content: <P><code className="text-text-primary text-xs bg-white/[0.04] px-1.5 py-0.5 rounded">~/Library/Application Support/bolt/</code></P> },
                  { label: "Windows", content: <P><code className="text-text-primary text-xs bg-white/[0.04] px-1.5 py-0.5 rounded">%APPDATA%\bolt\</code></P> },
                ]}
              />
            </section>

            <Divider />

            {/* Architecture */}
            <section id="architecture" className="scroll-mt-28 mb-8">
              <H2>Architecture</H2>
              <P>
                Bolt uses an Elm-style architecture: a <code className="text-text-primary text-xs bg-white/[0.04] px-1.5 py-0.5 rounded">Message</code> enum
                drives updates, which trigger view re-renders. Built on Iced 0.14 in multi-window daemon mode.
              </P>

              <CodeBlock>{`src/
├── main.rs            # Entry point
├── app.rs             # State + message handling
├── view.rs            # UI rendering (multi-window)
├── message.rs         # Message enum
├── model.rs           # Data structures
├── settings.rs        # Persistent settings (JSON)
├── theme.rs           # Styles and colors
├── tray.rs            # System tray
├── ipc.rs             # IPC server (localhost:9817)
├── nmh.rs             # NMH auto-installer
├── download/
│   ├── engine.rs      # Queue, segments, persistence
│   └── worker.rs      # HTTP streaming, retry, throttle
└── utils/
    └── format.rs      # Formatting helpers`}</CodeBlock>

              <H3>Dependencies</H3>
              <div className="my-4 rounded-xl border border-white/[0.08] overflow-hidden backdrop-blur-sm shadow-lg shadow-black/20">
                <table className="w-full text-sm">
                  <thead>
                    <tr className="border-b border-white/[0.06] bg-white/[0.02]">
                      <th className="text-left px-4 py-2.5 text-xs font-medium text-text-muted uppercase tracking-wider">Crate</th>
                      <th className="text-left px-4 py-2.5 text-xs font-medium text-text-muted uppercase tracking-wider">Purpose</th>
                    </tr>
                  </thead>
                  <tbody>
                    {[
                      ["iced 0.14", "GUI framework (multi-window daemon)"],
                      ["tokio", "Async runtime"],
                      ["reqwest", "HTTP client with streaming"],
                      ["serde", "JSON serialization"],
                      ["tray-icon", "Cross-platform system tray"],
                      ["rfd", "Native file dialogs"],
                    ].map(([crate_, purpose], i) => (
                      <tr key={i} className="border-b border-white/[0.03] last:border-0">
                        <td className="px-4 py-2.5 font-mono text-xs text-bolt">{crate_}</td>
                        <td className="px-4 py-2.5 text-text-secondary text-xs">{purpose}</td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            </section>
          </article>
        </div>
      </main>

      <Footer />
    </div>
  );
}

/* ── Shared components ── */

function H2({ children }: { children: React.ReactNode }) {
  return <h2 className="text-xl font-bold mb-3">{children}</h2>;
}

function H3({ children }: { children: React.ReactNode }) {
  return <h3 className="text-sm font-semibold text-text-primary mt-6 mb-2">{children}</h3>;
}

function P({ children }: { children: React.ReactNode }) {
  return <p className="text-sm text-text-secondary leading-relaxed">{children}</p>;
}

function Li({ children }: { children: React.ReactNode }) {
  return (
    <li className="flex items-start gap-2.5 text-sm text-text-secondary">
      <span className="w-1.5 h-1.5 rounded-full bg-bolt/50 mt-[7px] shrink-0" />
      <span>{children}</span>
    </li>
  );
}

function Step({ n, children }: { n: number; children: React.ReactNode }) {
  return (
    <li className="flex items-start gap-3 text-sm text-text-secondary">
      <span className="w-5 h-5 rounded-md bg-bolt/[0.08] border border-bolt/[0.12] flex items-center justify-center text-[11px] font-bold text-bolt shrink-0 mt-px">
        {n}
      </span>
      <span>{children}</span>
    </li>
  );
}

function Divider() {
  return <div className="h-px bg-gradient-to-r from-transparent via-white/[0.06] to-transparent my-4" />;
}

function Tabs({ tabs }: { tabs: { label: string; content: React.ReactNode }[] }) {
  const [active, setActive] = useState(0);
  return (
    <div className="my-4">
      <div className="flex gap-0.5 p-1 rounded-xl bg-white/[0.03] border border-white/[0.08] backdrop-blur-sm w-fit">
        {tabs.map((t, i) => (
          <button
            key={t.label}
            onClick={() => setActive(i)}
            className={`px-3.5 py-1.5 rounded-md text-xs font-medium transition-all ${
              active === i
                ? "bg-bolt/[0.12] text-bolt"
                : "text-text-muted hover:text-text-secondary"
            }`}
          >
            {t.label}
          </button>
        ))}
      </div>
      <div className="mt-1">{tabs[active].content}</div>
    </div>
  );
}
