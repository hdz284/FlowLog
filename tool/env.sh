#!/bin/bash
set -e

# =========================
# ENVIRONMENT SETUP SCRIPT
# =========================

echo "[START] FlowLog Environment Setup"

# =========================
# SETUP FUNCTIONS
# =========================

install_system_packages() {
    echo "[SETUP] Checking system packages..."
    sudo apt update && sudo apt upgrade -y
    
    local packages=()
    command -v htop >/dev/null || packages+=("htop")
    command -v dos2unix >/dev/null || packages+=("dos2unix")
    
    if [ ${#packages[@]} -gt 0 ]; then
        echo "[INSTALL] Installing packages: ${packages[*]}"
        sudo apt install -y "${packages[@]}"
    else
        echo "[OK] All required packages already installed"
    fi
}

install_rust() {
    if ! command -v rustc >/dev/null; then
        echo "[INSTALL] Installing Rust..."
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        export PATH="$HOME/.cargo/bin:$PATH"
        echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.bashrc
    else
        echo "[OK] Rust is already installed"
    fi
    
    echo "[UPDATE] Updating Rust to latest version..."
    rustup update && rustup default stable
}

# =========================
# MAIN EXECUTION
# =========================

main() {
    install_system_packages
    install_rust
    
    echo "[CHECK] Checking if project compiles..."
    cargo check
    
    echo "[FINISH] Environment setup completed successfully!"
}

main "$@"