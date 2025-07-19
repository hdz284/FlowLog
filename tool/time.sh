#!/bin/bash
set -e

# =========================
# OPTIMIZATION TIMING TEST SCRIPT
# =========================

# =========================
# CONFIGURATION
# =========================

CONFIG_FILE="./test/correctness_test/config.txt"
PROG_DIR="./test/correctness_test/program"
FACT_DIR="./test/correctness_test/dataset"
TIME_DIR="./result/time"
BINARY_PATH="./target/release/executing"
WORKERS=64

# =========================
# DATASET SETUP (per-dataset download)
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

    echo "[CLEANUP] Removing dataset $dataset_name..."
    rm -rf "$extract_path"
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
# TIMING FUNCTIONS
# =========================

run_single_timing_test() {
    local prog_name="$1"
    local dataset_name="$2"
    local optimization_flag="$3"
    local optimization_label="$4"

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
    local program_stem="${prog_name%.*}"
    local time_file="${TIME_DIR}/${program_stem}_${dataset_name}_${optimization_label}.txt"

    echo "[TIMING] Running $prog_name with $dataset_name ($optimization_label)"

    # Ensure time directory exists
    mkdir -p "$TIME_DIR"

    # Run the binary without CSV output (timing will be captured by the binary itself)
    echo "[RUN] Timing test: $prog_name ($optimization_label)"

    if [ -z "$optimization_flag" ]; then
        "$BINARY_PATH" --program "$prog_path" --facts "$fact_path" --workers "$WORKERS"
    else
        "$BINARY_PATH" --program "$prog_path" --facts "$fact_path" --workers "$WORKERS" "$optimization_flag"
    fi

    echo "[TIMING] Completed $prog_name ($optimization_label)"
}

run_all_timing_tests() {
    echo "[TIMING] Running optimization timing tests..."

    local optimizations=("" "-O1" "-O2" "-O3")
    local opt_labels=("none" "1" "2" "3")

    # Clean previous timing results
    rm -rf "$TIME_DIR"
    mkdir -p "$TIME_DIR"

    while IFS='=' read -r prog_name dataset_name; do
        if [ -z "$prog_name" ] || [ -z "$dataset_name" ]; then
            continue
        fi

        echo "[PROGRAM] Timing $prog_name with $dataset_name"
        echo "========================================"

        setup_dataset "$dataset_name"

        for i in "${!optimizations[@]}"; do
            run_single_timing_test "$prog_name" "$dataset_name" "${optimizations[$i]}" "${opt_labels[$i]}"
        done

        cleanup_dataset "$dataset_name"
    done < "$CONFIG_FILE"

    echo "[OK] All timing tests completed!"
}

generate_timing_table() {
    echo ""
    echo "============================"
    echo "[SUMMARY] Timing Results Table"
    echo "============================"

    printf "| %-20s | %-17s | %-17s | %-17s | %-17s |\n" "Program-Dataset" "No Optimization" "O1" "O2" "O3"
    printf "|----------------------|-------------------|-------------------|-------------------|-------------------|\n"

    while IFS='=' read -r prog_name dataset_name; do
        if [ -z "$prog_name" ] || [ -z "$dataset_name" ]; then
            continue
        fi

        local program_stem="${prog_name%.*}"
        local label="${program_stem}_${dataset_name}"
        printf "| %-20s " "$label"

        for opt in "none" "1" "2" "3"; do
            local time_file="${TIME_DIR}/${program_stem}_${dataset_name}_${opt}.txt"
            if [ -f "$time_file" ]; then
                elapsed_time=$(grep -oP '^[0-9]+\.[0-9]+' "$time_file" || echo "             N/A")
            else
                elapsed_time="             N/A"
            fi

            if [[ "$elapsed_time" =~ ^[0-9] ]]; then
                printf "| %17.6f " "$elapsed_time"
            else
                printf "| %-17s " "$elapsed_time"
            fi
        done

        printf "|\n"
    done < "$CONFIG_FILE"
}

generate_timing_csv() {
    echo ""
    echo "[CSV] Generating timing CSV file..."

    local csv_file="${TIME_DIR}/timing_results.csv"

    echo "Program,Dataset,No_Optimization,O1,O2,O3" > "$csv_file"

    while IFS='=' read -r prog_name dataset_name; do
        if [ -z "$prog_name" ] || [ -z "$dataset_name" ]; then
            continue
        fi

        local program_stem="${prog_name%.*}"
        printf "%s,%s" "$program_stem" "$dataset_name" >> "$csv_file"

        for opt in "none" "1" "2" "3"; do
            local time_file="${TIME_DIR}/${program_stem}_${dataset_name}_${opt}.txt"
            if [ -f "$time_file" ]; then
                elapsed_time=$(grep -oP '^[0-9]+\.[0-9]+' "$time_file" || echo "N/A")
            else
                elapsed_time="N/A"
            fi
            printf ",%s" "$elapsed_time" >> "$csv_file"
        done

        printf "\n" >> "$csv_file"
    done < "$CONFIG_FILE"

    echo "[CSV] Timing results saved to: $csv_file"
}

# =========================
# MAIN EXECUTION
# =========================

main() {
    echo "[START] FlowLog Optimization Timing Test"

    setup_config_file

    echo "=== SETUP COMPLETE ==="

    echo "[BUILD] Building the project..."
    cargo build --release

    run_all_timing_tests

    generate_timing_table
    generate_timing_csv

    echo "[FINISH] All timing test cases completed successfully."
}

main "$@"
