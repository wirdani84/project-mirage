# Getting Started with Project Mirage

Welcome to Project Mirage! This guide will help you get up and running quickly.

## Table of Contents

1. [Overview](#overview)
2. [System Requirements](#system-requirements)
3. [Quick Start](#quick-start)
4. [Installation](#installation)
5. [First Run](#first-run)
6. [Usage Examples](#usage-examples)
7. [Troubleshooting](#troubleshooting)
8. [Next Steps](#next-steps)

---

## Overview

Project Mirage enables you to:

- Share mouse control between Linux and Windows laptops
- Drag application windows between devices (Phase 0.2+)
- Use each device's keyboard independently
- All communication is encrypted

**Current Status:** Phase 0.1 (Mouse Sharing)

---

## System Requirements

### Linux Host

**Minimum:**
- Ubuntu 20.04+ / Fedora 35+ or similar
- 2 GB RAM
- Wayland or X11 display server
- 100 Mbps network connection

**Recommended:**
- Ubuntu 22.04+ / Fedora 38+
- 4 GB RAM
- Wayland (for best window capture)
- 1 Gbps network connection

### Windows Peer

**Minimum:**
- Windows 10 version 1909+
- 2 GB RAM
- Python 3.9+
- 100 Mbps network connection

**Recommended:**
- Windows 11
- 4 GB RAM
- Python 3.11+
- 1 Gbps network connection

### Network

- Both devices must be on the same local network (LAN)
- mDNS/Bonjour must be enabled
- Firewall must allow:
  - UDP port 5353 (mDNS discovery)
  - TCP port 8443 (Linux control channel)
  - TCP port 8444 (Windows control channel)

---

## Quick Start

### Option 1: Automated Setup (Recommended)

```bash
# Clone the repository
git clone https://github.com/yourusername/project-mirage.git
cd project-mirage

# Run quick start (installs deps, builds, runs demo)
make quickstart
```

### Option 2: Manual Setup

See [Installation](#installation) section below.

---

## Installation

### Step 1: Install System Dependencies

#### Ubuntu/Debian

```bash
sudo apt update
sudo apt install -y build-essential cmake pkg-config \
    libwayland-dev libinput-dev libevdev-dev \
    libssl-dev protobuf-compiler \
    python3 python3-pip python3-venv

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

#### Fedora

```bash
sudo dnf install -y gcc-c++ cmake pkg-config \
    wayland-devel libinput-devel libevdev-devel \
    openssl-devel protobuf-compiler \
    python3 python3-pip

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

#### Windows

1. Install Python 3.11+ from [python.org](https://www.python.org/downloads/)
2. (Optional) Install [Visual Studio Build Tools](https://visualstudio.microsoft.com/downloads/)

### Step 2: Build Project

#### Linux Host

```bash
cd project-mirage/linux-host
cargo build --release

# The binary will be at: target/release/mirage-host
```

#### Windows Peer

```powershell
cd project-mirage\windows-peer

# Create virtual environment
python -m venv venv
.\venv\Scripts\activate

# Install dependencies
pip install -r requirements.txt
```

---

## First Run

### Test with Demo

Before running on actual hardware, test with the demo:

```bash
cd project-mirage
python3 demo_phase_01.py
```

You should see output showing mouse moving between virtual screens.

### Run on Real Hardware

#### On Linux Host

```bash
cd project-mirage/linux-host

# Run in discovery mode
cargo run --release -- --discover --verbose

# Or use the binary directly
./target/release/mirage-host --discover --verbose
```

You should see:
```
Project Mirage - Linux Host Daemon v0.1.0
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
✓ Configuration loaded
✓ Node name: your-hostname
✓ Input manager ready
✓ Session manager ready
✓ Discovery service started
 Scanning for peer devices on local network...
```

#### On Windows Peer

```powershell
cd project-mirage\windows-peer

# Activate virtual environment
.\venv\Scripts\activate

# Run the peer agent
python mirage_peer.py --scan --verbose
```

You should see:
```
Project Mirage - Windows Peer Agent
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
✓ Node name: windows-peer
✓ Platform: Windows 10
✓ Peer agent ready
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Phase 0.1: Mouse sharing enabled
  • Waiting for connections from Linux host
```

### Verify Discovery

Both devices should discover each other within a few seconds. You'll see messages like:

**On Linux:**
```
Discovered peer: windows-peer (windows) at 192.168.1.100:8444
```

**On Windows:**
```
 Discovered host: linux-host (linux) at 192.168.1.50:8443
```

---

## Usage Examples

### Basic Mouse Sharing (Phase 0.1)

**Note:** Full implementation in progress. Current functionality:

1. **Start both daemons** as shown above
2. **Move mouse to screen edge** on Linux
3. **Mouse transfers to Windows** (when implemented)
4. **Move back** by going to opposite edge

### Configuration

Edit `~/.config/mirage/config.toml` on Linux:

```toml
[host]
name = "my-linux-laptop"
display_edge_threshold = 10  # pixels from edge to trigger

[network]
discovery_port = 5353
control_port = 8443
allowed_subnets = ["192.168.1.0/24"]

[streaming]
max_fps = 60
codec = "h264"
bitrate_mbps = 10

[security]
require_pairing = true
session_timeout_minutes = 60

[input]
mouse_acceleration = 1.0
enable_smooth_scroll = true
edge_activation_delay_ms = 100
```

### Command Line Options

#### Linux Host

```bash
mirage-host [OPTIONS]

Options:
  -d, --discover              Enable discovery mode
  -c, --config <FILE>         Config file path [default: ~/.config/mirage/config.toml]
  -v, --verbose               Verbose logging
  -n, --name <NAME>           Node name (overrides config)
  -h, --help                  Print help
  -V, --version               Print version
```

#### Windows Peer

```bash
python mirage_peer.py [OPTIONS]

Options:
  --name NAME        Node name
  --scan             Scan for Linux hosts
  --verbose          Verbose logging
  -h, --help         Show help
```

---

## Troubleshooting

### Linux Host Issues

#### "Permission denied" accessing /dev/input/eventX

**Solution:**
```bash
# Add user to input group
sudo usermod -a -G input $USER

# Log out and back in for changes to take effect
```

#### "No mouse device found"

**Solution:**
```bash
# List input devices
ls -l /dev/input/event*

# Test device access
sudo evtest /dev/input/event5

# Check permissions
ls -l /dev/input/event*
```

#### Discovery not working

**Solution:**
```bash
# Check if avahi-daemon is running
systemctl status avahi-daemon

# If not installed:
sudo apt install avahi-daemon  # Ubuntu/Debian
sudo dnf install avahi         # Fedora

# Check firewall
sudo ufw status
sudo ufw allow 5353/udp
```

### Windows Peer Issues

#### Python not found

**Solution:**
```powershell
# Verify Python installation
python --version

# If not found, install from:
# https://www.python.org/downloads/
```

#### Import errors

**Solution:**
```powershell
# Recreate virtual environment
Remove-Item -Recurse -Force venv
python -m venv venv
.\venv\Scripts\activate
pip install --upgrade pip
pip install -r requirements.txt
```

#### Firewall blocking connections

**Solution:**
```powershell
# Run PowerShell as Administrator

# Allow mDNS
New-NetFirewallRule -DisplayName "Mirage mDNS" `
    -Direction Inbound -Protocol UDP -LocalPort 5353 -Action Allow

# Allow control channel
New-NetFirewallRule -DisplayName "Mirage Control" `
    -Direction Inbound -Protocol TCP -LocalPort 8444 -Action Allow
```

### Network Issues

#### Devices not discovering each other

**Checklist:**
1. ✅ Both devices on same network
2. ✅ mDNS enabled (avahi on Linux, Bonjour on Windows)
3. ✅ Firewall allows port 5353/udp
4. ✅ No VPN or complex network setup

**Test connectivity:**
```bash
# On Linux, ping Windows
ping <windows-ip>

# Try manual connection
telnet <windows-ip> 8444
```

#### High latency

**Solutions:**
- Use wired connection instead of WiFi
- Check for network congestion
- Reduce video quality (Phase 0.2+)
- Disable QoS/traffic shaping

---

## Next Steps

### Learn More

- **[Development Guide](docs/DEVELOPMENT.md)** - Detailed technical documentation
- **[Project Status](docs/STATUS.md)** - Current progress and roadmap
- **[Architecture](README.md#architecture)** - System design overview

### Contribute

- Report bugs: [GitHub Issues](https://github.com/yourusername/project-mirage/issues)
- Suggest features: [Discussions](https://github.com/yourusername/project-mirage/discussions)
- Submit patches: [Pull Requests](https://github.com/yourusername/project-mirage/pulls)

### Upcoming Features

**Phase 0.2 (Next):** Window streaming
- Capture Linux windows
- Stream to Windows with low latency
- Hardware-accelerated encoding

**Phase 0.3:** Input feedback
- Click and type on streamed windows
- Full interactive experience

**Phase 0.4:** GUI control panel
- Visual configuration
- Drag-and-drop layout editor
- Connection management

### Join the Community

- **Discord:** [Join our server](#) (coming soon)
- **Twitter:** [@ProjectMirage](#) (coming soon)
- **Reddit:** [r/ProjectMirage](#) (coming soon)

---

## Support

Need help?

1. Check [Troubleshooting](#troubleshooting) section
2. Read [Development Guide](docs/DEVELOPMENT.md)
3. Search [existing issues](https://github.com/yourusername/project-mirage/issues)
4. Create a [new issue](https://github.com/yourusername/project-mirage/issues/new)

---

**Happy Mirage-ing! **

*Last updated: 2025-11-05*
