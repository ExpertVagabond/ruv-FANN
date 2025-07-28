#!/bin/bash
# Build WASM test fixtures from WAT files

echo "Building WASM test fixtures..."

# Check if wat2wasm is available
if ! command -v wat2wasm &> /dev/null; then
    echo "wat2wasm not found. Installing wabt..."
    # Try to install wabt if not present
    if command -v apt-get &> /dev/null; then
        sudo apt-get update && sudo apt-get install -y wabt
    elif command -v brew &> /dev/null; then
        brew install wabt
    else
        echo "Please install wabt (WebAssembly Binary Toolkit) manually"
        exit 1
    fi
fi

# Compile all WAT files to WASM
for wat_file in wasm_modules/*.wat; do
    if [ -f "$wat_file" ]; then
        wasm_file="${wat_file%.wat}.wasm"
        echo "Compiling $wat_file -> $wasm_file"
        wat2wasm "$wat_file" -o "$wasm_file" --enable-simd
    fi
done

echo "WASM fixtures built successfully!"