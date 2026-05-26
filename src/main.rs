use std::time::{Duration, Instant};
use std::process::Command;
use clap::Parser;
use anyhow::{anyhow, Result};
use mdns_sd::{ServiceDaemon, ServiceEvent};
use rust_cast::{
    CastDevice, ChannelMessage,
    channels::{
        heartbeat::HeartbeatResponse,
        media::{Media, StreamType, PlayerState, IdleReason, MediaResponse},
        receiver::CastDeviceApp,
    },
};

const SERVICE_TYPE: &str = "_googlecast._tcp.local.";
const DEFAULT_DESTINATION_ID: &str = "receiver-0";

#[derive(Parser, Debug)]
#[command(name = "chromecast-youtube")]
#[command(about = "Casts a YouTube or direct MP4 video/playlist to a Google Cast device / Nest smart display.", long_about = None)]
struct Cli {
    /// The YouTube URL, channel URL, playlist URL, or direct MP4 link
    #[arg(required = true)]
    url: String,

    /// Optional IP address of the target Cast device (bypasses mDNS discovery)
    #[arg(short, long)]
    address: Option<String>,

    /// Optional port of the target Cast device
    #[arg(short, long, default_value_t = 8009)]
    port: u16,

    /// Set mDNS discovery timeout in seconds
    #[arg(short, long, default_value_t = 3)]
    timeout: u64,
}

#[derive(Debug, Clone)]
struct DiscoveredDevice {
    friendly_name: String,
    model_name: String,
    ip: String,
    port: u16,
}

/// Discover Google Cast devices using mDNS
fn discover_devices(timeout: Duration) -> Result<Vec<DiscoveredDevice>> {
    let mdns = ServiceDaemon::new().map_err(|e| anyhow!("Failed to create mDNS daemon: {}", e))?;
    let receiver = mdns
        .browse(SERVICE_TYPE)
        .map_err(|e| anyhow!("Failed to browse mDNS services: {}", e))?;

    let mut devices = Vec::new();
    let start_time = Instant::now();

    println!("Scanning local network for Google Cast / Nest smart displays...");

    while start_time.elapsed() < timeout {
        if let Ok(event) = receiver.recv_timeout(Duration::from_millis(100)) {
            if let ServiceEvent::ServiceResolved(info) = event {
                let friendly_name = info
                    .get_property_val_str("fn")
                    .unwrap_or("Unknown Device")
                    .to_string();
                let model_name = info
                    .get_property_val_str("md")
                    .unwrap_or("Unknown Model")
                    .to_string();

                let addresses = info.get_addresses();
                let ip_addr = addresses
                    .iter()
                    .map(|addr| addr.to_string())
                    .find(|ip| ip.contains('.'))
                    .or_else(|| addresses.iter().next().map(|addr| addr.to_string()));

                if let Some(ip) = ip_addr {
                    let device = DiscoveredDevice {
                        friendly_name,
                        model_name,
                        ip,
                        port: info.get_port(),
                    };

                    if !devices.iter().any(|d: &DiscoveredDevice| d.ip == device.ip) {
                        println!(
                            "  Found device: \x1b[32m{}\x1b[0m (Model: \x1b[33m{}\x1b[0m, IP: {})",
                            device.friendly_name, device.model_name, device.ip
                        );
                        devices.push(device);
                    }
                }
            }
        }
    }

    Ok(devices)
}

/// Select a Cast device targeting Nest display or Google Home smart displays
fn select_target_device(devices: &[DiscoveredDevice]) -> Option<DiscoveredDevice> {
    if devices.is_empty() {
        return None;
    }

    // Prioritize displays by searching for Nest Hub or displays
    for device in devices {
        let name_lower = device.friendly_name.to_lowercase();
        let model_lower = device.model_name.to_lowercase();

        if model_lower.contains("nest")
            || model_lower.contains("hub")
            || model_lower.contains("display")
            || name_lower.contains("nest")
            || name_lower.contains("hub")
            || name_lower.contains("display")
            || model_lower.contains("google home")
            || name_lower.contains("google home")
        {
            println!(
                "Target Nest display auto-detected: \x1b[32m{}\x1b[0m (Model: \x1b[33m{}\x1b[0m)",
                device.friendly_name, device.model_name
            );
            return Some(device.clone());
        }
    }

    // Default to first found device
    let first = &devices[0];
    println!(
        "No specific Nest display matched. Selecting first discovered device: \x1b[32m{}\x1b[0m",
        first.friendly_name
    );
    Some(first.clone())
}

