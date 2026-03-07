# Bolt

Fast multi-threaded download manager for Linux, Windows and macOS. Built with Rust.

## Features

- **Multi-segment downloading** -- splits files into up to 8 parallel segments for maximum throughput
- **Pause, resume & persistence** -- stop and continue downloads without losing progress; state survives restarts
- **Speed limiting & concurrency** -- configurable bandwidth cap (KB/s) and simultaneous download limit (1–10)
- **Download scheduling** -- set a daily time window to auto-start all queued downloads
- **System tray & background mode** -- minimizes to tray on close; live tooltip with speed/status
- **Auto-resume on reconnect** -- detects network recovery and retries failed downloads automatically
- **Search & history** -- filter downloads by name or URL; completed downloads are tracked in persistent history
- **Batch downloads** -- paste multiple URLs or import from a text file to queue downloads in bulk
- **Smart queue management** -- auto-start queued downloads when slots open, auto-retry failed segments
- **Browser integration** -- Chrome extension intercepts downloads and sends them to Bolt with a confirmation popup
- **Dark / Light / System theme** -- choose between dark, light, or auto-follow OS theme
- **Multi-window popups** -- browser-intercepted downloads open in their own always-on-top dialog window
- **Cross-platform** -- runs on Linux (Wayland + X11), Windows and macOS with native file dialogs

### Coming soon

- **Proxy support** -- HTTP, HTTPS and SOCKS5 proxy with authentication and connection testing (implemented, UI hidden pending stabilization)

## Browser Integration

Bolt can intercept downloads from Chrome/Chromium browsers. Three components work together:

1. **Chrome extension** (`extension/`) -- intercepts `chrome.downloads.onCreated`, cancels the browser download, captures cookies and referrer, and sends the request to the native messaging host
2. **Native messaging host** (`bolt-nmh`) -- a small Rust binary that bridges Chrome's native messaging protocol (stdin/stdout with 4-byte length-prefixed JSON) to Bolt's TCP IPC server
3. **IPC server** (inside Bolt) -- listens on `localhost:9817` for JSON download requests; each incoming download opens its own popup window for confirmation

### Setup

```bash
# Build the native messaging host
cargo build --release -p bolt-nmh

# Install the native messaging host manifest (Linux)
cd bolt-nmh && ./install.sh

# Load the extension in Chrome
# 1. Open chrome://extensions
# 2. Enable Developer Mode
# 3. Click "Load unpacked" and select the extension/ directory
```

## Building

### Prerequisites

- Rust 1.88+ (install via [rustup](https://rustup.rs))

**Linux** (Wayland or X11):

```bash
# Debian/Ubuntu
sudo apt install pkg-config libssl-dev libfontconfig-dev \
  libgtk-3-dev libayatana-appindicator3-dev libxdo-dev

# Fedora
sudo dnf install pkg-config openssl-devel fontconfig-devel \
  gtk3-devel libayatana-appindicator-gtk3-devel libxdo-devel

# Arch
sudo pacman -S pkg-config openssl fontconfig gtk3 libayatana-appindicator libxdo
```

**Windows**: No extra system dependencies -- just Rust via `rustup`.

**macOS**: No extra system dependencies -- just Rust via `rustup`.

### Build and run

```bash
# Debug build
cargo run

# Release build (optimized)
cargo build --workspace --release

# Linux / macOS
./target/release/bolt

# Windows
.\target\release\bolt.exe
```

## Usage

1. Paste a download URL into the input bar and click **Add**
2. The file info is fetched and the download appears in the queue
3. Click the play button to start downloading
4. Use pause/resume/cancel buttons to control active downloads
5. Completed files can be opened or their folder revealed
6. Open **Settings** (gear icon) to configure speed limits, concurrency, theme, and download directory
7. Close the window while downloads are active -- the app minimizes to the system tray
8. Right-click the tray icon to show the window or quit

## Architecture

```
src/
├── main.rs              # Entry point, iced daemon config
├── app.rs               # Application state and message handling
├── message.rs           # Message enum (Elm architecture)
├── model.rs             # Data structures (DownloadItem, PendingDownload, etc.)
├── settings.rs          # Persistent settings and download database (JSON)
├── theme.rs             # Color scheme and widget styles (closure-based)
├── tray.rs              # System tray icon, menu, and event polling
├── ipc.rs               # TCP IPC server (localhost:9817) for browser integration
├── view.rs              # UI layout and rendering (multi-window dispatch)
├── lib.rs               # Module declarations
├── download/
│   ├── mod.rs           # Download module
│   ├── engine.rs        # Download engine (queue, segments, state, persistence)
│   └── worker.rs        # Segment worker (HTTP streaming, retry, throttling, I/O)
└── utils/
    ├── mod.rs           # Utils module
    └── format.rs        # Byte/speed/ETA formatting

bolt-nmh/                    # Native messaging host binary (workspace member)
├── Cargo.toml
├── com.bolt.nmh.json.template   # Chrome native messaging host manifest
├── install.sh               # Linux install script
└── src/main.rs              # Chrome native messaging ↔ Bolt IPC bridge

extension/                   # Chrome extension (Manifest V3)
├── manifest.json
├── background.js            # Download interception, cookie capture, native messaging
├── popup.html + popup.js    # Toggle on/off, connection status
└── icons/                   # SVG icons
```

## Dependencies

| Crate | Purpose |
|-------|---------|
| `iced` 0.14 | GUI framework (Elm architecture, multi-window daemon) |
| `iced_fonts` 0.3 | Bootstrap icon font |
| `tokio` | Async runtime |
| `reqwest` | HTTP client with streaming |
| `serde` / `serde_json` | Settings and download state serialization |
| `tray-icon` | Cross-platform system tray |
| `chrono` | Date/time for scheduling |
| `uuid` | Unique download identifiers |
| `rfd` | Native file dialogs |
| `image` | App icon loading |
| `directories` | Platform config/data paths |
| `anyhow` | Error handling |

## License

MIT -- see [LICENSE](LICENSE) for details.
