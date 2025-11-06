.PHONY: all build test clean run-demo install-deps help

# Default target
all: build

help:
	@echo "Project Mirage - Build System"
	@echo "=============================="
	@echo ""
	@echo "Available targets:"
	@echo "  make build          - Build all components"
	@echo "  make build-linux    - Build Linux host daemon"
	@echo "  make build-windows  - Setup Windows peer"
	@echo "  make test           - Run all tests"
	@echo "  make run-demo       - Run Phase 0.1 demo"
	@echo "  make run-host       - Run Linux host daemon"
	@echo "  make run-peer       - Run Windows peer agent"
	@echo "  make install-deps   - Install system dependencies"
	@echo "  make clean          - Clean build artifacts"
	@echo "  make fmt            - Format code"
	@echo "  make lint           - Run linters"
	@echo "  make docs           - Generate documentation"
	@echo ""

# Build targets
build: build-linux build-windows

build-linux:
	@echo "Building Linux host daemon..."
	cd linux-host && cargo build --release
	@echo "✓ Linux host built successfully"

build-windows:
	@echo "Setting up Windows peer..."
	cd windows-peer && \
		python3 -m venv venv 2>/dev/null || python -m venv venv && \
		. venv/bin/activate && \
		pip install -r requirements.txt
	@echo "✓ Windows peer setup complete"

# Test targets
test: test-linux test-python

test-linux:
	@echo "Running Rust tests..."
	cd linux-host && cargo test

test-python:
	@echo "Running Python tests..."
	cd windows-peer && python3 -m pytest tests/ || echo "No tests yet"

# Run targets
run-demo:
	@echo "Running Phase 0.1 demo..."
	python3 demo_phase_01.py

run-host:
	@echo "Starting Linux host daemon..."
	cd linux-host && cargo run --release -- --discover --verbose

run-peer:
	@echo "Starting Windows peer agent..."
	cd windows-peer && \
		. venv/bin/activate && \
		python mirage_peer.py --scan --verbose

# Development targets
fmt:
	@echo "Formatting code..."
	cd linux-host && cargo fmt

lint:
	@echo "Running linters..."
	cd linux-host && cargo clippy -- -D warnings

docs:
	@echo "Generating documentation..."
	cd linux-host && cargo doc --no-deps --open

# Installation targets
install-deps:
	@echo "Installing system dependencies..."
	@if command -v apt-get >/dev/null 2>&1; then \
		echo "Detected Debian/Ubuntu"; \
		sudo apt-get update; \
		sudo apt-get install -y build-essential cmake pkg-config \
			libwayland-dev libinput-dev libevdev-dev \
			libssl-dev protobuf-compiler python3-pip python3-venv; \
	elif command -v dnf >/dev/null 2>&1; then \
		echo "Detected Fedora"; \
		sudo dnf install -y gcc-c++ cmake pkg-config \
			wayland-devel libinput-devel libevdev-devel \
			openssl-devel protobuf-compiler python3-pip; \
	else \
		echo "Unsupported distribution. Please install dependencies manually."; \
		exit 1; \
	fi
	@echo "✓ System dependencies installed"
	@echo ""
	@echo "Installing Rust (if not present)..."
	@if ! command -v cargo >/dev/null 2>&1; then \
		curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y; \
		echo "✓ Rust installed. Please run: source $$HOME/.cargo/env"; \
	else \
		echo "✓ Rust already installed"; \
	fi

# Clean targets
clean:
	@echo "Cleaning build artifacts..."
	cd linux-host && cargo clean
	rm -rf windows-peer/venv
	rm -rf windows-peer/__pycache__
	find . -name "*.pyc" -delete
	find . -name "__pycache__" -type d -delete
	@echo "✓ Clean complete"

# Development workflow
dev: build run-demo

# Production workflow  
prod: install-deps build test
	@echo "✓ Production build complete"
	@echo ""
	@echo "To run:"
	@echo "  Linux:   cd linux-host/target/release && ./mirage-host --discover"
	@echo "  Windows: cd windows-peer && python mirage_peer.py --scan"

# Quick start
quickstart:
	@echo "Project Mirage - Quick Start"
	@echo "============================"
	@echo ""
	@echo "1. Installing dependencies..."
	@make install-deps
	@echo ""
	@echo "2. Building project..."
	@make build
	@echo ""
	@echo "3. Running demo..."
	@make run-demo
	@echo ""
	@echo "✓ Quick start complete!"
	@echo ""
	@echo "Next steps:"
	@echo "  - Review docs/DEVELOPMENT.md for detailed guide"
	@echo "  - Run 'make run-host' on Linux to start host daemon"
	@echo "  - Run 'make run-peer' on Windows to start peer agent"
