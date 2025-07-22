#!/bin/bash
# Exit immediately if a command exits with a non-zero status
set -e

############################################################
# OPTIMIZATION TIMING TEST SCRIPT
# This script measures execution time for FlowLog programs
# with different optimization levels (-O1, -O2, -O3)
# 
# Execution logs are saved to ./log/ directory
# Timing information is extracted from the "Dataflow executed" log line
# Results are generated as table and CSV in the log directory
############################################################

############################################################
# CONFIGURATION
# Define paths and parameters for timing tests
############################################################

CONFIG_FILE="./test/correctness_test/config.txt"      # Program/dataset pairs configuration
PROG_DIR="./test/correctness_test/program"            # Program files directory
FACT_DIR="./test/correctness_test/dataset"            # Dataset files directory
LOG_DIR="./result/log"                                 # Log output directory
BINARY_PATH="./target/release/executing"               # Path to compiled binary
WORKERS=64                                             # Number of worker threads

############################################################
# DATASET SETUP
# Functions to download, extract, and clean up datasets
############################################################

setup_dataset() {
    # Download and extract dataset if not already present
    local dataset_name="$1"
    local dataset_zip="./test/correctness_test/dataset/${dataset_name}.zip"
    local extract_path="${FACT_DIR}/${dataset_name}"
    local dataset_url="https://pages.cs.wisc.edu/~m0riarty/dataset/${dataset_name}.zip"

    # Check if dataset is already extracted
    if [ -d "$extract_path" ]; then
        echo "[OK] Dataset $dataset_name already extracted. Skipping."
        return
    fi

    mkdir -p "$FACT_DIR"

    # Download dataset if zip file doesn't exist
    if [ ! -f "$dataset_zip" ]; then
        echo "[DOWNLOAD] Downloading $dataset_name.zip from $dataset_url..."
        mkdir -p "$(dirname "$dataset_zip")"
        wget -O "$dataset_zip" "$dataset_url" || {
            echo "[ERROR] Failed to download dataset: $dataset_name"
            exit 1
        }
    fi

    # Extract the dataset
    echo "[EXTRACT] Extracting $dataset_name..."
    unzip -q "$dataset_zip" -d "$FACT_DIR"
    echo "[OK] Dataset $dataset_name ready."
}

cleanup_dataset() {
    # Remove extracted dataset to save space
    local dataset_name="$1"
    local extract_path="${FACT_DIR}/${dataset_name}"

    echo "[CLEANUP] Removing dataset $dataset_name..."
    rm -rf "$extract_path"
}

setup_config_file() {
    # Download config.txt if missing and fix line endings
    if [ -f "$CONFIG_FILE" ]; then
        echo "[OK] Config file already exists. Skipping download."
        return
    fi

    echo "[DOWNLOAD] Downloading config.txt..."
    mkdir -p "$(dirname "$CONFIG_FILE")"
    wget -O "$CONFIG_FILE" https://pages.cs.wisc.edu/~m0riarty/config.txt

    # Fix line endings (convert DOS to Unix format)
    echo "[FIX] Fixing line endings in config.txt..."
    dos2unix "$CONFIG_FILE" 2>/dev/null || true
    echo "[OK] Config file ready."
}

############################################################
# TIMING FUNCTIONS
# Functions to run timing tests and measure performance
############################################################

run_single_timing_test() {
    # Run a single timing test for a program/dataset/optimization combination
    local prog_name="$1"
    local dataset_name="$2"
    local optimization_flag="$3"
    local optimization_label="$4"

    # Set up program file paths and URLs
    local prog_file=$(basename "$prog_name")
    local prog_path="${PROG_DIR}/flowlog/${prog_file}"
    local prog_url="https://pages.cs.wisc.edu/~m0riarty/program/flowlog/${prog_file}"

    # Download program file if it doesn't exist
    mkdir -p "${PROG_DIR}/flowlog"
    if [ ! -f "$prog_path" ]; then
        echo "[DOWNLOAD] Downloading missing program: $prog_file..."
        wget -O "$prog_path" "$prog_url" || {
            echo "[ERROR] Failed to download program: $prog_file"
            exit 1
        }
    fi
    
    # Set up paths for timing test
    local fact_path="${FACT_DIR}/${dataset_name}"
    local program_stem="${prog_name%.*}"
    local log_file="${LOG_DIR}/${program_stem}_${dataset_name}_${optimization_label}.log"

    echo "[TIMING] Running $prog_name with $dataset_name ($optimization_label)"

    # Ensure log directory exists
    mkdir -p "$LOG_DIR"

    # Run the binary with specified optimization flag and capture output to log
    echo "[RUN] Timing test: $prog_name ($optimization_label)"

    if [ -z "$optimization_flag" ]; then
        RUST_LOG=info "$BINARY_PATH" --program "$prog_path" --facts "$fact_path" --workers "$WORKERS" > "$log_file" 2>&1
    else
        RUST_LOG=info "$BINARY_PATH" --program "$prog_path" --facts "$fact_path" --workers "$WORKERS" "$optimization_flag" > "$log_file" 2>&1
    fi

    echo "[TIMING] Completed $prog_name ($optimization_label)"
}

