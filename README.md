# Spotify TUI

![Continuous Integration](https://github.com/thewulf7/spotify-tui/workflows/Continuous%20Integration/badge.svg?branch=master&event=push)
![License](https://img.shields.io/badge/license-MIT-blueviolet.svg)
![Release](https://img.shields.io/github/v/release/thewulf7/spotify-tui?color=%23c694ff)

A Spotify client for the terminal written in Rust.

**Maintained fork of [Rigellute/spotify-tui](https://github.com/Rigellute/spotify-tui)** (original archived 2021).

Key improvements:
- **Zero-setup PKCE auth** — no Client Secret, uses ncspot client ID which has Extended Quota Mode
- **Integrated streaming** — spt registers itself as a Spotify Connect speaker via embedded librespot; play audio directly, no daemon needed
- **TrueColor album art** — half-block `▀` rendering with 24-bit RGB in playbar and as a banner above album/playlist track lists
- **Fixed API schema drift** — `available_markets`, `popularity`, `external_ids`, null image arrays all handled
- **Fixed PKCE token refresh** — bypasses rspotify's broken `client_credentials` refresh
- **Rate-limit friendly** — removed dead API calls; adaptive poll (5s playing / 30s paused)
- **Modern CI/CD** — `dtolnay/rust-toolchain`, `Swatinem/rust-cache@v2`, `softprops/action-gh-release@v2`

---

## Installation

The binary is `spt`.

### Download prebuilt binary

Grab the latest release from the [releases page](https://github.com/thewulf7/spotify-tui/releases):

```bash
# macOS
tar xzf spotify-tui-macos.tar.gz && mv spt ~/.local/bin/spt

# Linux
tar xzf spotify-tui-linux.tar.gz && mv spt ~/.local/bin/spt
```

### Build from source

Requires Rust 1.70+.

```bash
git clone https://github.com/thewulf7/spotify-tui
cd spotify-tui
cargo build --release
cp target/release/spt ~/.local/bin/spt
```

Linux deps: `pkg-config libssl-dev libxcb1-dev libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev`

---

## Authentication

spt uses **PKCE OAuth** with the ncspot client ID — no Client Secret needed, no Spotify Developer account required.

### Setup

Edit `~/.config/spotify-tui/client.yml`:

```yaml
client_id: d420a117a32841c2b3474932e49fb54b
client_secret: ""
device_id: ~
port: 8888
```

> This is the well-known ncspot/spotify-player community client ID which has Extended Quota Mode. Personal Spotify Developer apps in Development Mode 403 on playlist endpoints by default.

### First run

Run `spt` — it opens your browser for the Spotify permission screen. After approving, the token is cached and silently refreshed on every subsequent launch.

The redirect URI used is `http://127.0.0.1:8888/callback` (automatically handled — no manual copy-paste needed).

---

## Streaming device

When you run `spt`, it registers itself as a Spotify Connect speaker named **"spt"**.

- Press **`d`** to open the device picker and select "spt"
- Audio plays through your system's default output (librespot + rodio/CoreAudio)
- No spotifyd or separate daemon needed

---

## Album art

Album covers render using **Unicode half-block `▀` characters with 24-bit TrueColor** — each character holds two vertically stacked RGB pixels.

- **Playbar** — square art in the bottom now-playing bar
- **Album/playlist pages** — full-color banner above the track list

Requires a TrueColor terminal (iTerm2, Kitty, macOS Terminal ≥ 10.12, most modern Linux terminals).

---

## Usage

```
spt                              # launch the TUI
spt --help                       # CLI reference

spt play --name "Clancy" --album        # play an album
spt play --name "Jumpsuit" --track      # play a track
spt playback --toggle                   # play/pause
spt playback --like --shuffle           # like + shuffle
spt list --liked --limit 50            # list liked songs
spt search "twenty one pilots" --tracks # search
```

Press `?` inside the TUI for a full keybinding reference.

---

## Configuration

`~/.config/spotify-tui/config.yml`:

```yaml
theme:
  active: Cyan
  banner: LightCyan
  error_border: Red
  error_text: LightRed
  hint: Yellow
  hovered: Magenta
  inactive: Gray
  playbar_background: Black
  playbar_progress: LightCyan
  playbar_progress_text: Cyan
  playbar_text: White
  selected: LightCyan
  text: "255, 255, 255"
  header: White

behavior:
  seek_milliseconds: 5000
  volume_increment: 10
  tick_rate_milliseconds: 250
  enable_text_emphasis: true
  show_loading_indicator: true
  enforce_wide_search_bar: false
  liked_icon: ♥
  shuffle_icon: 🔀
  repeat_track_icon: 🔂
  repeat_context_icon: 🔁
  playing_icon: ▶
  paused_icon: ⏸
  set_window_title: true

keybindings:
  back: "ctrl-q"
  jump_to_album: "a"
  jump_to_artist_album: "A"
  manage_devices: "d"
  decrease_volume: "-"
  increase_volume: "+"
  toggle_playback: " "
  seek_backwards: "<"
  seek_forwards: ">"
  next_track: "n"
  previous_track: "p"
  copy_song_url: "c"
  copy_album_url: "C"
  help: "?"
  shuffle: "ctrl-s"
  repeat: "r"
  search: "/"
  audio_analysis: "v"
  jump_to_context: "o"
  basic_view: "B"
  add_item_to_queue: "z"
```

---

## What's different from the original

| | Original (Rigellute) | This fork |
|---|---|---|
| Auth | Client ID + Secret, manual redirect | PKCE, ncspot client ID (Extended Quota) |
| Streaming | External spotifyd daemon | Built-in librespot device |
| Album art | None | TrueColor half-block rendering |
| Token refresh | Broken (S256 error) | Fixed PKCE refresh |
| Rate limits | Frequent 403/429 spam | Removed dead API calls |
| CI/CD | Abandoned actions-rs | Modern toolchain |
| Maintenance | Archived (2021) | Active |

---

## Libraries used

- [tui-rs](https://github.com/fdehau/tui-rs)
- [rspotify](https://github.com/ramsayleung/rspotify)
- [librespot](https://github.com/librespot-org/librespot)
- [image](https://crates.io/crates/image)

---

## License

MIT OR Apache-2.0
