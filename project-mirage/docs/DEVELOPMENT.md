# Project Mirage - Development Guide

## Quick Start

### Prerequisites

**Linux Host (Ubuntu/Debian):**
```bash
sudo apt update
sudo apt install -y build-essential cmake pkg-config \
    libwayland-dev libinput-dev libevdev-dev \
    libssl-dev protobuf-compiler curl git

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

**Windows Peer:**
```powershell
# Install Python 3.11+
# Download from: https://www.python.org/downloads/

# Install Visual C++ Build Tools (optional, for native extensions)
# Download from: https://visualstudio.microsoft.com/downloads/
```

### Building from Source

#### Linux Host

```bash
cd project-mirage/linux-host

# Build in development mode
cargo build

# Build optimized release
cargo build --release

# Run tests
cargo test

# Run the daemon
cargo run -- --discover --verbose
```

#### Windows Peer

```powershell
cd project-mirage\windows-peer

# Create virtual environment
python -m venv venv
.\venv\Scripts\activate

# Install dependencies
pip install -r requirements.txt

# Run the peer agent
python mirage_peer.py --scan --verbose
```

## Development Workflow

### Phase 0.1: Mouse Sharing (Current)

**Goal:** Enable basic mouse control between Linux and Windows

**Linux Host Tasks:**
1. ✅ Device discovery via mDNS
2. ✅ Mouse event capture from libinput/evdev
3. ✅ Edge detection for screen boundaries
4. 🚧 Control channel setup (QUIC/WebRTC)
5. 🚧 Mouse event serialization and forwarding
6. 🚧 Input injection via uinput

**Windows Peer Tasks:**
1. ✅ Basic agent structure
2. 🚧 mDNS response and pairing
3. 🚧 Control channel client
4. 🚧 Mouse event deserialization
5. 🚧 Mouse injection via Win32 API
6. 🚧 Keyboard capture and forwarding

**Testing Phase 0.1:**
```bash
# Terminal 1 (Linux):
cd linux-host
cargo run -- --discover --verbose

# Terminal 2 (Windows or Linux for testing):
cd windows-peer
python mirage_peer.py --scan --verbose

# Expected behavior:
# - Both devices discover each other
# - Pairing prompt appears
# - Mouse moves between screens seamlessly
```

### Phase 0.2: Window Streaming (Next)

**Goal:** Stream a Linux window to Windows with visual rendering

**Linux Host Tasks:**
1. Wayland compositor integration (wlr-screencopy)
2. Window capture via PipeWire
3. Hardware video encoding (VAAPI/NVENC)
4. WebRTC media channel setup
5. Frame damage tracking
6. Stream quality adaptation

**Windows Peer Tasks:**
1. WebRTC media channel receiver
2. Hardware video decoding (DXVA/MFT)
3. Borderless window creation
4. DirectX/OpenGL rendering
5. Window positioning and sizing
6. Cursor overlay rendering

### Phase 0.3: Input Feedback Loop

**Goal:** Allow interaction with streamed windows

**Implementation:**
1. Capture local input on Windows peer
2. Forward to Linux host via data channel
3. Inject input into application on Linux
4. Update reflected in video stream

### Phase 0.4: GUI Control Panel

**Goal:** User-friendly configuration interface

**Features:**
- Drag-and-drop display layout editor
- Visual connection status
- Active window list
- Performance metrics
- Security settings

## Architecture Deep-Dive

### Input Handling Pipeline (Phase 0.1)

```
Linux Host:
┌─────────────────────────────────────────┐
│ /dev/input/eventX (evdev)               │
│   ↓                                     │
│ libinput → InputManager                 │
│   ↓                                     │
│ Edge Detection (screen boundaries)      │
│   ↓                                     │
│ Mouse Event Serialization (protobuf)    │
│   ↓                                     │
│ WebRTC Data Channel                     │
└─────────────────────────────────────────┘
              ↓ Network ↓
┌─────────────────────────────────────────┐
│ WebRTC Data Channel                     │
│   ↓                                     │
│ Event Deserialization                   │
│   ↓                                     │
│ Win32 API (SetCursorPos, mouse_event)   │
│   ↓                                     │
│ Windows Desktop                         │
└─────────────────────────────────────────┘
Windows Peer
```

### Window Streaming Pipeline (Phase 0.2)

```
Linux Host:
┌─────────────────────────────────────────┐
│ X11/Wayland Window                      │
│   ↓                                     │
│ PipeWire → DMA-buf                      │
│   ↓                                     │
│ FFmpeg (h264_vaapi encoder)             │
│   ↓                                     │
│ WebRTC Media Channel (RTP/SRTP)         │
└─────────────────────────────────────────┘
              ↓ Network ↓
