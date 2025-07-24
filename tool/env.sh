#!/bin/bash
set -e

echo "[SETUP] Setting up FlowLog development environment..."

# Install system packages
echo "[CHECK] Checking system packages..."
sudo apt update -qq && sudo apt upgrade -y -qq

packages=()
command -v htop >/dev/null || packages+=("htop")
command -v dos2unix >/dev/null || packages+=("dos2unix")

if [ ${#packages[@]} -gt 0 ]; then
    echo "[INSTALL] Installing: ${packages[*]}"
    sudo apt install -y -qq "${packages[@]}"
fi

# Install/update Rust
if ! command -v rustc >/dev/null 2>&1; then
    echo "[INSTALL] Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    # Source cargo environment for current session
    source "$HOME/.cargo/env"
    # Add to bashrc for future sessions
    echo 'source "$HOME/.cargo/env"' >> ~/.bashrc
else
    echo "[FOUND] Rust already installed"
    # Make sure cargo is in PATH for current session
    source "$HOME/.cargo/env" 2>/dev/null || export PATH="$HOME/.cargo/bin:$PATH"
fi

echo "[UPDATE] Updating Rust toolchain..."
rustup update >/dev/null && rustup default stable >/dev/null

# Verify Flowlog compiles
echo "[VERIFY] Verifying Flowlog compilation..."
cargo check

echo "[COMPLETE] Environment setup completed successfully!"