/// Extract direct progressive media stream URL using yt-dlp (with fallback to youtube-dl)
fn get_youtube_stream_url(url: &str) -> Result<String> {
    // If it doesn't look like a YouTube URL, just return it as a direct URL
    let is_youtube = url.contains("youtube.com") || url.contains("youtu.be");
    if !is_youtube {
        println!("Direct media URL provided. Bypassing stream extraction.");
        return Ok(url.to_string());
    }

    println!("Extracting direct video stream URL from YouTube using yt-dlp...");

    let output = Command::new("yt-dlp")
        .args(&["-g", "-f", "22/18/best[ext=mp4]/best", url])
        .output();

    let output = match output {
        Ok(out) => out,
        Err(_) => {
            println!("  yt-dlp is not available. Falling back to youtube-dl...");
            Command::new("youtube-dl")
                .args(&["-g", "-f", "22/18/best[ext=mp4]/best", url])
                .output()
                .map_err(|_| anyhow!(
                    "Neither 'yt-dlp' nor 'youtube-dl' was found. Please install 'yt-dlp' to cast YouTube links."
                ))?
        }
    };

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(anyhow!("Stream extraction failed: {}", stderr));
    }

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let extracted_url = stdout.lines().next().unwrap_or(&stdout).to_string();

    if extracted_url.is_empty() {
        return Err(anyhow!("Extracted stream URL is empty."));
    }

    println!("  Stream URL successfully extracted.");
    Ok(extracted_url)
}

/// Resolve channel or playlist video URLs using yt-dlp
fn get_playlist_urls(url: &str) -> Result<Vec<String>> {
    let is_playlist_or_channel = url.contains("/playlist")
        || url.contains("/@")
        || url.contains("/channel/")
        || url.contains("/c/")
        || url.contains("/user/")
        || url.contains("list=");

    if !is_playlist_or_channel {
        return Ok(vec![url.to_string()]);
    }

    println!("Playlist/Channel URL detected. Fetching all video links using yt-dlp...");

    let output = Command::new("yt-dlp")
        .args(&["--flat-playlist", "--print", "%(url)s", url])
        .output();

    let output = match output {
        Ok(out) => out,
        Err(_) => {
            Command::new("youtube-dl")
                .args(&["--flat-playlist", "--print", "%(url)s", url])
                .output()
                .map_err(|e| anyhow!("Failed to execute 'yt-dlp' or 'youtube-dl': {}", e))?
        }
    };

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(anyhow!("Failed to fetch playlist video links: {}", stderr));
    }

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let urls: Vec<String> = stdout
        .lines()
        .map(|line| line.trim().to_string())
        .filter(|line| !line.is_empty() && line.starts_with("http"))
        .collect();

    if urls.is_empty() {
        println!("  No video links resolved via flat-playlist. Using input URL as fallback.");
        return Ok(vec![url.to_string()]);
    }

    println!("  Successfully fetched {} video links.", urls.len());
    Ok(urls)
}