┌─────────────────────────────────────────┐
│ WebRTC Media Channel (RTP/SRTP)         │
│   ↓                                     │
│ FFmpeg (h264 decoder w/ DXVA)           │
│   ↓                                     │
│ DirectX/OpenGL Texture                  │
│   ↓                                     │
│ Borderless Window (layered)             │
└─────────────────────────────────────────┘
Windows Peer
```

## Debugging

### Linux Host Debugging

```bash
# Enable verbose logging
RUST_LOG=debug cargo run -- --discover

# Monitor input events directly
sudo evtest /dev/input/event5

# Check Wayland compositor
echo $WAYLAND_DISPLAY
weston-info  # or sway-msg, etc.

# Network traffic
sudo tcpdump -i any port 5353 or port 8443

# Check for hardware encoding support
vainfo  # VAAPI
nvidia-smi  # NVENC
```

### Windows Peer Debugging

```powershell
# Enable verbose Python logging
python mirage_peer.py --verbose --scan

# Check network connectivity
Test-NetConnection <linux-ip> -Port 8443

# Monitor input injection
# Use Spy++ from Visual Studio

# Check video decoder capabilities
# Use DXCapsViewer
```

## Performance Tuning

### Low Latency Configuration

```toml
# config.toml
[streaming]
max_fps = 120
codec = "h264"
bitrate_mbps = 20
low_latency_mode = true
buffer_size_ms = 10

[input]
mouse_acceleration = 1.0
edge_activation_delay_ms = 0
```

### High Quality Configuration

```toml
[streaming]
max_fps = 60
codec = "h265"
bitrate_mbps = 50
quality_preset = "slow"

[network]
mtu_size = 1500
enable_jumbo_frames = true
```

## Security Considerations

### TLS Certificate Generation

```bash
# Generate self-signed certificates for testing
openssl req -x509 -newkey rsa:4096 -nodes \
    -keyout ~/.config/mirage/key.pem \
    -out ~/.config/mirage/cert.pem \
    -days 365 -subj "/CN=mirage-host"
```

### Firewall Configuration

```bash
# Linux (ufw)
sudo ufw allow 5353/udp  # mDNS
sudo ufw allow 8443/tcp  # Control channel
sudo ufw allow 49152:65535/udp  # WebRTC media

# Windows (PowerShell as Admin)
New-NetFirewallRule -DisplayName "Mirage mDNS" -Direction Inbound -Protocol UDP -LocalPort 5353 -Action Allow
New-NetFirewallRule -DisplayName "Mirage Control" -Direction Inbound -Protocol TCP -LocalPort 8444 -Action Allow
```

## Contributing

### Code Style

- **Rust:** Follow `rustfmt` and `clippy` recommendations
- **Python:** Follow PEP 8, use `black` formatter
- **Commits:** Conventional commits format

### Testing Requirements

- All new features must include unit tests
- Integration tests for protocol changes
- Performance benchmarks for critical paths

### Pull Request Process

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit changes (`git commit -m 'feat: add amazing feature'`)
4. Push to branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## Troubleshooting

### Common Issues

**Issue:** "Permission denied" when accessing `/dev/input/eventX`

**Solution:**
```bash
# Add user to input group
sudo usermod -a -G input $USER
# Log out and back in
```

**Issue:** mDNS discovery not working

**Solution:**
```bash
# Check if avahi-daemon is running
systemctl status avahi-daemon

# Install if missing
sudo apt install avahi-daemon
```

**Issue:** Video encoding fails

**Solution:**
```bash
# Check available encoders
ffmpeg -encoders | grep h264

# Install VA-API drivers (Intel)
sudo apt install intel-media-va-driver

# Install NVENC drivers (NVIDIA)
# Make sure NVIDIA proprietary drivers are installed
```

## Resources

- [WebRTC Specification](https://www.w3.org/TR/webrtc/)
- [Wayland Protocol](https://wayland.freedesktop.org/docs/html/)
- [PipeWire Documentation](https://docs.pipewire.org/)
- [FFmpeg Encoding Guide](https://trac.ffmpeg.org/wiki/Encode/H.264)
- [Win32 API Reference](https://docs.microsoft.com/en-us/windows/win32/)

## License

MIT License - see LICENSE file for details
