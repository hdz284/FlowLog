#!/bin/bash

# Exit on any error
set -e  

# --------------------------
# System and Rust Setup
# --------------------------

echo "🔧 Updating system packages..."
sudo apt update && sudo apt upgrade -y

# Install htop if not present
if ! command -v htop &> /dev/null; then
    echo "📦 Installing htop..."
    sudo apt install -y htop
else
    echo "✅ htop is already installed."
fi

# Install Rust if not present
if ! command -v rustc &> /dev/null; then
    echo "🦀 Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    export PATH="$HOME/.cargo/bin:$PATH"
    echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.bashrc
    source ~/.bashrc
else
    echo "✅ Rust is already installed."
fi

# Ensure Rust is up to date
echo "🔄 Ensuring Rust is up-to-date..."
rustup update
rustup default stable

# --------------------------
# Dataset Bundle Setup
# --------------------------

mkdir -p ./test

ZIP_PATH="./test/correctness_test_bundle.zip"
UNZIP_DIR="./test"

if [ -d "./test/correctness_test_data" ] && [ -d "./test/correctness_test_program" ]; then
    echo "📁 Dataset already extracted. Skipping download."
else
    echo "⬇️ Downloading and extracting dataset bundle..."
    wget -O "$ZIP_PATH" https://pages.cs.wisc.edu/~m0riarty/correctness_test_bundle.zip
    unzip "$ZIP_PATH" -d "$UNZIP_DIR"
    rm "$ZIP_PATH"
    echo "✅ Dataset bundle extracted and zip file removed."
fi

echo "=== SETUP COMPLETE ==="

# --------------------------
# Result Verification Function
# --------------------------

verify_results() {
    local SIZE_FILE="${1:-./result/size.txt}"
    local CSV_DIR="${2:-./result}"

    echo "🔍 Verifying result files using $SIZE_FILE..."

    if [ ! -f "$SIZE_FILE" ]; then
        echo "❌ Error: size file $SIZE_FILE not found!"
        return 1
    fi

    local pass=true

    while IFS= read -r line; do
        local name="${line%%:*}"
        local count_str="${line##*:}"
        local expected=$(echo "$count_str" | grep -o '[0-9]\+')
        local csv_path="${CSV_DIR}/${name}"

        if [ ! -f "$csv_path" ]; then
            echo "❌ Missing CSV: $csv_path"
            pass=false
            continue
        fi

        local actual
        actual=$(wc -l < "$csv_path")

        if [ "$expected" -eq "$actual" ]; then
            echo "✅ $name: expected = $expected, actual = $actual"
        else
            echo "❌ $name: expected = $expected, actual = $actual"
            pass=false
        fi
    done < "$SIZE_FILE"

    if [ "$pass" = true ]; then
        echo "🎉 All results verified successfully!"
        return 0
    else
        echo "⚠️ Verification failed!"
        return 1
    fi
}

# --------------------------
# Run All Correctness Programs
# --------------------------

run_all_correctness_tests() {
    local PROG_DIR="./test/correctness_test_program"
    local FACT_DIR="./test/correctness_test_data"
    local CSV_DIR="./result"
    local WORKERS=32

    for prog_path in "$PROG_DIR"/*.dl; do
        prog_file=$(basename "$prog_path")
        name="${prog_file%.dl}"  # strip .dl extension

        echo "🚀 Running program: $prog_file"

        # Clean previous result
        rm -rf "$CSV_DIR"
        mkdir -p "$CSV_DIR"

        cargo run --release --bin executing \
            -- --program "$prog_path" \
               --facts "$FACT_DIR/$name/" \
               --csvs "$CSV_DIR/" \
               --verbose \
               --workers "$WORKERS" \
               --output-result

        echo "🔍 Verifying result for $name..."
        verify_results || {
            echo "❌ Verification failed for $name"
            exit 1
        }

        echo "✅ $name PASSED"
        echo "-----------------------------"
    done

    echo "🎉 All correctness tests completed!"
}

# --------------------------
# Run the Full Pipeline
# --------------------------

run_all_correctness_tests
