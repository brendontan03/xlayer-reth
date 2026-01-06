#!/usr/bin/env bash
set -euo pipefail

# Compile Time Benchmarking Script
# This script measures compile times for the xlayer-reth-node binary
# with and without custom linker configuration.

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
CARGO_CONFIG="$PROJECT_ROOT/.cargo/config.toml"
BACKUP_CONFIG="$PROJECT_ROOT/.cargo/config.toml.backup"
RESULTS_FILE="$PROJECT_ROOT/compile-benchmark-results.txt"
TEMP_TIMING_1="$PROJECT_ROOT/.timing1.tmp"
TEMP_TIMING_2="$PROJECT_ROOT/.timing2.tmp"

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
RED='\033[0;31m'
NC='\033[0m' # No Color

print_header() {
    echo -e "${BLUE}===================================================${NC}"
    echo -e "${BLUE}$1${NC}"
    echo -e "${BLUE}===================================================${NC}"
}

print_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

# Convert time string (e.g., "5m9.398s") to seconds
time_to_seconds() {
    local time_str="$1"
    local seconds=0

    # Extract minutes if present
    if [[ $time_str =~ ([0-9]+)m ]]; then
        seconds=$((seconds + ${BASH_REMATCH[1]} * 60))
    fi

    # Extract seconds
    if [[ $time_str =~ ([0-9]+\.[0-9]+)s ]]; then
        seconds=$(echo "$seconds + ${BASH_REMATCH[1]}" | bc)
    elif [[ $time_str =~ ([0-9]+)s ]]; then
        seconds=$((seconds + ${BASH_REMATCH[1]}))
    fi

    echo "$seconds"
}

# Function to clean build artifacts
clean_build() {
    print_info "Cleaning build artifacts..."
    cargo clean
}

# Function to measure compile time
measure_compile() {
    local label="$1"
    local profile="${2:-release}"
    local output_file="$3"

    print_header "Benchmarking: $label (profile: $profile)"

    # Clean before each build for fair comparison
    clean_build

    # Measure time using bash time
    print_info "Running compilation..."
    { time cargo build --profile "$profile" -p xlayer-reth-node; } 2>&1 | tee "$output_file"

    echo ""
}

# Parse timing output
parse_timing() {
    local file="$1"
    local real_time=$(grep "^real" "$file" | awk '{print $2}')
    local user_time=$(grep "^user" "$file" | awk '{print $2}')
    local sys_time=$(grep "^sys" "$file" | awk '{print $2}')

    echo "$real_time|$user_time|$sys_time"
}

