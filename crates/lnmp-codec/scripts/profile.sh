#!/bin/bash
# Performance profiling script for LNMP v0.5

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CODEC_DIR="$(dirname "$SCRIPT_DIR")"
PROJECT_ROOT="$(dirname "$(dirname "$CODEC_DIR")")"

cd "$PROJECT_ROOT"

echo "=== LNMP v0.5 Performance Profiling ==="
echo ""

# Function to check if a command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Function to run benchmarks
run_benchmarks() {
    echo "Running benchmarks..."
    cargo bench --bench v05_performance --manifest-path crates/lnmp-codec/Cargo.toml
    echo ""
    echo "Benchmark results saved to target/criterion/"
    echo "Open target/criterion/report/index.html to view detailed results"
}

# Function to generate flamegraph
generate_flamegraph() {
    if ! command_exists flamegraph; then
        echo "flamegraph not found. Install with: cargo install flamegraph"
        return 1
    fi
    
    echo "Generating flamegraph..."
    cargo flamegraph --bench v05_performance --manifest-path crates/lnmp-codec/Cargo.toml
    echo ""
    echo "Flamegraph saved to flamegraph.svg"
}

# Function to run specific benchmark group
run_specific() {
    local group=$1
    echo "Running benchmark group: $group"
    cargo bench --bench v05_performance --manifest-path crates/lnmp-codec/Cargo.toml -- "$group"
}

# Function to compare benchmarks
compare_benchmarks() {
    if ! command_exists critcmp; then
        echo "critcmp not found. Install with: cargo install critcmp"
        return 1
    fi
    
    echo "Comparing benchmarks..."
    echo "First, save baseline:"
    echo "  cargo bench --bench v05_performance --save-baseline before"
    echo ""
    echo "Then, after making changes:"
    echo "  cargo bench --bench v05_performance --save-baseline after"
    echo ""
    echo "Finally, compare:"
    echo "  critcmp before after"
}

# Main menu
case "${1:-}" in
    "bench")
        run_benchmarks
        ;;
    "flame")
        generate_flamegraph
        ;;
    "nested")
        run_specific "nested"
        ;;
    "streaming")
        run_specific "streaming"
        ;;
    "delta")
        run_specific "delta"
        ;;
    "v04_vs_v05")
        run_specific "v04_vs_v05"
        ;;
    "compare")
        compare_benchmarks
        ;;
    "help"|*)
        echo "Usage: $0 [command]"
        echo ""
        echo "Commands:"
        echo "  bench         - Run all benchmarks"
        echo "  flame         - Generate flamegraph (requires cargo-flamegraph)"
        echo "  nested        - Run nested structure benchmarks only"
        echo "  streaming     - Run streaming benchmarks only"
        echo "  delta         - Run delta encoding benchmarks only"
        echo "  v04_vs_v05    - Run v0.4 vs v0.5 comparison benchmarks"
        echo "  compare       - Show how to compare benchmark results"
        echo "  help          - Show this help message"
        echo ""
        echo "Examples:"
        echo "  $0 bench                    # Run all benchmarks"
        echo "  $0 flame                    # Generate flamegraph"
        echo "  $0 nested                   # Run nested benchmarks only"
        ;;
esac
