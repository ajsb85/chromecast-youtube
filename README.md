# Google Cast & Nest Display Caster in Rust

A lightweight command-line utility written in Rust that discovers Google Cast devices (Chromecast and Nest smart displays) on your local network, extracts direct progressive media stream URLs from YouTube videos, channels, or playlists using `yt-dlp`, and casts them sequentially.

## Features

- 📶 **mDNS Auto-Discovery**: Scans the local network for Google Cast devices using Multicast DNS (`_googlecast._tcp.local.`).
- 🖥️ **Smart Display Prioritization**: Intelligently parses device Friendly Names (`fn`) and Model Names (`md`) to auto-detect and target Nest Hubs and Google Home displays.
- 🔀 **YouTube Channel & Playlist Support**: Automatically extracts all video links from channel and playlist URLs (e.g. `https://www.youtube.com/@anylist-app`) and schedules sequential casting.
- 🔄 **Sequential Playback State Tracking**: Connects to the Cast device, listens to media status messages, and automatically resolves/casts the next video in the queue when the current one finishes.
- 🏎️ **Optimized Progressive MP4 Streams**: Configures `yt-dlp` to extract pre-merged progressive MP4 stream formats containing both video and audio, preventing adaptive HLS playback failures on the default media receiver.
- 🐳 **Dynamic MIME Content Types**: Automatically handles different stream container formats (like progressive `.mp4` or HLS `.m3u8` playlists) dynamically at load time.
- 💓 **Session Heartbeat Keep-Alive**: Includes an event loop that automatically replies to Keep-Alive heartbeat PINGs, ensuring your casting sessions remain active.

## Prerequisites

To cast YouTube videos, playlists, or channels, you must have `yt-dlp` (recommended) or `youtube-dl` installed and available in your `PATH`.

### Installing `yt-dlp`
On Debian/Ubuntu:
```bash
sudo apt update
sudo apt install yt-dlp
```
Via python pip:
```bash
pip install -U yt-dlp
```

## Getting Started

### Installation & Compilation
Clone the repository and compile the binary in release mode:
```bash
git clone https://github.com/<your-username>/chromecast-youtube.git
cd chromecast-youtube
cargo build --release
```
The compiled binary will be available at `./target/release/chromecast`.

### Usage

Cast a single YouTube video (auto-detects display):
```bash
./chromecast https://www.youtube.com/watch?v=XShbT8oXGys
```

Cast an entire YouTube channel or playlist sequentially:
```bash
./chromecast https://www.youtube.com/@anylist-app
```

Bypass mDNS discovery and target a specific Cast device IP directly:
```bash
./chromecast -a 192.168.68.101 https://www.youtube.com/watch?v=XShbT8oXGys
```

Customize the mDNS scanning timeout (default: 3 seconds):
```bash
./chromecast -t 5 https://www.youtube.com/watch?v=XShbT8oXGys
```

## CLI Help Options
To print all command-line arguments:
```bash
./chromecast --help
```

## License

This project is licensed under the MIT License. See [LICENSE](LICENSE) for details.
