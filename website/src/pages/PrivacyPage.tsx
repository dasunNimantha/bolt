import { useEffect } from "react";
import { Navbar } from "../components/Navbar";
import { Footer } from "../components/Footer";
import { Shield, ExternalLink } from "lucide-react";

const REPO = "dasunNimantha/bolt";

export function PrivacyPage() {
  useEffect(() => {
    window.scrollTo(0, 0);
  }, []);

  return (
    <div className="min-h-screen">
      <Navbar />

      <div className="max-w-3xl mx-auto px-6 pt-28 pb-20">
        <div className="mb-12">
          <div className="flex items-center gap-2 text-bolt text-sm font-medium mb-3">
            <Shield className="w-4 h-4" />
            Privacy
          </div>
          <h1 className="text-3xl md:text-4xl font-bold tracking-tight">
            Privacy Policy
          </h1>
          <p className="text-text-secondary mt-2">
            Bolt Download Manager Browser Extension &middot; Last updated March 8, 2026
          </p>
        </div>

        <article className="space-y-10">
          <Section title="Overview">
            <P>
              Bolt Download Manager ("the Extension") is a browser extension that intercepts
              downloads initiated in your browser and hands them off to the Bolt desktop application
              installed on your computer. The Extension is designed with privacy as a core principle:
              all data stays on your machine and is never transmitted to any external server.
            </P>
          </Section>

          <Section title="Data Accessed">
            <P>
              When a download is initiated in your browser, the Extension accesses the following
              information:
            </P>
            <ul className="space-y-3 mt-4">
              <Li label="Download URL">The address of the file being downloaded.</Li>
              <Li label="Filename">The name suggested by the browser or server for the downloaded file.</Li>
              <Li label="Referrer">The page URL from which the download was triggered.</Li>
              <Li label="Cookies" opt>
                Cookies associated with the download domain, used to authenticate the download
                request so Bolt can resume or accelerate it. Cookie access requires an explicit
                opt-in by the user via the extension popup.
              </Li>
            </ul>
          </Section>

          <Section title="How Data Is Used">
            <P>
              All accessed data is sent exclusively to the <strong className="text-text-primary">locally installed
              Bolt desktop application</strong> on your computer via the browser's Native Messaging API.
              This communication happens entirely on your local machine through an OS-level IPC channel.
              The data is used solely to:
            </P>
            <ul className="space-y-2 mt-4">
              <BulletLi>Add the download to Bolt's download queue.</BulletLi>
              <BulletLi>Authenticate with the server hosting the file (using the forwarded cookies, if granted).</BulletLi>
            </ul>
            <P className="mt-4">
              If the Bolt desktop application is not running, the Extension falls back to the
              browser's default download handler and no data is sent.
            </P>
          </Section>

          <Section title="Data Storage">
            <P>
              The Extension stores a single preference (<code className="text-text-primary text-xs bg-white/[0.04] px-1.5 py-0.5 rounded">enabled: true/false</code>) in
              local storage to remember whether download interception is turned on or off.
              No download URLs, cookies, filenames, or any other personal data is stored by the Extension.
            </P>
          </Section>

          <Section title="Data Sharing">
            <P>The Extension does <strong className="text-text-primary">not</strong>:</P>
            <ul className="space-y-2 mt-4">
              <BulletLi>Collect analytics or telemetry.</BulletLi>
              <BulletLi>Send any data to external servers, third parties, or cloud services.</BulletLi>
              <BulletLi>Track browsing history or user behavior.</BulletLi>
              <BulletLi>Use any data for advertising purposes.</BulletLi>
            </ul>
            <P className="mt-4">
              All data remains on your local machine and is only shared between the Extension and
              the Bolt desktop application.
            </P>
          </Section>

          <Section title="Permissions Justification">
            <div className="rounded-xl border border-white/[0.08] overflow-hidden backdrop-blur-sm shadow-lg shadow-black/20 mt-4">
              <table className="w-full text-sm">
                <thead>
                  <tr className="border-b border-white/[0.06] bg-white/[0.02]">
                    <th className="text-left px-4 py-2.5 text-xs font-medium text-text-muted uppercase tracking-wider">Permission</th>
                    <th className="text-left px-4 py-2.5 text-xs font-medium text-text-muted uppercase tracking-wider">Reason</th>
                  </tr>
                </thead>
                <tbody>
                  {([
                    ["downloads", "Detect new downloads and cancel them so Bolt can handle them. Falls back to the browser's default handler when Bolt isn't running."],
                    ["nativeMessaging", "Communicate with the Bolt desktop application via the native messaging host."],
                    ["storage", "Persist the on/off toggle preference."],
                    ["cookies (optional)", "Forward authentication cookies to Bolt so it can download files that require login. Requested at runtime when the user enables it."],
                    ["<all_urls> (optional)", "Access cookies for any domain from which a download may originate. Requested at runtime alongside cookies."],
                  ] as const).map(([perm, reason], i) => (
                    <tr key={i} className="border-b border-white/[0.03] last:border-0">
                      <td className="px-4 py-2.5 font-mono text-xs text-bolt whitespace-nowrap">{perm}</td>
                      <td className="px-4 py-2.5 text-text-secondary text-xs">{reason}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </Section>

          <Section title="Open Source">
            <P>
              The Extension and the Bolt desktop application are open source. You can review the
              complete source code at:
            </P>
            <a
              href={`https://github.com/${REPO}`}
              target="_blank"
              rel="noopener noreferrer"
              className="inline-flex items-center gap-1.5 mt-2 text-sm text-bolt hover:underline"
            >
              github.com/{REPO}
              <ExternalLink className="w-3 h-3" />
            </a>
          </Section>

          <Section title="Contact">
            <P>
              If you have questions about this privacy policy, please{" "}
              <a
                href={`https://github.com/${REPO}/issues`}
                target="_blank"
                rel="noopener noreferrer"
                className="text-bolt hover:underline"
              >
                open an issue
              </a>{" "}
              on the GitHub repository.
            </P>
          </Section>

          <Section title="Changes">
            <P>
              Any changes to this privacy policy will be reflected in this document and in
              the Extension's browser store listing.
            </P>
          </Section>
        </article>
      </div>

      <Footer />
    </div>
  );
}

function Section({ title, children }: { title: string; children: React.ReactNode }) {
  return (
    <section>
      <h2 className="text-lg font-bold mb-3 text-text-primary">{title}</h2>
      {children}
    </section>
  );
}

function P({ children, className = "" }: { children: React.ReactNode; className?: string }) {
  return <p className={`text-sm text-text-secondary leading-relaxed ${className}`}>{children}</p>;
}

function Li({ label, opt, children }: { label: string; opt?: boolean; children: React.ReactNode }) {
  return (
    <li className="flex items-start gap-3 text-sm text-text-secondary">
      <span className="shrink-0 mt-0.5 px-2 py-0.5 rounded-md bg-white/[0.04] border border-white/[0.08] font-mono text-xs text-bolt">
        {label}
        {opt && <span className="text-text-muted ml-1 font-sans text-[10px]">(optional)</span>}
      </span>
      <span className="leading-relaxed">{children}</span>
    </li>
  );
}

function BulletLi({ children }: { children: React.ReactNode }) {
  return (
    <li className="flex items-start gap-2.5 text-sm text-text-secondary">
      <span className="w-1.5 h-1.5 rounded-full bg-bolt/50 mt-[7px] shrink-0" />
      <span>{children}</span>
    </li>
  );
}
