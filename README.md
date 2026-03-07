# Bolt

Fast multi-threaded download manager for Linux, Windows and macOS.Built with Rust.

## Features

- **Multi-segment downloading** -- splits files into up to 8 parallel segments for maximum throughput
- **Pause / Resume** -- stop and continue downloads without losing progress
- **Auto-retry** -- failed segments retry automatically with exponential backoff (up to 3 attempts)
- **Queue management** -- add links without starting them immediately, start when ready
- **Connection optimization** -- aggressive keep-alive, connection pooling, TCP nodelay
- **Buffered I/O** -- 256KB write batching for fewer syscalls and higher disk throughput
- **File type detection** -- automatic categorization (video, audio, document, archive, image, app)
- **Dark / Light theme** -- toggle between themes with a single click
- **Configurable download directory** -- pick any folder via native file dialog
- **Cross-platform** -- runs on Linux, Windows and macOS

## Building

### Prerequisites

- Rust 1.70+ (install via [rustup](https://rustup.rs))

**Linux** (Wayland or X11):

```bash
# Debian/Ubuntu
sudo apt install pkg-config libssl-dev libfontconfig-dev

# Fedora
sudo dnf install pkg-config openssl-devel fontconfig-devel

# Arch
sudo pacman -S pkg-config openssl fontconfig
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

## Architecture

```
src/
├── main.rs              # Entry point, iced app config
├── app.rs               # Application state and message handling
├── message.rs           # Message enum (Elm architecture)
├── model.rs             # Data structures (DownloadItem, SpeedTracker, etc.)
├── settings.rs          # Persistent settings (load/save JSON)
├── theme.rs             # Color scheme and widget styles
├── view.rs              # UI layout and rendering
├── lib.rs               # Module declarations
├── download/
│   ├── mod.rs           # Download module
│   ├── engine.rs        # Download engine (queue, segments, state)
│   └── worker.rs        # Segment worker (HTTP streaming, retry, I/O)
└── utils/
    ├── mod.rs           # Utils module
    └── format.rs        # Byte/speed/ETA formatting
```

## License

MIT -- see [LICENSE](LICENSE) for details.
