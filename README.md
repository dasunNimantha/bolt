# Bolt

Fast multi-threaded download manager for Linux, Windows and macOS. Built with Rust.

## Features

- **Multi-segment downloading** -- splits files into up to 8 parallel segments for maximum throughput
- **Pause / Resume** -- stop and continue downloads without losing progress
- **Download persistence** -- saves download state to disk and restores on restart
- **Speed limiting** -- configurable bandwidth cap (KB/s) with smooth per-chunk throttling
- **Concurrent download limits** -- control how many downloads run simultaneously (1–10)
- **Download scheduling** -- schedule downloads to start at a specific date and time
- **System tray** -- minimizes to tray on close when downloads are active; live tooltip with speed/status
- **Auto-retry** -- failed segments retry automatically with exponential backoff (up to 3 attempts)
- **Auto-start queued** -- queued downloads start automatically when a slot opens
- **Queue management** -- add links without starting them immediately, start when ready
- **Connection optimization** -- aggressive keep-alive, connection pooling, TCP nodelay
- **Buffered I/O** -- 256KB write batching for fewer syscalls and higher disk throughput
- **File type detection** -- automatic categorization (video, audio, document, archive, image, app)
- **Dark / Light theme** -- toggle between themes in settings
- **Settings page** -- grouped, card-based settings UI for downloads, appearance, and preferences
- **Configurable download directory** -- pick any folder via native file dialog
- **Cross-platform** -- runs on Linux, Windows and macOS

## Building

### Prerequisites

- Rust 1.70+ (install via [rustup](https://rustup.rs))

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
cargo build --release

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
├── main.rs              # Entry point, iced app config
├── app.rs               # Application state and message handling
├── message.rs           # Message enum (Elm architecture)
├── model.rs             # Data structures (DownloadItem, SpeedTracker, etc.)
├── settings.rs          # Persistent settings and download database (JSON)
├── theme.rs             # Color scheme and widget styles
├── tray.rs              # System tray icon, menu, and event polling
├── view.rs              # UI layout and rendering
├── lib.rs               # Module declarations
├── download/
│   ├── mod.rs           # Download module
│   ├── engine.rs        # Download engine (queue, segments, state, persistence)
│   └── worker.rs        # Segment worker (HTTP streaming, retry, throttling, I/O)
└── utils/
    ├── mod.rs           # Utils module
    └── format.rs        # Byte/speed/ETA formatting
```

## Dependencies

| Crate | Purpose |
|-------|---------|
| `iced` | GUI framework (Elm architecture) |
| `iced_aw` | Additional widgets and icon fonts |
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
