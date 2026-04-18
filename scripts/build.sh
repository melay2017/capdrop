#!/bin/bash
# Build script for macOS
set -e

echo "=== CapDrop macOS Build ==="

# Check prerequisites
command -v cargo >/dev/null 2>&1 || { echo "Error: cargo not found"; exit 1; }
command -v pnpm >/dev/null 2>&1 || { echo "Error: pnpm not found"; exit 1; }

# Install frontend deps
echo "[1/3] Installing frontend dependencies..."
pnpm install

# Build
echo "[2/3] Building Tauri app..."
cargo tauri build --manifest-path src-tauri/Cargo.toml

echo "[3/3] Build complete!"
echo "Output: src-tauri/target/release/bundle/"
