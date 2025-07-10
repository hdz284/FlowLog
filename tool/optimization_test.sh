#!/bin/bash
set -e

# =========================
# OPTIMIZATION TEST SCRIPT (Correctness + Timing)
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
    
    if [ -d "$FACT_DIR" ] && [ -d "$PROG_DIR" ]; then
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

    if [ ! -f "$result_size_file" ] || [ ! -f "$reference_size_file" ]; then
        echo "[ERROR] Missing result or reference size file!"
        return 1
    fi

    # Sort both files in place before comparison
    sort -o "$result_size_file" "$result_size_file"
    sort -o "$reference_size_file" "$reference_size_file"
    
    # Simple file comparison
    if cmp -s "$result_size_file" "$reference_size_file"; then
        echo "[PASS] Files match - correctness passed for $prog_name ($optimization)"
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
    local optimization_flag="$3"
    local optimization_label="$4"

    local prog_path="${PROG_DIR}/${prog_name}"
    local fact_path="${FACT_DIR}/${dataset_name}"

    echo "[TEST] Running $prog_name with $dataset_name ($optimization_label)"

    rm -rf "$CSV_DIR/csvs"
    mkdir -p "$CSV_DIR/csvs"

    if [ -z "$optimization_flag" ]; then
        "$BINARY_PATH" --program "$prog_path" --facts "$fact_path" --csvs "$CSV_DIR" --workers "$WORKERS"
    else
        "$BINARY_PATH" --program "$prog_path" --facts "$fact_path" --csvs "$CSV_DIR" --workers "$WORKERS" "$optimization_flag"
    fi

    local program_stem="${prog_name%.*}"
    local result_size_file="${CSV_DIR}/csvs/size.txt"

    verify_results_with_reference "$program_stem" "$optimization_label" "$result_size_file" || {
        echo "[ERROR] Verification failed for $prog_name ($optimization_label)"
        exit 1
    }

    echo "[DONE] Test completed for $prog_name ($optimization_label)"
}

run_all_optimization_tests() {
    echo "[TEST] Running optimization tests..."

    local optimizations=("" "-O1" "-O2" "-O3")
    local opt_labels=("none" "1" "2" "3")

    while IFS='=' read -r prog_name dataset_name; do
        if [ -z "$prog_name" ] || [ -z "$dataset_name" ]; then
            continue
        fi

        echo "[PROGRAM] Testing $prog_name with $dataset_name"
        echo "========================================"

        for i in "${!optimizations[@]}"; do
            run_single_optimization_test "$prog_name" "$dataset_name" "${optimizations[$i]}" "${opt_labels[$i]}"
        done
    done < "$CONFIG_FILE"

    echo "[OK] All optimization tests passed!"
}

generate_timing_table() {
    echo ""
    echo "============================"
    echo "[SUMMARY] Timing Results Table"
    echo "============================"

    printf "| %-10s | %-17s | %-17s | %-17s | %-17s |\n" "Program" "No Optimization" "O1" "O2" "O3"
    printf "|------------|-------------------|-------------------|-------------------|-------------------|\n"

    while IFS='=' read -r prog_name dataset_name; do
        if [ -z "$prog_name" ] || [ -z "$dataset_name" ]; then
            continue
        fi

        local program_stem="${prog_name%.*}"
        printf "| %-10s " "$program_stem"

        for opt in "none" "1" "2" "3"; do
            local time_file="result/time/${program_stem}_${opt}.txt"
            if [ -f "$time_file" ]; then
                elapsed_time=$(grep -oP '^[0-9]+\.[0-9]+' "$time_file" || echo "N/A")
            else
                elapsed_time="              N/A"
            fi

            # Pad numbers nicely
            if [[ "$elapsed_time" =~ ^[0-9] ]]; then
                printf "| %17.6f " "$elapsed_time"
            else
                printf "| %-17s " "$elapsed_time"
            fi
        done

        printf "|\n"
    done < "$CONFIG_FILE"
}

# =========================
# MAIN EXECUTION
# =========================

main() {
    echo "[START] FlowLog Optimization Test (Correctness + Timing)"

    echo "[BUILD] Building the project..."
    cargo build --release

    setup_dataset
    setup_size_reference

    echo "=== SETUP COMPLETE ==="

    # run_all_optimization_tests

    generate_timing_table

    echo "[FINISH] All optimization test cases completed successfully."
}

main "$@"
