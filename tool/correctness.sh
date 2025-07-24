#!/bin/bash
set -e

# Color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

echo -e "${BLUE}[START]${NC} FlowLog Correctness Testing"

# Configuration
CONFIG_FILE="./test/correctness_test/config.txt"
PROG_DIR="./test/correctness_test/program"
FACT_DIR="./test/correctness_test/dataset"
SIZE_DIR="./test/correctness_test/correctness_size"
RESULT_DIR="./result"
BINARY_PATH="./target/release/executing"
WORKERS=64

setup_dataset() {
    local dataset_name="$1" 
    local dataset_zip="./test/correctness_test/dataset/${dataset_name}.zip"
    local extract_path="${FACT_DIR}/${dataset_name}"
    local dataset_url="https://pages.cs.wisc.edu/~m0riarty/dataset/${dataset_name}.zip"

    [ -d "$extract_path" ] && { echo -e "${GREEN}[FOUND]${NC} Dataset $dataset_name"; return; }

    mkdir -p "$FACT_DIR"
    if [ ! -f "$dataset_zip" ]; then
        echo -e "${CYAN}[DOWNLOAD]${NC} $dataset_name.zip"
        wget -q -O "$dataset_zip" "$dataset_url" || { echo -e "${RED}[ERROR]${NC} Download failed: $dataset_name"; exit 1; }
    fi

    echo -e "${YELLOW}[EXTRACT]${NC} $dataset_name"
    unzip -q "$dataset_zip" -d "$FACT_DIR"
}

cleanup_dataset() {
    local dataset_name="$1"
    echo -e "${YELLOW}[CLEANUP]${NC} $dataset_name"
    rm -rf "${FACT_DIR}/${dataset_name}" "${FACT_DIR}/${dataset_name}.zip"
}

setup_config_file() {
    [ -f "$CONFIG_FILE" ] && return
    echo -e "${CYAN}[DOWNLOAD]${NC} config.txt"
    mkdir -p "$(dirname "$CONFIG_FILE")"
    wget -q -O "$CONFIG_FILE" https://pages.cs.wisc.edu/~m0riarty/config.txt
    dos2unix "$CONFIG_FILE" 2>/dev/null || true
}

setup_size_reference() {
    [ -d "$SIZE_DIR" ] && return
    echo -e "${CYAN}[DOWNLOAD]${NC} Reference sizes"
    local zip_path="./test/correctness_test/solution_size.zip"
    mkdir -p ./test/correctness_test
    wget -q -O "$zip_path" https://pages.cs.wisc.edu/~m0riarty/correctness_size.zip
    unzip -q "$zip_path" -d "./test/correctness_test"
    rm "$zip_path"
}

verify_results() {
    local SIZE_FILE="${1:-./result/csvs/size.txt}"
    local CSV_DIR="${2:-./result/csvs}"

    [ ! -f "$SIZE_FILE" ] && { echo -e "${RED}[ERROR]${NC} Size file not found: $SIZE_FILE"; return 1; }

    local pass=true
    while IFS= read -r line; do
        local name="${line%%:*}"
        local expected=$(echo "${line##*:}" | grep -o '[0-9]\+')
        local csv_path="${CSV_DIR}/${name}.csv"

        if [ ! -f "$csv_path" ]; then
            echo -e "${RED}[FAIL]${NC} Missing: $csv_path"
            pass=false
            continue
        fi

        local actual=$(wc -l < "$csv_path")
        if [ "$expected" -eq "$actual" ]; then
            echo -e "${GREEN}[PASS]${NC} $name: $expected"
        else
            echo -e "${RED}[FAIL]${NC} $name: expected=$expected, actual=$actual"
            pass=false
        fi
    done < "$SIZE_FILE"

    [ "$pass" = true ] && return 0 || { echo -e "${RED}[ERROR]${NC} Verification failed"; return 1; }
}

