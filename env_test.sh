#!/bin/bash
set -e

# =========================
# CONFIGURATION
# =========================

CONFIG_FILE="./test/correctness_test/config.txt"
PROG_DIR="./test/correctness_test/program"
FACT_DIR="./test/correctness_test/dataset"
CSV_DIR="./result"
BINARY_PATH="./target/release/executing"
WORKERS=64

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

setup_dataset() {
    mkdir -p ./test
    
    if [ -d "./test/correctness_test/dataset" ] && [ -d "./test/correctness_test/program" ]; then
        echo "[OK] Dataset already extracted. Skipping download."
        return
    fi
    
    echo "[DOWNLOAD] Downloading and extracting dataset bundle..."
    local zip_path="./test/correctness_test.zip"
    wget -O "$zip_path" https://pages.cs.wisc.edu/~m0riarty/correctness_test.zip
    unzip "$zip_path" -d "./test"
    rm "$zip_path"
    echo "[OK] Dataset extracted and zip file removed."

    echo "[FIX] Fixing line endings in config.txt..."
    dos2unix "$CONFIG_FILE" 2>/dev/null || true
}

# =========================
# TEST FUNCTIONS
# =========================

verify_results() {
    local SIZE_FILE="${1:-./result/size.txt}"
    local CSV_DIR="${2:-./result}"

    echo "[VERIFY] Checking result files using $SIZE_FILE..."

    if [ ! -f "$SIZE_FILE" ]; then
        echo "[ERROR] Size file $SIZE_FILE not found!"
        return 1
    fi

    local pass=true

    while IFS= read -r line; do
        local name="${line%%:*}"
        local count_str="${line##*:}"
        local expected=$(echo "$count_str" | grep -o '[0-9]\+')
        local csv_path="${CSV_DIR}/${name}.csv"

        if [ ! -f "$csv_path" ]; then
            echo "[FAIL] Missing CSV: $csv_path"
            pass=false
            continue
        fi

        local actual
        actual=$(wc -l < "$csv_path")

        if [ "$expected" -eq "$actual" ]; then
            echo "[PASS] $name: expected = $expected, actual = $actual"
        else
            echo "[FAIL] $name: expected = $expected, actual = $actual"
            pass=false
        fi
    done < "$SIZE_FILE"

    if [ "$pass" = true ]; then
        echo "[OK] All results verified successfully!"
        return 0
    else
        echo "[ERROR] Verification failed!"
        return 1
    fi
}

run_single_test() {
    local prog_name="$1"
    local dataset_name="$2" 
    local sharing_flag="$3"
    local test_case="$4"
    
    local prog_path="${PROG_DIR}/${prog_name}"
    local fact_path="${FACT_DIR}/${dataset_name}"

    echo "[TEST] Running $prog_name with $dataset_name ($test_case)"

    # Validate inputs
    if [ ! -f "$prog_path" ]; then
        echo "[ERROR] Program not found: $prog_path"
        exit 1
    fi

    if [ ! -d "$fact_path" ]; then
        echo "[ERROR] Dataset not found: $fact_path"
        exit 1
    fi

    # Clean and prepare output directory
    rm -rf "$CSV_DIR"
    mkdir -p "$CSV_DIR"

    # Print the running command
    echo "[RUN] Command executing: RUST_LOG=info $BINARY_PATH --program $prog_path --facts $fact_path --csvs $CSV_DIR --workers $WORKERS $sharing_flag"

    # Run the binary
    RUST_LOG=info "$BINARY_PATH" \
        --program "$prog_path" \
        --facts "$fact_path" \
        --csvs "$CSV_DIR" \
        --workers "$WORKERS" \
        $sharing_flag

    # Verify results
    echo "[VERIFY] Checking results for $test_case..."
    verify_results || {
        echo "[ERROR] Verification failed for $prog_name ($test_case)"
        exit 1
    }

    echo "[PASS] Test completed: $prog_name ($test_case)"
}

run_tests_for_binary() {
    local BUILD_TYPE="$1"

    echo "[TEST] Running tests for build type: $BUILD_TYPE"

    while IFS='=' read -r prog_name dataset_name; do
        if [ -z "$prog_name" ] || [ -z "$dataset_name" ]; then
            continue
        fi

        echo "[PROGRAM] Testing $prog_name with $dataset_name"
        echo "----------------------------------------"

        for sharing_flag in "" "--no-sharing"; do
            local test_case="enable-sharing"
            if [ "$sharing_flag" = "--no-sharing" ]; then
                test_case="no-sharing"
            fi

            run_single_test "$prog_name" "$dataset_name" "$sharing_flag" "$test_case"
        done
    done < "$CONFIG_FILE"
    echo "[OK] All tests passed for build type: $BUILD_TYPE"
}

# =========================
# MAIN EXECUTION
# =========================

main() {
    echo "[START] FlowLog Environment Test"
    
    install_system_packages
    install_rust
    setup_dataset
    
    echo "=== SETUP COMPLETE ==="
    
    echo "[BUILD] Building Present Semiring (default)..."
    cargo build --release

    run_tests_for_binary "present"

    echo "[FINISH] All test cases per program finished successfully."
}

main "$@"