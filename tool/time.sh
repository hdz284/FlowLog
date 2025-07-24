#!/bin/bash
set -e

echo "[START] FlowLog Optimization Timing Test"

# Configuration
CONFIG_FILE="./test/correctness_test/config.txt"
PROG_DIR="./test/correctness_test/program"
FACT_DIR="./test/correctness_test/dataset"
TIME_DIR="./result/time"
BINARY_PATH="./target/release/executing"
WORKERS=64

setup_dataset() {
    local dataset_name="$1"
    local dataset_zip="./test/correctness_test/dataset/${dataset_name}.zip"
    local extract_path="${FACT_DIR}/${dataset_name}"
    local dataset_url="https://pages.cs.wisc.edu/~m0riarty/dataset/${dataset_name}.zip"

    [ -d "$extract_path" ] && { echo "[OK] Dataset $dataset_name already extracted"; return; }

    mkdir -p "$FACT_DIR"
    if [ ! -f "$dataset_zip" ]; then
        echo "[DOWNLOAD] $dataset_name.zip"
        wget -q -O "$dataset_zip" "$dataset_url" || { echo "[ERROR] Download failed: $dataset_name"; exit 1; }
    fi

    echo "[EXTRACT] $dataset_name"
    unzip -q "$dataset_zip" -d "$FACT_DIR"
}

cleanup_dataset() {
    local dataset_name="$1"
    echo "[CLEANUP] $dataset_name"
    rm -rf "${FACT_DIR}/${dataset_name}"
}

setup_config_file() {
    [ -f "$CONFIG_FILE" ] && return
    echo "[DOWNLOAD] config.txt"
    mkdir -p "$(dirname "$CONFIG_FILE")"
    wget -q -O "$CONFIG_FILE" https://pages.cs.wisc.edu/~m0riarty/config.txt
    dos2unix "$CONFIG_FILE" 2>/dev/null || true
}

run_timing_test() {
    local prog_name="$1" dataset_name="$2" optimization_flag="$3" optimization_label="$4"
    local prog_file=$(basename "$prog_name")
    local prog_path="${PROG_DIR}/flowlog/${prog_file}"
    local prog_url="https://pages.cs.wisc.edu/~m0riarty/program/flowlog/${prog_file}"

    # Download program if needed
    mkdir -p "${PROG_DIR}/flowlog"
    if [ ! -f "$prog_path" ]; then
        echo "[DOWNLOAD] $prog_file"
        wget -q -O "$prog_path" "$prog_url" || { echo "[ERROR] Download failed: $prog_file"; exit 1; }
    fi

    echo "[TIMING] $prog_name with $dataset_name ($optimization_label)"

    # Run the binary with optimization flag
    if [ -z "$optimization_flag" ]; then
        "$BINARY_PATH" --program "$prog_path" --facts "${FACT_DIR}/${dataset_name}" --workers "$WORKERS"
    else
        "$BINARY_PATH" --program "$prog_path" --facts "${FACT_DIR}/${dataset_name}" --workers "$WORKERS" "$optimization_flag"
    fi
}

run_all_timing_tests() {
    echo "[TIMING] Running optimization timing tests"
    rm -rf "$TIME_DIR"
    mkdir -p "$TIME_DIR"

    # Define optimization flags and labels
    local optimizations=("" "-O1" "-O2" "-O3")
    local opt_labels=("none" "1" "2" "3")

    while IFS='=' read -r prog_name dataset_name; do
        [ -z "$prog_name" ] || [ -z "$dataset_name" ] && continue

        echo "[PROGRAM] $prog_name with $dataset_name"
        setup_dataset "$dataset_name"

        # Run tests with all optimization levels
        for i in "${!optimizations[@]}"; do
            run_timing_test "$prog_name" "$dataset_name" "${optimizations[$i]}" "${opt_labels[$i]}"
        done

        cleanup_dataset "$dataset_name"
    done < "$CONFIG_FILE"

    echo "[OK] All timing tests completed"
}

generate_timing_table() {
    echo ""
    echo "[SUMMARY] Timing Results Table"
    echo "=============================="

    printf "| %-20s | %-17s | %-17s | %-17s | %-17s |\n" "Program-Dataset" "O0" "O1" "O2" "O3"
    printf "|----------------------|-------------------|-------------------|-------------------|-------------------|\n"

    while IFS='=' read -r prog_name dataset_name; do
        [ -z "$prog_name" ] || [ -z "$dataset_name" ] && continue

        local program_stem="${prog_name%.*}"
        local label="${program_stem}_${dataset_name}"
        printf "| %-20s " "$label"

        # Display timing for each optimization level
        for opt in "none" "1" "2" "3"; do
            local time_file="${TIME_DIR}/${program_stem}_${dataset_name}_${opt}.txt"
            if [ -f "$time_file" ]; then
                elapsed_time=$(grep -oP '^[0-9]+\.[0-9]+' "$time_file" 2>/dev/null || echo "N/A")
            else
                elapsed_time="N/A"
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
    echo "[CSV] Generating timing CSV file"

    local csv_file="${TIME_DIR}/timing_results.csv"
    echo "Program,Dataset,O0,O1,O2,O3" > "$csv_file"

    while IFS='=' read -r prog_name dataset_name; do
        [ -z "$prog_name" ] || [ -z "$dataset_name" ] && continue

        local program_stem="${prog_name%.*}"
        printf "%s,%s" "$program_stem" "$dataset_name" >> "$csv_file"

        # Write timing data for each optimization level
        for opt in "none" "1" "2" "3"; do
            local time_file="${TIME_DIR}/${program_stem}_${dataset_name}_${opt}.txt"
            if [ -f "$time_file" ]; then
                elapsed_time=$(grep -oP '^[0-9]+\.[0-9]+' "$time_file" 2>/dev/null || echo "N/A")
            else
                elapsed_time="N/A"
            fi
            printf ",%s" "$elapsed_time" >> "$csv_file"
        done
        printf "\n" >> "$csv_file"
    done < "$CONFIG_FILE"

    echo "[CSV] Timing results saved to: $csv_file"
}

# Main execution
echo "[BUILD] Building binary"
cargo build --release >/dev/null

setup_config_file
run_all_timing_tests
generate_timing_table
generate_timing_csv

echo "[FINISH] All timing tests completed successfully"