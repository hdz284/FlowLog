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
    local dataset_name="$1"
    local dataset_zip="./test/correctness_test/dataset/${dataset_name}.zip"
    local extract_path="${FACT_DIR}/${dataset_name}"
    local dataset_url="https://pages.cs.wisc.edu/~m0riarty/dataset/${dataset_name}.zip"

    if [ -d "$extract_path" ]; then
        echo "[OK] Dataset $dataset_name already extracted. Skipping."
        return
    fi

    mkdir -p "$FACT_DIR"

    if [ ! -f "$dataset_zip" ]; then
        echo "[DOWNLOAD] Downloading $dataset_name.zip from $dataset_url..."
        mkdir -p "$(dirname "$dataset_zip")"
        wget -O "$dataset_zip" "$dataset_url" || {
            echo "[ERROR] Failed to download dataset: $dataset_name"
            exit 1
        }
    fi

    echo "[EXTRACT] Extracting $dataset_name..."
    unzip -q "$dataset_zip" -d "$FACT_DIR"
    echo "[OK] Dataset $dataset_name ready."
}

cleanup_dataset() {
    local dataset_name="$1"
    local extract_path="${FACT_DIR}/${dataset_name}"
    local zip_path="${FACT_DIR}/${dataset_name}.zip"

    echo "[CLEANUP] Removing dataset $dataset_name..."
    rm -rf "$extract_path"
    rm -f "$zip_path"
}

setup_size_reference() {
    if [ -d "$SIZE_DIR" ]; then
        echo "[OK] Size reference already extracted. Skipping download."
        return
    fi

    echo "[DOWNLOAD] Downloading and extracting size reference..."
    local zip_path="./test/correctness_test/solution_size.zip"
    mkdir -p ./test/correctness_test

    wget -O "$zip_path" https://pages.cs.wisc.edu/~m0riarty/correctness_size.zip
    unzip "$zip_path" -d "./test/correctness_test"
    rm "$zip_path"
    echo "[OK] Size reference extracted."
}

setup_config_file() {
    if [ -f "$CONFIG_FILE" ]; then
        echo "[OK] Config file already exists. Skipping download."
        return
    fi

    echo "[DOWNLOAD] Downloading config.txt..."
    mkdir -p "$(dirname "$CONFIG_FILE")"
    wget -O "$CONFIG_FILE" https://pages.cs.wisc.edu/~m0riarty/config.txt

    echo "[FIX] Fixing line endings in config.txt..."
    dos2unix "$CONFIG_FILE" 2>/dev/null || true
    echo "[OK] Config file ready."
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

    echo "[VERIFY] Comparing result with reference for $prog_name on $dataset_name..."

    if [ ! -f "$result_size_file" ] || [ ! -f "$reference_size_file" ]; then
        echo "[ERROR] Missing result or reference size file!"
        return 1
    fi

    sort -o "$result_size_file" "$result_size_file"
    sort -o "$reference_size_file" "$reference_size_file"

    if cmp -s "$result_size_file" "$reference_size_file"; then
        echo "[PASS] Files match - correctness passed for $prog_name ($test_label)"
    else
        echo "[FAIL] Files differ - test failed for $prog_name ($test_label)!"
        diff "$reference_size_file" "$result_size_file" || true
        return 1
    fi
}

run_test() {
    local prog_name="$1"
    local dataset_name="$2"
    local flags="$3"
    local test_label="$4"

    local prog_file=$(basename "$prog_name")
    local prog_path="${PROG_DIR}/flowlog/${prog_file}"
    local prog_url="https://pages.cs.wisc.edu/~m0riarty/program/flowlog/${prog_file}"

    mkdir -p "${PROG_DIR}/flowlog"
    if [ ! -f "$prog_path" ]; then
        echo "[DOWNLOAD] Downloading missing program: $prog_file..."
        wget -O "$prog_path" "$prog_url" || {
            echo "[ERROR] Failed to download program: $prog_file"
            exit 1
        }
    fi

    local fact_path="${FACT_DIR}/${dataset_name}"
    if [ ! -d "$fact_path" ]; then
        echo "[ERROR] Dataset not found after extraction: $fact_path"
        exit 1
    fi

    echo "[TEST] Running $prog_file with $dataset_name ($test_label)"

    rm -rf "$RESULT_DIR/csvs"
    mkdir -p "$RESULT_DIR/csvs"

    local cmd="$BINARY_PATH --program $prog_path --facts $fact_path --csvs $RESULT_DIR --workers $WORKERS"
    if [ -n "$flags" ]; then cmd="$cmd $flags"; fi

    echo "[RUN] Command: RUST_LOG=info $cmd"
    RUST_LOG=info $cmd

    local result_size_file="$RESULT_DIR/csvs/size.txt"
    verify_results "$result_size_file" "$RESULT_DIR/csvs" || {
        echo "[ERROR] Basic verification failed for $prog_file ($test_label)"
        exit 1
    }

    local program_stem="${prog_file%.*}"
    local reference_size_file="${SIZE_DIR}/${program_stem}_${dataset_name}_size.txt"
    if [ -f "$reference_size_file" ]; then
        verify_results_with_reference "$program_stem" "$dataset_name" "$test_label" "$result_size_file" || {
            echo "[ERROR] Reference verification failed for $prog_file ($test_label)"
            exit 1
        }
    else
        echo "[WARN] No reference file found for $program_stem, skipping reference verification"
    fi

    echo "[PASS] Test passed: $prog_file ($test_label)"
}

run_all_tests() {
    echo "[TEST] Running all correctness tests..."
    rm -rf "$RESULT_DIR"

    local sharing_flags=("" "--no-sharing")
    local sharing_labels=("sharing" "no-sharing")

    local optimization_flags=("" "-O1" "-O2" "-O3")
    local optimization_labels=("none" "O1" "O2" "O3")

    while IFS='=' read -r prog_name dataset_name; do
        if [ -z "$prog_name" ] || [ -z "$dataset_name" ]; then
            continue
        fi

        echo "[PROGRAM] Testing $prog_name with $dataset_name"
        echo "========================================"

        # Setup dataset once
        setup_dataset "$dataset_name"

        for i in "${!sharing_flags[@]}"; do
            for j in "${!optimization_flags[@]}"; do
                local combined_flags="${sharing_flags[$i]} ${optimization_flags[$j]}"
                combined_flags=$(echo "$combined_flags" | xargs)

                local test_label="${sharing_labels[$i]}-${optimization_labels[$j]}"
                run_test "$prog_name" "$dataset_name" "$combined_flags" "$test_label"
            done
        done

        # Cleanup dataset after all flags
        cleanup_dataset "$dataset_name"

    done < "$CONFIG_FILE"

    echo "[OK] All correctness tests passed!"
}

# =========================
# MAIN EXECUTION
# =========================

main() {
    echo "[START] FlowLog Correctness Test (Single-dataset Mode)"

    setup_config_file
    setup_size_reference

    echo "[BUILD] Building Present Semiring..."
    cargo build --release

    echo "[RUN] Running correctness tests..."
    run_all_tests

    echo "[FINISH] All correctness tests completed successfully."
}

main "$@"
