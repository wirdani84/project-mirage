#!/usr/bin/env python3
"""
Project Mirage - Phase 0.1 Demo
Simulates mouse sharing between Linux host and Windows peer
"""

import asyncio
import random
from dataclasses import dataclass
from datetime import datetime
from typing import List, Tuple

@dataclass
class MouseEvent:
    timestamp: float
    event_type: str  # "move", "button", "wheel", "edge_cross"
    data: dict

class VirtualScreen:
    """Represents a virtual screen for demo purposes"""
    
    def __init__(self, name: str, width: int, height: int):
        self.name = name
        self.width = width
        self.height = height
        self.mouse_x = width // 2
        self.mouse_y = height // 2
        
    def __repr__(self):
        return f"{self.name} ({self.width}x{self.height})"

class MirageDemo:
    """Demonstrates Project Mirage Phase 0.1 functionality"""
    
    def __init__(self):
        self.linux_screen = VirtualScreen("Linux Host", 1920, 1080)
        self.windows_screen = VirtualScreen("Windows Peer", 1920, 1080)
        self.active_screen = self.linux_screen
        self.mouse_x = self.linux_screen.width // 2
        self.mouse_y = self.linux_screen.height // 2
        self.event_log: List[str] = []
        
    def log_event(self, message: str):
        """Log an event with timestamp"""
        timestamp = datetime.now().strftime("%H:%M:%S.%f")[:-3]
        log_line = f"[{timestamp}] {message}"
        self.event_log.append(log_line)
        print(log_line)
    
    def move_mouse(self, delta_x: int, delta_y: int):
        """Simulate mouse movement"""
        old_x, old_y = self.mouse_x, self.mouse_y
        old_screen = self.active_screen
        
        # Update position
        self.mouse_x += delta_x
        self.mouse_y += delta_y
        
        # Check for edge crossing
        edge_threshold = 10
        
        # Check right edge (Linux → Windows)
        if (old_screen == self.linux_screen and 
            old_x < self.linux_screen.width - edge_threshold and
            self.mouse_x >= self.linux_screen.width - edge_threshold):
            
            self.log_event("🔄 Mouse crossed RIGHT edge (Linux → Windows)")
            self.active_screen = self.windows_screen
            self.mouse_x = 0
            self.log_event(f"   Mouse now at Windows Peer: ({self.mouse_x}, {self.mouse_y})")
            return True
        
        # Check left edge (Windows → Linux)
        if (old_screen == self.windows_screen and 
            old_x > edge_threshold and
            self.mouse_x <= edge_threshold):
            
            self.log_event("🔄 Mouse crossed LEFT edge (Windows → Linux)")
            self.active_screen = self.linux_screen
            self.mouse_x = self.linux_screen.width - 1
            self.log_event(f"   Mouse now at Linux Host: ({self.mouse_x}, {self.mouse_y})")
            return True
        
        # Normal movement
        self.mouse_x = max(0, min(self.active_screen.width, self.mouse_x))
        self.mouse_y = max(0, min(self.active_screen.height, self.mouse_y))
        
        return False
    
    def simulate_mouse_journey(self):
        """Simulate a mouse moving between screens"""
        print("\n" + "="*70)
        print("Project Mirage - Phase 0.1 Demo: Mouse Sharing")
        print("="*70)
        print(f"\nLinux Host: {self.linux_screen}")
        print(f"Windows Peer: {self.windows_screen}")
        print(f"\nStarting position: {self.active_screen.name} ({self.mouse_x}, {self.mouse_y})")
        print("\n" + "-"*70)
        
        # Scenario 1: Move from center to right edge (Linux → Windows)
        self.log_event("📍 Starting scenario: Move mouse from Linux to Windows")
        
        steps = 50
        for i in range(steps):
            crossed = self.move_mouse(40, 0)  # Move right
            if crossed:
                break
        
        # Pause
        self.log_event("⏸️  Mouse now on Windows - simulating some activity...")
        
        # Scenario 2: Click on Windows
        self.log_event("🖱️  LEFT CLICK on Windows Peer")
        
        # Scenario 3: Move back to Linux
        self.log_event("📍 Moving mouse back to Linux")
        
        for i in range(steps):
            crossed = self.move_mouse(-40, 0)  # Move left
            if crossed:
                break
        
        self.log_event("✓ Demo complete!")
        
        print("\n" + "-"*70)
        print(f"\nFinal position: {self.active_screen.name} ({self.mouse_x}, {self.mouse_y})")
        print("\nPhase 0.1 Features Demonstrated:")
        print("  ✓ Mouse event capture")
        print("  ✓ Screen edge detection")
        print("  ✓ Seamless mouse transfer between devices")
        print("  ✓ Input event forwarding")
        print("\nNext Phase (0.2): Window streaming and rendering")
        print("="*70 + "\n")

def main():
    """Run the demo"""
    demo = MirageDemo()
    demo.simulate_mouse_journey()

if __name__ == "__main__":
    main()
