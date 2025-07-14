#!/bin/bash
set -e

# =========================
# CORRECTNESS TEST SCRIPT
# =========================

# =========================
# CONFIGURATION
# =========================

CONFIG_FILE="./test/correctness_test/config.txt"
PROG_DIR="./test/correctness_test/program"
FACT_DIR="./test/correctness_test/dataset"
SIZE_DIR="./test/correctness_test/correctness_size"
RESULT_DIR="./result"
BINARY_PATH="./target/release/executing"
WORKERS=64

# =========================
# DATASET SETUP
# =========================

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

verify_results() {
    local SIZE_FILE="${1:-./result/csvs/size.txt}"
    local CSV_DIR="${2:-./result/csvs}"

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

verify_results_with_reference() {
    local prog_name="$1"
    local dataset_name="$2"
    local test_label="$3"
    local result_size_file="$4"
    local reference_size_file="${SIZE_DIR}/${prog_name}_${dataset_name}_size.txt"

    echo "[VERIFY] Checking results for $prog_name with $test_label..."
    echo "[VERIFY] Comparing $result_size_file with $reference_size_file"

    if [ ! -f "$result_size_file" ] || [ ! -f "$reference_size_file" ]; then
        echo "[ERROR] Missing result or reference size file!"
        return 1
    fi

    # Sort both files in place before comparison
    sort -o "$result_size_file" "$result_size_file"
    sort -o "$reference_size_file" "$reference_size_file"
    
    # Simple file comparison
    if cmp -s "$result_size_file" "$reference_size_file"; then
        echo "[PASS] Files match - correctness passed for $prog_name ($test_label)"
    else
        echo "[FAIL] Files differ - test failed for $prog_name ($test_label)!"
        echo "[DEBUG] Showing differences:"
        diff "$reference_size_file" "$result_size_file" || true
        return 1
    fi
}

run_test() {
    local prog_name="$1"
    local dataset_name="$2"
    local flags="$3"
    local test_label="$4"
    
    local prog_path="${PROG_DIR}/${prog_name}"
    local fact_path="${FACT_DIR}/${dataset_name}"

    echo "[TEST] Running $prog_name with $dataset_name ($test_label)"

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
    rm -rf "$RESULT_DIR/csvs"
    mkdir -p "$RESULT_DIR/csvs"

    # Build command with flags
    local cmd="$BINARY_PATH --program $prog_path --facts $fact_path --csvs $RESULT_DIR --workers $WORKERS"
    if [ -n "$flags" ]; then
        cmd="$cmd $flags"
    fi

    # Print the running command
    echo "[RUN] Command executing: RUST_LOG=info $cmd"

    # Run the binary
    RUST_LOG=info $cmd

    # First verify basic results consistency
    local result_size_file="$RESULT_DIR/csvs/size.txt"
    echo "[VERIFY] Checking basic result consistency..."
    verify_results "$result_size_file" "$RESULT_DIR/csvs" || {
        echo "[ERROR] Basic verification failed for $prog_name ($test_label)"
        exit 1
    }

    # Then verify against reference if available
    local program_stem="${prog_name%.*}"
    local reference_size_file="${SIZE_DIR}/${program_stem}_${dataset_name}_size.txt"

    if [ -f "$reference_size_file" ]; then
        echo "[VERIFY] Checking against reference..."
        verify_results_with_reference "$program_stem" "$dataset_name" "$test_label" "$result_size_file" || {
            echo "[ERROR] Reference verification failed for $prog_name ($test_label)"
            exit 1
        }
    else
        echo "[ERROR] No reference file found for $program_stem, skipping reference verification"
        exit 1
    fi

    echo "[PASS] Test completed: $prog_name ($test_label)"
}

run_all_tests() {
    echo "[TEST] Running all correctness tests..."
    rm -rf "$RESULT_DIR"

    # Define sharing configurations
    local sharing_flags=("" "--no-sharing")
    local sharing_labels=("sharing" "no-sharing")

    # Define optimization configurations
    local optimization_flags=("" "-O1" "-O2" "-O3")
    local optimization_labels=("none" "O1" "O2" "O3")

    while IFS='=' read -r prog_name dataset_name; do
        if [ -z "$prog_name" ] || [ -z "$dataset_name" ]; then
            continue
        fi

        echo "[PROGRAM] Testing $prog_name with $dataset_name"
        echo "========================================"

        # Test all combinations of sharing and optimization flags
        for i in "${!sharing_flags[@]}"; do
            for j in "${!optimization_flags[@]}"; do
                local combined_flags="${sharing_flags[$i]} ${optimization_flags[$j]}"
                # Remove extra spaces
                combined_flags=$(echo "$combined_flags" | xargs)
                
                local test_label="${sharing_labels[$i]}-${optimization_labels[$j]}"
                
                run_test "$prog_name" "$dataset_name" "$combined_flags" "$test_label"
            done
        done
        
    done < "$CONFIG_FILE"

    echo "[OK] All correctness tests passed!"
}

# =========================
# MAIN EXECUTION
# =========================

main() {
    echo "[START] FlowLog Correctness Test (Including Optimization Correctness)"
    
    setup_dataset
    setup_size_reference
    
    echo "=== DATASET SETUP COMPLETE ==="

    echo "[BUILD] Building Present Semiring (default)..."
    cargo build --release
    
    echo "=== RUNNING ALL CORRECTNESS TESTS ==="
    run_all_tests

    echo "[FINISH] All correctness test cases finished successfully."
}

main "$@"