# Display side-by-side comparison
show_comparison() {
    local timing1="$1"
    local timing2="$2"

    IFS='|' read -r real1 user1 sys1 <<< "$timing1"
    IFS='|' read -r real2 user2 sys2 <<< "$timing2"

    # Convert to seconds for calculation
    local real1_sec=$(time_to_seconds "$real1")
    local real2_sec=$(time_to_seconds "$real2")
    local user1_sec=$(time_to_seconds "$user1")
    local user2_sec=$(time_to_seconds "$user2")
    local sys1_sec=$(time_to_seconds "$sys1")
    local sys2_sec=$(time_to_seconds "$sys2")

    # Calculate improvements
    local real_diff=$(echo "$real1_sec - $real2_sec" | bc)
    local user_diff=$(echo "$user1_sec - $user2_sec" | bc)
    local sys_diff=$(echo "$sys1_sec - $sys2_sec" | bc)

    local real_pct=$(echo "scale=2; ($real_diff / $real1_sec) * 100" | bc)
    local user_pct=$(echo "scale=2; ($user_diff / $user1_sec) * 100" | bc)

    print_header "Side-by-Side Comparison"

    printf "${CYAN}%-20s${NC} │ ${YELLOW}%-15s${NC} │ ${GREEN}%-15s${NC} │ %-15s\n" \
        "Metric" "Default Linker" "Custom Linker" "Improvement"
    echo "────────────────────────────────────────────────────────────────────────"

    # Real time
    if (( $(echo "$real_diff > 0" | bc -l) )); then
        printf "%-20s │ %-15s │ %-15s │ ${GREEN}-%s (%.1f%%)${NC}\n" \
            "Wall time" "$real1" "$real2" "${real_diff}s" "$real_pct"
    else
        printf "%-20s │ %-15s │ %-15s │ ${RED}+%s (%.1f%%)${NC}\n" \
            "Wall time" "$real1" "$real2" "${real_diff#-}s" "${real_pct#-}"
    fi

    # User time
    if (( $(echo "$user_diff > 0" | bc -l) )); then
        printf "%-20s │ %-15s │ %-15s │ ${GREEN}-%s (%.1f%%)${NC}\n" \
            "User CPU time" "$user1" "$user2" "${user_diff}s" "$user_pct"
    else
        printf "%-20s │ %-15s │ %-15s │ ${RED}+%s (%.1f%%)${NC}\n" \
            "User CPU time" "$user1" "$user2" "${user_diff#-}s" "${user_pct#-}"
    fi

    # System time
    if (( $(echo "$sys_diff > 0" | bc -l) )); then
        printf "%-20s │ %-15s │ %-15s │ ${GREEN}-%ss${NC}\n" \
            "System CPU time" "$sys1" "$sys2" "$sys_diff"
    else
        printf "%-20s │ %-15s │ %-15s │ ${RED}+%ss${NC}\n" \
            "System CPU time" "$sys1" "$sys2" "${sys_diff#-}"
    fi

    echo ""

    # Summary
    if (( $(echo "$real_diff > 0" | bc -l) )); then
        echo -e "${GREEN}✓ Custom linker is faster by ${real_diff}s (${real_pct}%)${NC}"
    else
        echo -e "${RED}✗ Custom linker is slower by ${real_diff#-}s (${real_pct#-}%)${NC}"
    fi
}

# Main benchmarking flow
main() {
    cd "$PROJECT_ROOT"

    # Initialize results file
    {
        echo "Compile Time Benchmark Results"
        echo "Generated at: $(date)"
        echo "System: $(uname -a)"
        echo "Rust version: $(rustc --version)"
        echo ""
    } > "$RESULTS_FILE"

    # Benchmark 1: Without custom linker
    print_header "Phase 1: Baseline (default linker)"
    if [ -f "$CARGO_CONFIG" ]; then
        print_info "Backing up existing .cargo/config.toml..."
        mv "$CARGO_CONFIG" "$BACKUP_CONFIG"
    fi

    measure_compile "Default Linker" "release" "$TEMP_TIMING_1"

    # Benchmark 2: With custom linker (mold/lld)
    print_header "Phase 2: With Custom Linker"
    if [ -f "$BACKUP_CONFIG" ]; then
        print_info "Restoring .cargo/config.toml with custom linker configuration..."
        mv "$BACKUP_CONFIG" "$CARGO_CONFIG"
    else
        print_warning ".cargo/config.toml not found! Make sure it exists with linker configuration."
        exit 1
    fi

    measure_compile "Custom Linker (mold/lld)" "release" "$TEMP_TIMING_2"

    # Parse timings
    timing1=$(parse_timing "$TEMP_TIMING_1")
    timing2=$(parse_timing "$TEMP_TIMING_2")

    # Show comparison
    show_comparison "$timing1" "$timing2"

    # Save to results file
    {
        echo ""
        echo "DETAILED RESULTS"
        echo "================"
        echo ""
        echo "Default Linker:"
        grep -E "^(real|user|sys)" "$TEMP_TIMING_1"
        echo ""
        echo "Custom Linker (mold/lld):"
        grep -E "^(real|user|sys)" "$TEMP_TIMING_2"
    } >> "$RESULTS_FILE"

    print_info "Full results saved to: $RESULTS_FILE"

    # Cleanup temp files
    rm -f "$TEMP_TIMING_1" "$TEMP_TIMING_2"
}

# Run the benchmark
main "$@"
