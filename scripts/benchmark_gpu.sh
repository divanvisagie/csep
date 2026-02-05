#!/usr/bin/env bash
set -euo pipefail

# Benchmark csep cache build: CPU (ONNX) vs CUDA (Qwen3)
#
# Builds two release binaries — one default (ONNX) and one with --features cuda (Qwen3).
# Uses hyperfine to compare cold cache build and warm cache search times.
#
# Prerequisites:
#   - hyperfine installed
#   - CUDA toolkit available (for the cuda build)
#
# Usage:
#   ./scripts/benchmark_gpu.sh [target_directory]

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
TARGET_DIR="${1:-.}"

CPU_BIN="$REPO_DIR/target/release/csep-cpu"
CUDA_BIN="$REPO_DIR/target/release/csep-cuda"

if ! command -v hyperfine &>/dev/null; then
    echo "Error: hyperfine is not installed. Install with: cargo install hyperfine"
    exit 1
fi

echo "=== Building CPU (ONNX) binary ==="
cargo build --release --manifest-path "$REPO_DIR/Cargo.toml" 2>&1
cp "$REPO_DIR/target/release/csep" "$CPU_BIN"

echo ""
echo "=== Building CUDA (Qwen3) binary ==="
cargo build --release --features cuda --manifest-path "$REPO_DIR/Cargo.toml" 2>&1
cp "$REPO_DIR/target/release/csep" "$CUDA_BIN"

echo ""
echo "=== Benchmarking cache build in: $TARGET_DIR ==="
echo ""

CLEAR_CACHE="$CPU_BIN cache --clear 2>/dev/null || true"

hyperfine \
    --warmup 0 \
    --min-runs 3 \
    --prepare "$CLEAR_CACHE" \
    --command-name "CPU (ONNX)" \
    "cd $TARGET_DIR && $CPU_BIN cache --build" \
    --command-name "CUDA (Qwen3)" \
    "cd $TARGET_DIR && $CUDA_BIN cache --build"

echo ""
echo "=== Benchmarking search (warm cache) ==="
echo ""

# Build caches for both backends
eval "$CLEAR_CACHE"
(cd "$TARGET_DIR" && "$CPU_BIN" cache --build 2>/dev/null)
(cd "$TARGET_DIR" && "$CUDA_BIN" cache --build 2>/dev/null)

QUERY="main entry point"

hyperfine \
    --warmup 3 \
    --command-name "CPU (ONNX)" \
    "cd $TARGET_DIR && $CPU_BIN '$QUERY'" \
    --command-name "CUDA (Qwen3)" \
    "cd $TARGET_DIR && $CUDA_BIN '$QUERY'"

# Clean up
rm -f "$CPU_BIN" "$CUDA_BIN"
