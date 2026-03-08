import { useState, useEffect } from "react";
import { Link } from "react-router-dom";
import { BoltLogo } from "./BoltLogo";
import { Github, Menu, X, Star } from "lucide-react";

const links = [
  { label: "Features", href: "/#features", route: false },
  { label: "How It Works", href: "/#how-it-works", route: false },
  { label: "Docs", href: "/docs", route: true },
  { label: "Download", href: "/#download", route: false },
];

function NavLink({ label, href, route, onClick }: { label: string; href: string; route: boolean; onClick?: () => void }) {
  const cls = "text-sm text-text-secondary hover:text-text-primary transition-colors relative after:absolute after:bottom-[-4px] after:left-0 after:w-0 after:h-px after:bg-bolt after:transition-all after:duration-300 hover:after:w-full";
  if (route) {
    return <Link to={href} onClick={onClick} className={cls}>{label}</Link>;
  }
  return <a href={href} onClick={onClick} className={cls}>{label}</a>;
}

export function Navbar() {
  const [scrolled, setScrolled] = useState(false);
  const [mobileOpen, setMobileOpen] = useState(false);

  useEffect(() => {
    const onScroll = () => setScrolled(window.scrollY > 20);
    window.addEventListener("scroll", onScroll, { passive: true });
    return () => window.removeEventListener("scroll", onScroll);
  }, []);

  return (
    <nav
      className={`fixed top-0 left-0 right-0 z-50 transition-all duration-500 border-b ${
        scrolled
          ? "bg-surface/80 backdrop-blur-xl border-white/[0.06] shadow-lg shadow-black/20"
          : "bg-transparent border-transparent"
      }`}
    >
      <div className="max-w-6xl mx-auto px-6 h-16 flex items-center justify-between">
        <Link to="/" className="flex items-center gap-2.5 group">
          <BoltLogo className="w-8 h-8 transition-transform duration-300 group-hover:scale-110" />
          <span className="text-lg font-bold tracking-tight">Bolt</span>
        </Link>

        <div className="hidden md:flex items-center gap-8">
          {links.map((l) => (
            <NavLink key={l.href} {...l} />
          ))}
          <a
            href="https://github.com/dasunNimantha/bolt"
            target="_blank"
            rel="noopener noreferrer"
            className="flex items-center gap-2 px-4 py-2 rounded-lg bg-white/[0.04] border border-white/[0.06] text-sm text-text-secondary hover:text-text-primary hover:bg-white/[0.08] hover:border-white/[0.1] transition-all"
          >
            <Github className="w-4 h-4" />
            <Star className="w-3 h-3" />
            Star
          </a>
        </div>

        <button
          onClick={() => setMobileOpen(!mobileOpen)}
          className="md:hidden text-text-secondary hover:text-text-primary p-2 rounded-lg hover:bg-white/[0.06] transition-colors"
        >
          {mobileOpen ? <X className="w-5 h-5" /> : <Menu className="w-5 h-5" />}
        </button>
      </div>

      {mobileOpen && (
        <div className="md:hidden bg-surface/80 backdrop-blur-xl border-b border-white/[0.06] px-6 pb-4 pt-1">
          {links.map((l) => (
            <div key={l.href} className="py-1">
              <NavLink {...l} onClick={() => setMobileOpen(false)} />
            </div>
          ))}
          <a
            href="https://github.com/dasunNimantha/bolt"
            target="_blank"
            rel="noopener noreferrer"
            className="flex items-center gap-2 py-3 text-sm text-text-secondary hover:text-text-primary transition-colors"
          >
            <Github className="w-4 h-4" />
            GitHub
          </a>
        </div>
      )}
    </nav>
  );
}