verify_results_with_reference() {
    local prog_name="$1" dataset_name="$2" test_label="$3" result_size_file="$4"
    local reference_size_file="${SIZE_DIR}/${prog_name}_${dataset_name}_size.txt"

    [ ! -f "$result_size_file" ] || [ ! -f "$reference_size_file" ] && {
        echo -e "${RED}[ERROR]${NC} Missing files for reference check"
        return 1
    }

    sort -o "$result_size_file" "$result_size_file"
    sort -o "$reference_size_file" "$reference_size_file"

    if cmp -s "$result_size_file" "$reference_size_file"; then
        echo -e "${GREEN}[PASS]${NC} Reference check: $prog_name ($test_label)"
    else
        echo -e "${RED}[FAIL]${NC} Reference mismatch: $prog_name ($test_label)"
        return 1
    fi
}

run_test() {
    local prog_name="$1" dataset_name="$2" flags="$3" test_label="$4"
    local prog_file=$(basename "$prog_name")
    local prog_path="${PROG_DIR}/flowlog/${prog_file}"
    local prog_url="https://pages.cs.wisc.edu/~m0riarty/program/flowlog/${prog_file}"

    # Download program if needed
    mkdir -p "${PROG_DIR}/flowlog"
    if [ ! -f "$prog_path" ]; then
        echo -e "${CYAN}[DOWNLOAD]${NC} $prog_file"
        wget -q -O "$prog_path" "$prog_url" || { echo -e "${RED}[ERROR]${NC} Download failed: $prog_file"; exit 1; }
    fi

    [ ! -d "${FACT_DIR}/${dataset_name}" ] && { echo -e "${RED}[ERROR]${NC} Dataset not found: $dataset_name"; exit 1; }

    echo -e "${BLUE}[TEST]${NC} $prog_file with $dataset_name ($test_label)"

    # Prepare result directory
    rm -rf "$RESULT_DIR/csvs"
    mkdir -p "$RESULT_DIR/csvs"

    # Run test
    local cmd="$BINARY_PATH --program $prog_path --facts ${FACT_DIR}/${dataset_name} --csvs $RESULT_DIR --workers $WORKERS"
    [ -n "$flags" ] && cmd="$cmd $flags"
    
    # print out the command to be executed
    echo -e "${YELLOW}[RUNNING]${NC} $cmd"
    RUST_LOG=info $cmd  # >/dev/null if you want to suppress output

    # Verify results
    local result_size_file="$RESULT_DIR/csvs/size.txt"
    verify_results "$result_size_file" "$RESULT_DIR/csvs" || { echo -e "${RED}[ERROR]${NC} Verification failed: $prog_file ($test_label)"; exit 1; }

    # Check against reference if available
    local program_stem="${prog_file%.*}"
    local reference_size_file="${SIZE_DIR}/${program_stem}_${dataset_name}_size.txt"
    if [ -f "$reference_size_file" ]; then
        verify_results_with_reference "$program_stem" "$dataset_name" "$test_label" "$result_size_file" || {
            echo -e "${RED}[ERROR]${NC} Reference check failed: $prog_file ($test_label)"
            exit 1
        }
    fi
}

run_all_tests() {
    echo -e "${BLUE}[TESTS]${NC} Running all correctness tests"
    rm -rf "$RESULT_DIR"

    # Test configurations
    local optimization_flags=("" "-O1" "-O2" "-O3")
    local optimization_labels=("O0" "O1" "O2" "O3")

    while IFS='=' read -r prog_name dataset_name; do
        [ -z "$prog_name" ] || [ -z "$dataset_name" ] && continue

        echo -e "${CYAN}[PROGRAM]${NC} $prog_name with $dataset_name"
        setup_dataset "$dataset_name"

        # Test all optimization flag combinations (sharing always enabled)
        for j in "${!optimization_flags[@]}"; do
            local flags="${optimization_flags[$j]}"
            local test_label="sharing-${optimization_labels[$j]}"
            run_test "$prog_name" "$dataset_name" "$flags" "$test_label"
        done

        cleanup_dataset "$dataset_name"
    done < "$CONFIG_FILE"

    echo -e "${GREEN}[COMPLETE]${NC} All tests passed"
}

# Main execution
echo -e "${YELLOW}[BUILD]${NC} Building binary"
cargo build --release >/dev/null

setup_config_file
setup_size_reference
run_all_tests

echo -e "${GREEN}[FINISH]${NC} Correctness testing completed successfully"
