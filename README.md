# Bolt

Fast multi-threaded download manager for Linux, Windows and macOS. Built with Rust and [iced](https://iced.rs).

## Features

- **Multi-segment downloading** -- splits files into up to 8 parallel segments for maximum throughput (auto-scales by file size: 1 segment < 5 MB, 2 < 20 MB, 4 < 50 MB, 6 < 200 MB, 8 for larger)
- **Pause, resume & persistence** -- stop and continue downloads without losing progress; segment state survives app restarts
- **Speed limiting** -- configurable global bandwidth cap in KB/s, distributed evenly across active segments
- **Concurrent downloads** -- control how many downloads run simultaneously (1–10); queued downloads auto-start when slots free up
- **Download scheduling** -- set a daily time window to auto-start queued downloads
- **Proxy support** -- HTTP, HTTPS and SOCKS5 proxy with username/password authentication and one-click connection testing
- **System tray & background mode** -- minimizes to tray on close; live tooltip shows speed and active count; `--background` flag starts hidden
- **Auto-resume on reconnect** -- periodic connectivity check (`generate_204`); retries all failed downloads when network returns
- **File categorization** -- automatic category detection (Video, Audio, Document, Archive, Image, Application) from file extension
- **Download history** -- completed downloads are tracked persistently (up to 500 entries) with filename and URL search
- **Batch downloads** -- paste multiple URLs or import from a text file to queue downloads in bulk
- **Browser integration** -- Chrome/Chromium extension intercepts downloads, captures cookies and referrer, and sends them to Bolt with a confirmation popup
- **Dark / Light / System theme** -- three theme modes with full widget styling
- **Multi-window** -- browser-intercepted downloads each open in their own always-on-top confirmation dialog with Start/Queue/Dismiss
- **Auto-start on login** -- optional system autostart (Linux `.desktop`, macOS `launchd`, Windows registry)
- **Cross-platform** -- Linux (Wayland + X11), Windows, macOS with native file dialogs

## Browser Integration

Bolt intercepts downloads from Chrome/Chromium-based browsers. Three components work together:

1. **Chrome extension** (`extension/`) -- intercepts `chrome.downloads.onCreated`, cancels the browser download, captures cookies and referrer, sends the request to the native messaging host
2. **Native messaging host** (`bolt-nmh`) -- bridges Chrome's native messaging protocol (stdin/stdout, 4-byte length-prefixed JSON) to Bolt's TCP IPC server
3. **IPC server** (inside Bolt) -- listens on `localhost:9817` for JSON download requests; each download opens its own confirmation popup

The extension popup provides toggles for:
- **Intercept Downloads** -- enable/disable download interception
- **Forward Cookies** -- send browser cookies with the download request (requires optional `cookies` permission)

### Setup

```bash
# Build the native messaging host
cargo build --release -p bolt-nmh

# Install the NMH manifest (Linux — auto-detects Chrome, Chromium, Brave, Edge, Vivaldi)
cd bolt-nmh && ./install.sh <your-extension-id>

# Alternatively, Bolt auto-installs/updates the NMH manifest on startup
# if an existing manifest is found (dev mode)
```

Load the extension in Chrome:
1. Open `chrome://extensions`
2. Enable **Developer Mode**
3. Click **Load unpacked** and select the `extension/` directory

## Building

### Prerequisites

- Rust 1.88+ (install via [rustup](https://rustup.rs))

**Linux** (Wayland or X11):

```bash
# Debian/Ubuntu
sudo apt install pkg-config libssl-dev libfontconfig-dev \
  libgtk-3-dev libayatana-appindicator3-dev

# Fedora
sudo dnf install pkg-config openssl-devel fontconfig-devel \
  gtk3-devel libayatana-appindicator-gtk3-devel

# Arch
sudo pacman -S pkg-config openssl fontconfig gtk3 libayatana-appindicator
```

**Windows / macOS**: No extra system dependencies -- just Rust via `rustup`.

### Build and run

```bash
# Debug build
cargo run

# Release build (full workspace: bolt + bolt-nmh)
cargo build --workspace --release

# Run
./target/release/bolt              # Linux / macOS
.\target\release\bolt.exe          # Windows
./target/release/bolt --background # Start minimized to tray
```

### Packaging

Pre-built packages include both `bolt` and `bolt-nmh` binaries:

| Format | Config |
|--------|--------|
| `.deb` | `Cargo.toml` `[package.metadata.deb]` -- package name `bolt-dm` |
| `.rpm` | `Cargo.toml` `[package.metadata.generate-rpm]` |
| AUR | `dist/aur/PKGBUILD` -- package name `bolt-dm-bin` |

```bash
# Build .deb
cargo install cargo-deb
cargo deb

# Build .rpm
cargo install cargo-generate-rpm
cargo generate-rpm
```

## Usage

1. Paste a download URL into the input bar and click **Add**
2. The file info is resolved and the download appears in the queue
3. Click the play button to start downloading
4. Use pause/resume/cancel buttons to control active downloads
5. Completed files can be opened or their folder revealed
6. Open **Settings** (gear icon) to configure speed limits, concurrency, scheduling, theme, proxy, and download directory
7. Close the window -- the app minimizes to the system tray
8. Right-click the tray icon to show the window or quit

## Architecture

```
src/
├── main.rs              # Entry point, iced daemon, font loading
├── app.rs               # Application state, message handling, IPC polling
├── message.rs           # Message enum (Elm architecture)
├── model.rs             # DownloadItem, FileCategory, SpeedTracker, etc.
├── settings.rs          # AppSettings, DownloadDatabase, DownloadHistory, ProxyConfig
├── theme.rs             # ColorScheme, ThemeMode, widget styles
├── tray.rs              # System tray icon, menu, tooltip, event polling
├── ipc.rs               # TCP IPC server (localhost:9817) for browser integration
├── nmh.rs               # NMH manifest auto-install for detected browsers
├── autostart.rs         # Platform autostart (Linux/macOS/Windows)
├── view.rs              # UI layout, settings panel, popup confirmation windows
├── download/
│   ├── engine.rs        # Download engine (queue, segments, concurrency, persistence)
│   └── worker.rs        # Segment worker (HTTP streaming, retry, throttling, I/O)
└── utils/
    └── format.rs        # Byte/speed/ETA/filename formatting

bolt-nmh/                # Native messaging host (workspace member)
├── src/main.rs          # Chrome NMH ↔ Bolt IPC bridge
├── install.sh           # Linux/macOS install script
└── com.bolt.nmh.json.template

extension/               # Chrome extension (Manifest V3)
├── manifest.json        # Permissions: downloads, nativeMessaging, storage
├── background.js        # Download interception, cookie forwarding, NMH messaging
├── popup.html           # Extension popup UI
└── popup.js             # Toggle logic, connection status check
```

## Dependencies

| Crate | Purpose |
|-------|---------|
| `iced` 0.14 | GUI framework (Elm architecture, multi-window, Wayland + X11) |
| `iced_fonts` 0.3 | Bootstrap icon font |
| `tokio` 1 | Async runtime (full features) |
| `futures` 0.3 | Stream utilities for download workers |
| `reqwest` 0.11 | HTTP client (streaming, rustls-tls, SOCKS5 proxy) |
| `serde` / `serde_json` | Settings, download state, and IPC serialization |
| `tray-icon` 0.21 | Cross-platform system tray |
| `chrono` 0.4 | Date/time for scheduling and history |
| `uuid` 1 | Unique download identifiers |
| `rfd` 0.14 | Native file/folder dialogs |
| `image` 0.25 | App icon loading (PNG) |
| `directories` 5.0 | Platform config/data paths (`~/.config/Bolt/`) |
| `url` 2 | URL parsing and validation |
| `anyhow` 1.0 | Error handling |
| `gtk` 0.18 | Linux tray support |

## License

MIT -- see [LICENSE](LICENSE) for details.