fn main() -> Result<()> {
    let args = Cli::parse();

    // 1. Resolve Cast Device (Discovery or direct argument)
    let (ip, port) = match args.address {
        Some(addr) => {
            println!("Target Cast device explicitly set to: {}", addr);
            (addr, args.port)
        }
        None => {
            let timeout = Duration::from_secs(args.timeout);
            let devices = discover_devices(timeout)?;
            let chosen = select_target_device(&devices)
                .ok_or_else(|| anyhow!("No Chromecast / Nest display found on local network."))?;
            (chosen.ip, chosen.port)
        }
    };

    // 2. Fetch Playlist/Channel Video URLs
    let playlist_urls = get_playlist_urls(&args.url)?;

    // 3. Connect to the Google Cast Device
    println!("Connecting to Cast device at {}:{}...", ip, port);
    let cast_device = CastDevice::connect_without_host_verification(ip, port)
        .map_err(|e| anyhow!("Failed to establish secure connection to Cast device: {:?}", e))?;

    // Connect to the base receiver channel
    cast_device
        .connection
        .connect(DEFAULT_DESTINATION_ID.to_string())
        .map_err(|e| anyhow!("Failed to establish virtual connection to receiver-0: {:?}", e))?;

    // Send initial ping to confirm socket and channel are open
    cast_device
        .heartbeat
        .ping()
        .map_err(|e| anyhow!("Heartbeat ping failed: {:?}", e))?;

    // 4. Launch Default Media Receiver app
    println!("Launching Default Media Receiver (App ID: CC1AD845)...");
    let app = cast_device
        .receiver
        .launch_app(&CastDeviceApp::DefaultMediaReceiver)
        .map_err(|e| anyhow!("Failed to launch media receiver app: {:?}", e))?;

    println!(
        "  App successfully run: {} (Session ID: {})",
        app.display_name, app.session_id
    );

    // Connect connection channel to our newly started app's transport channel
    cast_device
        .connection
        .connect(app.transport_id.as_str())
        .map_err(|e| anyhow!("Failed to connect to application transport channel: {:?}", e))?;

    // 5. Play each item in the playlist sequentially
    let total_videos = playlist_urls.len();
    for (index, video_url) in playlist_urls.iter().enumerate() {
        println!(
            "\n\x1b[35m=== Playing Video {}/{} ===\x1b[0m",
            index + 1,
            total_videos
        );
        println!("Video Link: \x1b[34m{}\x1b[0m", video_url);

        // Extract direct stream URL
        let stream_url = match get_youtube_stream_url(video_url) {
            Ok(url) => url,
            Err(e) => {
                eprintln!(
                    "\x1b[31mFailed to extract stream for {}: {}. Skipping to next video.\x1b[0m",
                    video_url, e
                );
                continue;
            }
        };

        // Determine content type
        let mut content_type = "video/mp4".to_string();
        if video_url.contains(".mp3") {
            content_type = "audio/mp3".to_string();
        } else if stream_url.contains(".m3u8") || stream_url.contains("index.m3u8") {
            content_type = "application/x-mpegURL".to_string();
        }

        // Load the media
        println!("Casting media stream to display...");
        let load_status = match cast_device.media.load(
            app.transport_id.as_str(),
            app.session_id.as_str(),
            &Media {
                content_id: stream_url,
                content_type,
                stream_type: StreamType::Buffered,
                duration: None,
                metadata: None,
            },
        ) {
            Ok(status) => status,
            Err(e) => {
                eprintln!(
                    "\x1b[31mFailed to load media: {:?}. Skipping to next video.\x1b[0m",
                    e
                );
                continue;
            }
        };

        if let Some(entry) = load_status.entries.first() {
            println!("  Playback started. Player State: \x1b[32m{:?}\x1b[0m", entry.player_state);
        }

        println!("\x1b[36mPlaying video... Press Ctrl+C to stop casting.\x1b[0m");

        // Loop to reply to PING and monitor playback status
        let mut video_finished = false;
        while !video_finished {
            match cast_device.receive() {
                Ok(ChannelMessage::Heartbeat(HeartbeatResponse::Ping)) => {
                    // Reply with PONG to satisfy keep-alive requirements
                    if let Err(e) = cast_device.heartbeat.pong() {
                        eprintln!("Failed to reply with Heartbeat pong: {:?}", e);
                    }
                }
                Ok(ChannelMessage::Media(media_response)) => {
                    if let MediaResponse::Status(status) = media_response {
                        if let Some(entry) = status.entries.first() {
                            match (entry.player_state, entry.idle_reason) {
                                (PlayerState::Idle, Some(IdleReason::Finished)) => {
                                    println!("\x1b[32mVideo finished playing naturally.\x1b[0m");
                                    video_finished = true;
                                }
                                (PlayerState::Idle, Some(IdleReason::Error)) => {
                                    eprintln!("\x1b[31mPlayer reported an error during playback.\x1b[0m");
                                    video_finished = true;
                                }
                                (PlayerState::Idle, Some(IdleReason::Cancelled)) => {
                                    println!("\x1b[33mPlayback cancelled by sender.\x1b[0m");
                                    video_finished = true;
                                }
                                _ => {} // Video is playing, buffering, or paused
                            }
                        }
                    }
                }
                Ok(_) => {} // Ignore other incoming messages
                Err(e) => {
                    eprintln!("Socket error or connection closed: {}", e);
                    return Err(anyhow!("Connection error: {}", e));
                }
            }
        }
    }

    println!("\n\x1b[32mFinished playing all items in the playlist!\x1b[0m");
    Ok(())
}
