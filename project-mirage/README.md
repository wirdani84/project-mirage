# Project Mirage 🌟

**Cross-OS Distributed Window Interaction System**

## Overview

Project Mirage enables seamless interaction with application windows across two physically independent laptops running different operating systems. Drag a window from your Linux laptop to your Windows laptop and continue using it as if it were local.

### Key Features

- 🖱️ **Shared Mouse Control**: Move your cursor seamlessly between devices
- 🪟 **Live Window Streaming**: Mirror and interact with windows across devices
- ⌨️ **Independent Keyboards**: Each device maintains its own keyboard
- 🔒 **Secure Communication**: Encrypted TLS/DTLS channels
- ⚡ **Low Latency**: Hardware-accelerated encoding/decoding, <50ms end-to-end

## Architecture

```
┌─────────────────┐       ┌─────────────────┐
│  Linux Host     │◄─────►│  Windows Peer   │
│  (Mouse Owner)  │       │  (Own Keyboard) │
└─────────────────┘       └─────────────────┘
```

### System Components

1. **Discovery & Pairing**: mDNS-based device discovery with TLS authentication
2. **Input Coordination**: Mouse/keyboard event forwarding via WebRTC data channels
3. **Window Capture**: Wayland/PipeWire capture with hardware encoding
4. **Stream Transport**: WebRTC media channels for low-latency video
5. **Session Management**: Control plane for window lifecycle and layout

## Technology Stack

| Component | Technology |
|-----------|-----------|
| **Core Language** | Rust (for performance & safety) |
| **Linux Display** | Wayland, PipeWire, libinput |
| **Windows Display** | Win32 API, DirectX |
| **Streaming** | WebRTC (libdatachannel) |
| **Video Encoding** | FFmpeg (VAAPI/NVENC) |
| **Control Protocol** | gRPC over QUIC |
| **Security** | TLS 1.3, DTLS-SRTP |

## Development Roadmap

- [x] Phase 0.0: Project setup and architecture
- [ ] Phase 0.1: Mouse sharing & control link
- [ ] Phase 0.2: Window capture & stream (Linux → Windows)
- [ ] Phase 0.3: Input feedback loop
- [ ] Phase 0.4: GUI control panel & layout manager
- [ ] Phase 0.5: Bi-directional streaming (Windows → Linux)
- [ ] Phase 0.6+: Clipboard sync, file drag, optimizations

## Building

### Prerequisites

**Linux Host:**
```bash
# Ubuntu/Debian
sudo apt install build-essential cmake libwayland-dev libinput-dev \
    libpipewire-0.3-dev libavcodec-dev libavformat-dev libssl-dev \
    protobuf-compiler qtbase5-dev

# Fedora
sudo dnf install gcc-c++ cmake wayland-devel libinput-devel \
    pipewire-devel ffmpeg-devel openssl-devel protobuf-compiler qt5-qtbase-devel
```

**Windows Peer:**
- Visual Studio 2022 with C++ Desktop Development
- CMake 3.20+
- vcpkg for dependencies

### Build Instructions

```bash
# Clone and build
git clone https://github.com/yourusername/project-mirage.git
cd project-mirage

# Build Linux host
cd linux-host
mkdir build && cd build
cmake ..
make -j$(nproc)

# Build Windows peer (in PowerShell)
cd windows-peer
mkdir build && cd build
cmake ..
cmake --build . --config Release
```

## Usage

### 1. Start the Linux Host Daemon

```bash
./mirage-host --discover
```

### 2. Start the Windows Peer Agent

```powershell
.\mirage-peer.exe --scan
```

### 3. Pair Devices

- Both devices will auto-discover on LAN
- Confirm pairing code on both screens
- Encryption keys will be exchanged

### 4. Enable Mouse Sharing

- Move mouse to the edge of your Linux screen
- Cursor will seamlessly transition to Windows laptop
- Move back to regain control on Linux

### 5. Drag Windows Between Devices

- Drag a window to the screen edge
- Window will appear on the other laptop
- Click and type to interact with it remotely

## Configuration

Edit `~/.config/mirage/config.toml`:

```toml
[host]
name = "my-linux-laptop"
display_edge_threshold = 10  # pixels

[network]
discovery_port = 5353
control_port = 8443
allowed_subnets = ["192.168.1.0/24"]

[streaming]
max_fps = 60
codec = "h264"  # h264, h265, av1
bitrate_mbps = 10

[security]
require_pairing = true
session_timeout_minutes = 60
```

## Security Considerations

- All communication is encrypted (TLS 1.3 / DTLS-SRTP)
- Mutual authentication required before pairing
- No remote code execution - only pixel and input data
- Optional per-window permission prompts
- LAN-only by default (TURN relay optional for WAN)

## Performance Targets

| Metric | Target |
|--------|--------|
| Mouse latency | <10ms (LAN) |
| Frame latency | <50ms end-to-end |
| Frame rate | 60fps adaptive |
| CPU overhead | <10% idle |

## Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

MIT License - see [LICENSE](LICENSE) for details.

## Acknowledgments

Built with inspiration from:
- Barrier/Synergy (input sharing)
- Sunshine/Moonlight (game streaming)
- Wayland/wlroots (compositor architecture)
- WebRTC (real-time communication)

---

**Status**: 🚧 Active Development - Phase 0.1

*Built with ❤️ for seamless multi-device workflows*
