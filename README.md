# Nyx TV

Fork of [Fredolx/open-tv](https://github.com/Fredolx/open-tv), licensed under GPL-2.0.

Nyx TV is an IPTV desktop application for managing playlists, favorites, and
custom channels. Video playback and recording use mpv and ffmpeg.

## Features

- Import IPTV channels from M3U files, M3U links, or Xtream sources
- Record while watching
- Manage multiple IPTV sources
- Navigate with a keyboard or TV remote
- Search channels and organize favorites
- Create and share custom channels
- Restream channels to other devices

## Prerequisites

Nyx TV uses mpv, ffmpeg, and yt-dlp. Install them with your platform's package
manager:

```sh
brew install mpv ffmpeg yt-dlp
sudo dnf install mpv ffmpeg yt-dlp
sudo zypper install mpv ffmpeg yt-dlp
sudo pacman -Syu mpv ffmpeg yt-dlp
sudo apt install mpv ffmpeg yt-dlp
scoop install mpv ffmpeg yt-dlp
```

Windows installers bundle the required playback tools.

## Build from source

Install Rust and Dioxus CLI 0.7.10:

```sh
cargo install dioxus-cli --version 0.7.10 --locked
```

On Debian or Ubuntu, install the desktop build dependencies:

```sh
sudo apt install -y pkg-config libwebkit2gtk-4.1-dev libgtk-3-dev libxdo-dev \
  libayatana-appindicator3-dev librsvg2-dev libssl-dev build-essential
```

Run the desktop app or create distributable bundles from `desktop/`:

```sh
cd desktop
dx serve --platform desktop
dx bundle --platform desktop --release
```

The desktop interface uses components adapted from
[rust-ui/ui](https://github.com/rust-ui/ui), licensed under MIT.

## Docker

For Nvidia GPUs:

```sh
docker run --rm -it \
  --net=host \
  --env="DISPLAY" \
  --volume="/tmp/.X11-unix:/tmp/.X11-unix:rw" \
  --volume="$HOME/.Xauthority:/root/.Xauthority:rw" \
  --volume="$HOME/.local/share/nyx-tv:/root/.local/share/nyx-tv" \
  --gpus all \
  ghcr.io/0xti4n/nyx-tv:latest
```

For Intel and AMD GPUs:

```sh
docker run --rm -it \
  --net=host \
  --env="DISPLAY" \
  --volume="/tmp/.X11-unix:/tmp/.X11-unix:rw" \
  --volume="$HOME/.Xauthority:/root/.Xauthority:rw" \
  --volume="$HOME/.local/share/nyx-tv:/root/.local/share/nyx-tv" \
  --device /dev/dri \
  ghcr.io/0xti4n/nyx-tv:latest
```

## Feedback

Report bugs or suggest improvements by opening an issue in this repository.

## Hotkeys

- F1: Help
- Ctrl+A: Show all channels
- Ctrl+S: Show categories
- Ctrl+D: Show favorites
- Ctrl+F: Search
- Ctrl+Q: Enable or disable livestreams
- Ctrl+W: Enable or disable movies
- Ctrl+E: Enable or disable series
- Backspace or Escape: Go back
- Arrow keys, Tab, and Shift+Tab: Navigate
- Ctrl+= / Ctrl+- / Ctrl+0: Zoom in, zoom out, or reset zoom

## Settings

### Stream caching

Enable stream caching if a slow internet connection or IPTV provider causes
frequent pauses. Disable it if streams drop completely, if you need lower
latency, or if caching causes playback issues.
