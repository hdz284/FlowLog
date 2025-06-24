#!/bin/bash

# Exit on any error
set -e  

# Update system package
echo "Updating System Package..."
sudo apt update && sudo apt upgrade -y

# Install htop
if ! command -v htop &> /dev/null; then
    echo "Installing htop..."
    sudo apt install -y htop
else
    echo "htop is already installed."
fi

# Install rust
if ! command -v rustc &> /dev/null; then
    echo "Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    export PATH="$HOME/.cargo/bin:$PATH"
    echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.bashrc
    source ~/.bashrc
else
    echo "Rust is already installed."
fi

# Ensuring newest Rust version
echo "Ensuring Rust is up-to-date..."
rustup update
rustup default stable

echo "=== SETUP COMPLETE ==="
