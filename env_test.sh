#!/bin/bash

# Exit on any error
set -e  

# --------------------------
# System and Rust Setup
# --------------------------

echo "🔧 Updating system packages..."
sudo apt update && sudo apt upgrade -y

# Install htop and dos2unix if not present
missing_packages=()

if ! command -v htop &> /dev/null; then
    missing_packages+=("htop")
fi

if ! command -v dos2unix &> /dev/null; then
    missing_packages+=("dos2unix")
fi

if [ ${#missing_packages[@]} -ne 0 ]; then
    echo "📦 Installing missing packages: ${missing_packages[*]}..."
    sudo apt install -y "${missing_packages[@]}"
else
    echo "✅ htop and dos2unix are already installed."
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

ZIP_PATH="./test/correctness_test.zip"
UNZIP_DIR="./test"

if [ -d "./test/correctness_test/dataset" ] && [ -d "./test/correctness_test/program" ]; then
    echo "📁 Dataset already extracted. Skipping download."
else
    echo "⬇️ Downloading and extracting dataset bundle..."
    wget -O "$ZIP_PATH" https://pages.cs.wisc.edu/~m0riarty/correctness_test.zip
    unzip "$ZIP_PATH" -d "$UNZIP_DIR"
    rm "$ZIP_PATH"
    echo "✅ Dataset extracted and zip file removed."

    # Fix config.txt line endings
    echo "🛠️ Fixing line endings in config.txt..."
    dos2unix ./test/correctness_test/config.txt 2>/dev/null || true
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
        local csv_path="${CSV_DIR}/${name}.csv"

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
# Test Runner for a Build
# --------------------------

run_tests_for_binary() {
    local BUILD_TYPE="$1"
    local BINARY_PATH="./target/release/executing"

    echo "🚀 Running tests for build type: $BUILD_TYPE"

    local CONFIG_FILE="./test/correctness_test/config.txt"
    local PROG_DIR="./test/correctness_test/program"
    local FACT_DIR="./test/correctness_test/dataset"
    local CSV_DIR="./result"
    local WORKERS=32

    while IFS='=' read -r prog_name dataset_name; do
        if [ -z "$prog_name" ] || [ -z "$dataset_name" ]; then
            continue
        fi

        local prog_path="${PROG_DIR}/${prog_name}"
        local fact_path="${FACT_DIR}/${dataset_name}"

        echo "🔧 Testing Program: $prog_name, Dataset: $dataset_name"

        if [ ! -f "$prog_path" ]; then
            echo "❌ Program not found: $prog_path"
            exit 1
        fi

        if [ ! -d "$fact_path" ]; then
            echo "❌ Dataset not found: $fact_path"
            exit 1
        fi

        for sharing_flag in "" "--no-sharing"; do
            local test_case="with-sharing"
            if [ "$sharing_flag" = "--no-sharing" ]; then
                test_case="no-sharing"
            fi

            echo "▶️ [$BUILD_TYPE] Running test case: $test_case"

            rm -rf "$CSV_DIR"
            mkdir -p "$CSV_DIR"

            "$BINARY_PATH" \
                --program "$prog_path" \
                --facts "$fact_path/" \
                --csvs "$CSV_DIR/" \
                --workers "$WORKERS" \
                --output-result $sharing_flag

            echo "🔍 Verifying result ($test_case)..."
            verify_results || {
                echo "❌ Verification failed for $prog_name ($BUILD_TYPE, $test_case)"
                exit 1
            }

            echo "✅ Test Passed: $prog_name ($BUILD_TYPE, $test_case)"
            echo "----------------------------------------"
        done
    done < "$CONFIG_FILE"

    echo "🎉 All tests passed for build type: $BUILD_TYPE"
}

# --------------------------
# Full Build and Test Pipeline
# --------------------------

echo "🔨 Building Present Semiring (default)..."
cargo build --release

run_tests_for_binary "present"

echo "🔨 Building Isize Semiring..."
cargo build --release --features isize-type --no-default-features

run_tests_for_binary "isize"

echo "🏁 All 4 test cases per program completed successfully."
