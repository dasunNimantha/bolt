import { Link } from "react-router-dom";
import { BoltLogo } from "./BoltLogo";
import { Github, ExternalLink, Heart } from "lucide-react";

export function Footer() {
  return (
    <footer className="relative border-t border-white/[0.04] py-14">
      <div className="absolute inset-0 -z-10">
        <div className="absolute bottom-0 left-1/2 -translate-x-1/2 w-[500px] h-[200px] rounded-full bg-bolt/[0.03] blur-[100px]" />
      </div>

      <div className="max-w-6xl mx-auto px-6">
        <div className="flex flex-col md:flex-row items-center justify-between gap-8">
          <div className="flex items-center gap-3">
            <BoltLogo className="w-6 h-6" />
            <span className="text-sm text-text-muted">
              Built with <Heart className="w-3 h-3 inline text-bolt/60 mx-0.5" /> using Rust &middot; MIT License
            </span>
          </div>

          <div className="flex items-center gap-6">
            <a
              href="https://github.com/dasunNimantha/bolt"
              target="_blank"
              rel="noopener noreferrer"
              className="flex items-center gap-2 text-sm text-text-muted hover:text-text-primary transition-colors"
            >
              <Github className="w-4 h-4" />
              Source
            </a>
            <a
              href="https://github.com/dasunNimantha/bolt/issues"
              target="_blank"
              rel="noopener noreferrer"
              className="flex items-center gap-1.5 text-sm text-text-muted hover:text-text-primary transition-colors"
            >
              Issues
              <ExternalLink className="w-3 h-3" />
            </a>
            <a
              href="https://github.com/dasunNimantha/bolt/releases"
              target="_blank"
              rel="noopener noreferrer"
              className="flex items-center gap-1.5 text-sm text-text-muted hover:text-text-primary transition-colors"
            >
              Releases
              <ExternalLink className="w-3 h-3" />
            </a>
            <Link
              to="/privacy"
              className="text-sm text-text-muted hover:text-text-primary transition-colors"
            >
              Privacy
            </Link>
          </div>
        </div>
      </div>
    </footer>
  );
}
