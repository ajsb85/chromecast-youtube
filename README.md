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

### System-wide Installation

You can install the compiled utility system-wide to make the `chromecast` command available from anywhere in your shell:

```bash
sudo cp target/release/chromecast /usr/local/bin/chromecast
sudo chmod 755 /usr/local/bin/chromecast
```

Once installed, you can invoke the program directly as `chromecast` without prefixing it with `./target/release/` or `./`.

### Command-Line Arguments & Options

```
Usage: chromecast [OPTIONS] <URLS>...
```

| Argument / Option | Short | Long | Type | Default | Description |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `<URLS>...` | - | - | `Vec<String>` | *Required* | One or more space-separated YouTube videos, channels, playlists, or direct progressive stream links. |
| Target Address | `-a` | `--address` | `String` | `None` | Optional IP address of your Cast device. Bypasses standard mDNS discovery to connect directly. |
| Network Port | `-p` | `--port` | `u16` | `8009` | The TCP port of the Chromecast / Nest display. |
| Discovery Timeout | `-t` | `--timeout` | `u64` | `3` | Maximum time in seconds to scan the local network for cast devices. |
| Loop Playback | `-l` | `--loop-playlist` | `Flag` | `Disabled` | Loops the entire unified media queue infinitely. When all videos finish, casting automatically restarts from the first video. |

---

### Usage & Examples

#### 1. Cast a Single YouTube Video (Auto-Discovery)
Scans the local network, auto-detects Nest displays or Chromecasts, resolves a direct progressive stream, and starts casting:
```bash
chromecast https://www.youtube.com/watch?v=XShbT8oXGys
```

#### 2. Combine Multiple Playlists & Channels into a Single Queue
Pass multiple space-separated links. The caster aggregates all individual videos into a single sequential queue:
```bash
chromecast https://www.youtube.com/playlist?list=PLaGRwR1U7ndDySvz0tY3hJHS05iezhw9r https://www.youtube.com/@anylist-app
```

#### 3. Loop the Merged Queue Infinitely (`-l` / `--loop-playlist`)
Plays all resolved videos from both playlists sequentially. When the final video completes, the queue instantly wraps around and restarts playback from the first item:
```bash
chromecast -l "https://www.youtube.com/playlist?list=PLaGRwR1U7ndDySvz0tY3hJHS05iezhw9r" "https://www.youtube.com/playlist?list=PLaGRwR1U7ndCj2SwBXmOZBlzO7g9jlO1Z"
```

#### 4. Cast to a Specific IP Address Directly (`-a` / `--address`)
Skips mDNS discovery completely and connects directly to the specified IP address:
```bash
chromecast -a 192.168.68.101 https://www.youtube.com/watch?v=XShbT8oXGys
```

#### 5. Adjust mDNS Search Duration (`-t` / `--timeout`)
Allows more or less time (in seconds) to query network devices before selecting a target display:
```bash
chromecast -t 6 https://www.youtube.com/watch?v=XShbT8oXGys
```

---

## CLI Help Interface

To view all arguments, flags, and options from the terminal:
```bash
chromecast --help
```

## License

This project is licensed under the MIT License. See [LICENSE](LICENSE) for details.
