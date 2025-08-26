#!/bin/bash
set -e

echo "[SETUP] Setting up FlowLog development environment..."

# Detect operating system
OS="$(uname -s)"
echo "[DETECT] Operating system: $OS"

# Install system packages
echo "[CHECK] Checking system packages..."

if [[ "$OS" == "Linux" ]]; then
    # Linux (Ubuntu/Debian)
    sudo apt update -qq && sudo apt upgrade -y -qq
    
    packages=()
    command -v htop >/dev/null || packages+=("htop")
    command -v dos2unix >/dev/null || packages+=("dos2unix")
    
    if [ ${#packages[@]} -gt 0 ]; then
        echo "[INSTALL] Installing Linux packages: ${packages[*]}"
        sudo apt install -y -qq "${packages[@]}"
    fi
    
elif [[ "$OS" == "Darwin" ]]; then
    # macOS
    # Check if Homebrew is installed
    if ! command -v brew >/dev/null 2>&1; then
        echo "[INSTALL] Installing Homebrew..."
        /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
        # Add Homebrew to PATH for current session
        eval "$(/opt/homebrew/bin/brew shellenv)"
    else
        echo "[FOUND] Homebrew already installed"
    fi
    
    # Update Homebrew
    echo "[UPDATE] Updating Homebrew..."
    brew update >/dev/null
    
    packages=()
    command -v htop >/dev/null || packages+=("htop")
    command -v dos2unix >/dev/null || packages+=("dos2unix")
    
    if [ ${#packages[@]} -gt 0 ]; then
        echo "[INSTALL] Installing macOS packages: ${packages[*]}"
        brew install "${packages[@]}"
    fi
    
else
    echo "[WARNING] Unsupported operating system: $OS"
    echo "[WARNING] Skipping system package installation..."
fi

# Install/update Rust
RUST_VERSION="1.89.0"
if ! command -v rustc >/dev/null 2>&1; then
    echo "[INSTALL] Installing Rust $RUST_VERSION..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain $RUST_VERSION
    # Source cargo environment for current session
    source "$HOME/.cargo/env"
    
    # Add to shell configuration for future sessions
    # Detect shell and add to appropriate config file
    if [[ "$OS" == "Darwin" ]] && [[ "$SHELL" == *"zsh"* ]]; then
        echo "[CONFIG] Adding Rust to ~/.zshrc for macOS zsh"
        echo 'source "$HOME/.cargo/env"' >> ~/.zshrc
    else
        echo "[CONFIG] Adding Rust to ~/.bashrc"
        echo 'source "$HOME/.cargo/env"' >> ~/.bashrc
    fi
else
    echo "[FOUND] Rust already installed"
    # Make sure cargo is in PATH for current session
    source "$HOME/.cargo/env" 2>/dev/null || export PATH="$HOME/.cargo/bin:$PATH"
    # Set to specific version if not already set
    echo "[PIN] Setting Rust toolchain to version $RUST_VERSION..."
    rustup toolchain install $RUST_VERSION >/dev/null
    rustup default $RUST_VERSION >/dev/null
fi

# Verify Flowlog compiles
echo "[VERIFY] Verifying Flowlog compilation..."
cargo check

echo "[COMPLETE] Environment setup completed successfully!"