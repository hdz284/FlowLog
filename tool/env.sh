#!/bin/bash
# Exit immediately if a command exits with a non-zero status
set -e

############################################################
# ENVIRONMENT SETUP SCRIPT
# This script sets up the development environment for FlowLog
# by installing system packages and Rust toolchain
############################################################

echo "[START] FlowLog Environment Setup"

############################################################
# SETUP FUNCTIONS
############################################################

install_system_packages() {
    # Update package lists and upgrade existing packages
    echo "[SETUP] Checking system packages..."
    sudo apt update && sudo apt upgrade -y
    
    # Check for required packages and add missing ones to install list
    local packages=()
    command -v htop >/dev/null || packages+=("htop")          # System monitor
    command -v dos2unix >/dev/null || packages+=("dos2unix")  # Line ending converter
    
    # Install any missing packages
    if [ ${#packages[@]} -gt 0 ]; then
        echo "[INSTALL] Installing packages: ${packages[*]}"
        sudo apt install -y "${packages[@]}"
    else
        echo "[OK] All required packages already installed"
    fi
}

install_rust() {
    # Check if Rust is already installed
    if ! command -v rustc >/dev/null; then
        # Install Rust using the official installer
        echo "[INSTALL] Installing Rust..."
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        # Add Rust to PATH for current session and future sessions
        export PATH="$HOME/.cargo/bin:$PATH"
        echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.bashrc
    else
        echo "[OK] Rust is already installed"
    fi
    
    echo "[UPDATE] Moving Rust to latest version..."
    rustup update && rustup default stable
}

############################################################
# MAIN EXECUTION
############################################################

main() {
    # Install required system packages
    install_system_packages
    
    # Install and update Rust toolchain
    install_rust
    
    # Verify the project compiles correctly
    echo "[CHECK] Checking if project compiles..."
    cargo check
    
    # Print completion message
    echo "[FINISH] Environment setup completed successfully!"
}

# Call main function with all script arguments
main "$@"