run_all_timing_tests() {
    # Run timing tests for all programs with all optimization levels
    echo "[TIMING] Running optimization timing tests..."

    # Define optimization flags and labels
    local optimizations=("" "-O1" "-O2" "-O3")
    local opt_labels=("none" "1" "2" "3")

    # Clean previous logs
    rm -rf "$LOG_DIR"
    mkdir -p "$LOG_DIR"

    # Read each program=dataset pair from config file
    while IFS='=' read -r prog_name dataset_name; do
        if [ -z "$prog_name" ] || [ -z "$dataset_name" ]; then
            continue
        fi

        echo "[PROGRAM] Timing $prog_name with $dataset_name"
        echo "========================================"

        # Setup dataset once for all optimization levels
        setup_dataset "$dataset_name"

        # Run tests with all optimization levels
        for i in "${!optimizations[@]}"; do
            run_single_timing_test "$prog_name" "$dataset_name" "${optimizations[$i]}" "${opt_labels[$i]}"
        done

        # Cleanup dataset after all optimization tests
        cleanup_dataset "$dataset_name"
    done < "$CONFIG_FILE"

    echo "[OK] All timing tests completed!"
}

############################################################
# TIMING EXTRACTION FUNCTIONS
# Functions to extract timing information from log files
############################################################

extract_time_from_log() {
    # Extract timing information from log file by parsing "Dataflow executed" line
    local log_file="$1"
    
    if [ ! -f "$log_file" ]; then
        echo "N/A"
        return
    fi
    
    # Look for the "Dataflow executed" line and extract the duration
    # Format: "2025-07-22T21:41:00.527157Z  INFO executing::dataflow: 3.933584239s:	Dataflow executed"
    local time_line=$(grep "Dataflow executed" "$log_file" 2>/dev/null | tail -1)
    
    if [ -z "$time_line" ]; then
        echo "N/A"
        return
    fi
    
    # Extract time value using grep and sed
    # Look for pattern like "3.933584239s:" and remove the "s:"
    local extracted_time=$(echo "$time_line" | grep -oE '[0-9]+\.[0-9]+s:' | sed 's/s://' 2>/dev/null || echo "N/A")
    echo "$extracted_time"
}

############################################################
# RESULT GENERATION FUNCTIONS
# Functions to generate timing results table and CSV
############################################################

generate_timing_table() {
    # Generate and display a formatted table of timing results
    echo ""
    echo "============================"
    echo "[SUMMARY] Timing Results Table"
    echo "============================"

    printf "| %-20s | %-17s | %-17s | %-17s | %-17s |\n" "Program-Dataset" "No Optimization" "O1" "O2" "O3"
    printf "|----------------------|-------------------|-------------------|-------------------|-------------------|\n"

    # Read each program=dataset pair and display timing results
    while IFS='=' read -r prog_name dataset_name; do
        if [ -z "$prog_name" ] || [ -z "$dataset_name" ]; then
            continue
        fi

        local program_stem="${prog_name%.*}"
        local label="${program_stem}_${dataset_name}"
        printf "| %-20s " "$label"

        # Display timing for each optimization level
        for opt in "none" "1" "2" "3"; do
            local log_file="${LOG_DIR}/${program_stem}_${dataset_name}_${opt}.log"
            elapsed_time=$(extract_time_from_log "$log_file")

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
    # Generate CSV file with timing results for analysis
    echo ""
    echo "[CSV] Generating timing CSV file..."

    local csv_file="${LOG_DIR}/timing_results.csv"

    # Write CSV header
    echo "Program,Dataset,No_Optimization,O1,O2,O3" > "$csv_file"

    # Read each program=dataset pair and write timing data
    while IFS='=' read -r prog_name dataset_name; do
        if [ -z "$prog_name" ] || [ -z "$dataset_name" ]; then
            continue
        fi

        local program_stem="${prog_name%.*}"
        printf "%s,%s" "$program_stem" "$dataset_name" >> "$csv_file"

        # Write timing data for each optimization level
        for opt in "none" "1" "2" "3"; do
            local log_file="${LOG_DIR}/${program_stem}_${dataset_name}_${opt}.log"
            elapsed_time=$(extract_time_from_log "$log_file")
            printf ",%s" "$elapsed_time" >> "$csv_file"
        done

        printf "\n" >> "$csv_file"
    done < "$CONFIG_FILE"

    echo "[CSV] Timing results saved to: $csv_file"
}

############################################################
# MAIN EXECUTION
# Entry point for the script
############################################################

main() {
    # Print start message
    echo "[START] FlowLog Optimization Timing Test"

    # Ensure config file is present
    setup_config_file

    echo "=== SETUP COMPLETE ==="

    # Build the Rust binary
    echo "[BUILD] Building the project..."
    cargo build --release

    # Run all timing tests
    run_all_timing_tests

    # Generate results in table and CSV format
    generate_timing_table
    generate_timing_csv

    # Print finish message
    echo "[FINISH] All timing test cases completed successfully."
}

# Call main function with all script arguments
main "$@"
