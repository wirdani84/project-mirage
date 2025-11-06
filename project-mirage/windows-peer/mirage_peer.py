"""
Project Mirage - Windows Peer Agent
Receives mouse control and renders streamed windows from Linux host
"""

import asyncio
import sys
import socket
import platform
from dataclasses import dataclass
from typing import Optional, Dict, List
import logging

# Check if running on Windows
IS_WINDOWS = platform.system() == "Windows"

if IS_WINDOWS:
    import ctypes
    from ctypes import wintypes
    import win32api
    import win32con
    import win32gui
    import pywintypes

logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger(__name__)


@dataclass
class PeerConfig:
    """Configuration for Windows peer"""
    node_name: str = "windows-peer"
    discovery_port: int = 5353
    control_port: int = 8444
    max_fps: int = 60


class InputInjector:
    """Injects mouse and keyboard input on Windows"""
    
    def __init__(self):
        if not IS_WINDOWS:
            logger.warning("Input injection only works on Windows")
            return
            
        # Windows constants for input injection
        self.MOUSEEVENTF_MOVE = 0x0001
        self.MOUSEEVENTF_LEFTDOWN = 0x0002
        self.MOUSEEVENTF_LEFTUP = 0x0004
        self.MOUSEEVENTF_RIGHTDOWN = 0x0008
        self.MOUSEEVENTF_RIGHTUP = 0x0010
        self.MOUSEEVENTF_WHEEL = 0x0800
        
    def move_mouse(self, x: int, y: int):
        """Move mouse to absolute position"""
        if IS_WINDOWS:
            win32api.SetCursorPos((x, y))
        else:
            logger.debug(f"Move mouse to ({x}, {y})")
    
    def move_mouse_relative(self, dx: int, dy: int):
        """Move mouse by relative delta"""
        if IS_WINDOWS:
            pos = win32api.GetCursorPos()
            self.move_mouse(pos[0] + dx, pos[1] + dy)
        else:
            logger.debug(f"Move mouse by ({dx}, {dy})")
    
    def click_button(self, button: str, pressed: bool):
        """Click mouse button"""
        if not IS_WINDOWS:
            logger.debug(f"{'Press' if pressed else 'Release'} {button} button")
            return
            
        if button == "left":
            flag = self.MOUSEEVENTF_LEFTDOWN if pressed else self.MOUSEEVENTF_LEFTUP
        elif button == "right":
            flag = self.MOUSEEVENTF_RIGHTDOWN if pressed else self.MOUSEEVENTF_RIGHTUP
        else:
            return
            
        win32api.mouse_event(flag, 0, 0, 0, 0)
    
    def scroll_wheel(self, delta: int):
        """Scroll mouse wheel"""
        if IS_WINDOWS:
            win32api.mouse_event(self.MOUSEEVENTF_WHEEL, 0, 0, delta, 0)
        else:
            logger.debug(f"Scroll wheel: {delta}")


class DiscoveryClient:
    """mDNS-based discovery client for finding Linux host"""
    
    def __init__(self, config: PeerConfig):
        self.config = config
        self.discovered_hosts: Dict[str, dict] = {}
        
    async def start_discovery(self):
        """Start discovering Linux hosts"""
        logger.info("🔍 Starting mDNS discovery...")
        
        try:
            # Simple UDP broadcast for discovery
            # In production, use proper mDNS library (zeroconf)
            sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
            sock.setsockopt(socket.SOL_SOCKET, socket.SO_BROADCAST, 1)
            sock.bind(('', self.config.discovery_port))
            
            logger.info(f"✓ Listening for broadcasts on port {self.config.discovery_port}")
            logger.info("   Waiting for Linux host to announce...")
            
            # For demo purposes, we'll just wait
            # In real implementation, listen for mDNS announcements
            await asyncio.sleep(2)
            
        except Exception as e:
            logger.error(f"Discovery error: {e}")
    
    def get_hosts(self) -> List[dict]:
        """Get list of discovered hosts"""
        return list(self.discovered_hosts.values())


class StreamRenderer:
    """Renders incoming video streams in borderless windows"""
    
    def __init__(self):
        self.active_windows: Dict[str, any] = {}
        
    def create_mirror_window(self, stream_id: str, width: int, height: int):
        """Create a borderless window for rendering stream"""
        logger.info(f"Creating mirror window for stream {stream_id} ({width}x{height})")
        
        if not IS_WINDOWS:
            logger.warning("Mirror windows only work on Windows")
            return
            
        # In Phase 0.2, this will create actual window using Win32 API
        # For now, just log
        self.active_windows[stream_id] = {
            'width': width,
            'height': height,
            'created_at': asyncio.get_event_loop().time()
        }
    
    def close_mirror_window(self, stream_id: str):
        """Close a mirror window"""
        if stream_id in self.active_windows:
            logger.info(f"Closing mirror window for stream {stream_id}")
            del self.active_windows[stream_id]


class MiragePeerAgent:
    """Main Windows peer agent"""
    
    def __init__(self, config: PeerConfig):
        self.config = config
        self.input_injector = InputInjector()
        self.discovery = DiscoveryClient(config)
        self.renderer = StreamRenderer()
        self.running = False
        
    async def start(self):
        """Start the peer agent"""
        logger.info("🌟 Project Mirage - Windows Peer Agent")
        logger.info("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━")
        logger.info(f"✓ Node name: {self.config.node_name}")
        logger.info(f"✓ Platform: {platform.system()} {platform.release()}")
        
        self.running = True
        
        # Start discovery
        await self.discovery.start_discovery()
        
        logger.info("✓ Peer agent ready")
        logger.info("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━")
        logger.info("Phase 0.1: Mouse sharing enabled")
        logger.info("  • Waiting for connections from Linux host")
        logger.info("  • Press Ctrl+C to exit")
        
    async def run(self):
        """Main run loop"""
        while self.running:
            await asyncio.sleep(1)
            
            # In Phase 0.1, we'll listen for mouse control messages
            # In Phase 0.2, we'll also render video streams
    
    def stop(self):
        """Stop the peer agent"""
        logger.info("🛑 Shutting down...")
        self.running = False


async def main():
    """Main entry point"""
    import argparse
    
    parser = argparse.ArgumentParser(description="Project Mirage - Windows Peer Agent")
    parser.add_argument("--name", help="Node name", default="windows-peer")
    parser.add_argument("--scan", action="store_true", help="Scan for Linux hosts")
    parser.add_argument("--verbose", action="store_true", help="Verbose logging")
    
    args = parser.parse_args()
    
    if args.verbose:
        logging.getLogger().setLevel(logging.DEBUG)
    
    config = PeerConfig(node_name=args.name)
    agent = MiragePeerAgent(config)
    
    try:
        await agent.start()
        await agent.run()
    except KeyboardInterrupt:
        logger.info("\n\nReceived interrupt signal")
    finally:
        agent.stop()
        logger.info("✓ Peer agent stopped")


if __name__ == "__main__":
    if sys.platform == "win32":
        # On Windows, use ProactorEventLoop for better performance
        asyncio.set_event_loop_policy(asyncio.WindowsProactorEventLoopPolicy())
    
    asyncio.run(main())
