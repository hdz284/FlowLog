#!/bin/bash
set -e

# =========================
# OPTIMIZATION TEST SCRIPT
# =========================

# =========================
# CONFIGURATION
# =========================

CONFIG_FILE="./test/correctness_test/config.txt"
PROG_DIR="./test/correctness_test/program"
FACT_DIR="./test/correctness_test/dataset"
SIZE_DIR="./test/correctness_test/correctness_size"
CSV_DIR="./result"
BINARY_PATH="./target/release/executing"
WORKERS=64

# =========================
# DATASET SETUP
# =========================

setup_dataset() {
    mkdir -p ./test/correctness_test
    
    if [ -d "./test/correctness_test/dataset" ] && [ -d "./test/correctness_test/program" ]; then
        echo "[OK] Dataset already extracted. Skipping download."
    else
        echo "[DOWNLOAD] Downloading and extracting dataset bundle..."
        local zip_path="./test/correctness_test.zip"
        wget -O "$zip_path" https://pages.cs.wisc.edu/~m0riarty/correctness_test.zip
        unzip "$zip_path" -d "./test"
        rm "$zip_path"
        echo "[OK] Dataset extracted and zip file removed."

        echo "[FIX] Fixing line endings in config.txt..."
        dos2unix "$CONFIG_FILE" 2>/dev/null || true
    fi
}

setup_size_reference() {
    if [ -d "$SIZE_DIR" ]; then
        echo "[OK] Size reference already extracted. Skipping download."
        return
    fi
    
    echo "[DOWNLOAD] Downloading and extracting size reference..."
    local zip_path="./test/correctness_test/solution_size.zip"
    wget -O "$zip_path" https://pages.cs.wisc.edu/~m0riarty/correctness_size.zip
    unzip "$zip_path" -d "./test/correctness_test"
    rm "$zip_path"
    echo "[OK] Size reference extracted and zip file removed."
}

# =========================
# TEST FUNCTIONS
# =========================

verify_results_with_reference() {
    local prog_name="$1"
    local optimization="$2"
    local result_size_file="$3"
    local reference_size_file="${SIZE_DIR}/${prog_name}_size.txt"

    echo "[VERIFY] Checking results for $prog_name with $optimization optimization..."
    echo "[VERIFY] Comparing $result_size_file with $reference_size_file"

    if [ ! -f "$result_size_file" ]; then
        echo "[ERROR] Result size file $result_size_file not found!"
        return 1
    fi

    if [ ! -f "$reference_size_file" ]; then
        echo "[ERROR] Reference size file $reference_size_file not found!"
        return 1
    fi

    # Sort both files in place before comparison
    sort -o "$result_size_file" "$result_size_file"
    sort -o "$reference_size_file" "$reference_size_file"
    
    # Simple file comparison
    if cmp -s "$result_size_file" "$reference_size_file"; then
        echo "[PASS] Files are identical - test passed for $prog_name ($optimization)!"
        return 0
    else
        echo "[FAIL] Files differ - test failed for $prog_name ($optimization)!"
        echo "[DEBUG] Showing differences:"
        diff "$reference_size_file" "$result_size_file" || true
        return 1
    fi
}

run_single_optimization_test() {
    local prog_name="$1"
    local dataset_name="$2"
    local optimization="$3"
    
    local prog_path="${PROG_DIR}/${prog_name}"
    local fact_path="${FACT_DIR}/${dataset_name}"

    echo "[TEST] Running $prog_name with $dataset_name ($optimization)"

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
    echo "[RUN] Command executing: RUST_LOG=info $BINARY_PATH --program $prog_path --facts $fact_path --csvs $CSV_DIR --workers $WORKERS $optimization"

    # Run the binary
    RUST_LOG=info "$BINARY_PATH" \
        --program "$prog_path" \
        --facts "$fact_path" \
        --csvs "$CSV_DIR" \
        --workers "$WORKERS" \
        "$optimization"

    # Verify results
    echo "[VERIFY] Checking results for $optimization..."
    local program_stem="${prog_name%.*}"
    local result_size_file="${CSV_DIR}/size.txt"

    verify_results_with_reference "$program_stem" "$optimization" "$result_size_file" || {
        echo "[ERROR] Verification failed for $prog_name ($optimization)"
        exit 1
    }

    echo "[PASS] Test completed: $prog_name ($optimization)"
}

run_optimization_tests() {
    echo "[TEST] Running optimization tests..."

    local optimizations=("-O1" "-O2" "-O3")

    while IFS='=' read -r prog_name dataset_name; do
        if [ -z "$prog_name" ] || [ -z "$dataset_name" ]; then
            continue
        fi

        echo "[PROGRAM] Testing $prog_name with $dataset_name"
        echo "========================================"

        for optimization in "${optimizations[@]}"; do
            run_single_optimization_test "$prog_name" "$dataset_name" "$optimization"
        done
    done < "$CONFIG_FILE"
    
    echo "[OK] All optimization tests passed!"
}

# =========================
# MAIN EXECUTION
# =========================

main() {
    echo "[START] FlowLog Optimization Test"
    
    
    echo "[BUILD] Building the project..."
    cargo build --release
    
    setup_dataset
    setup_size_reference
    
    echo "=== SETUP COMPLETE ==="
    
    run_optimization_tests

    echo "[FINISH] All optimization test cases finished successfully."
}

main "$@"