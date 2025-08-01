#!/bin/bash
# Build script for WASM with SIMD optimizations

set -e

echo "🚀 Building Neuro-Synaptic Simulator for WASM with SIMD..."

# Check if wasm-pack is installed
if ! command -v wasm-pack &> /dev/null; then
    echo "❌ wasm-pack not found. Installing..."
    curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
fi

# Clean previous builds
echo "🧹 Cleaning previous builds..."
rm -rf pkg/
rm -rf target/wasm32-unknown-unknown/

# Build with SIMD enabled
echo "🔨 Building with SIMD optimizations..."
RUSTFLAGS="-C target-feature=+simd128" wasm-pack build \
    --target web \
    --out-dir pkg \
    --release \
    -- --features wasm-simd

# Optimize WASM size
echo "📦 Optimizing WASM size..."
if command -v wasm-opt &> /dev/null; then
    wasm-opt -Oz \
        --enable-simd \
        pkg/neuro_synaptic_simulator_bg.wasm \
        -o pkg/neuro_synaptic_simulator_bg_opt.wasm
    
    # Replace original with optimized version
    mv pkg/neuro_synaptic_simulator_bg_opt.wasm pkg/neuro_synaptic_simulator_bg.wasm
    echo "✅ WASM optimization complete"
else
    echo "⚠️  wasm-opt not found. Skipping optimization."
    echo "   Install with: npm install -g wasm-opt"
fi

# Generate TypeScript types
echo "📝 Generating TypeScript definitions..."
if [ -f pkg/neuro_synaptic_simulator.d.ts ]; then
    echo "✅ TypeScript definitions generated"
else
    echo "⚠️  TypeScript definitions not generated"
fi

# Create package.json for npm publishing
echo "📦 Creating package.json..."
cat > pkg/package.json << EOF
{
  "name": "@ruv-fann/neuro-synaptic-simulator",
  "version": "0.1.0",
  "description": "High-performance neuro-synaptic chip simulator with WASM and SIMD support",
  "main": "neuro_synaptic_simulator.js",
  "types": "neuro_synaptic_simulator.d.ts",
  "files": [
    "neuro_synaptic_simulator_bg.wasm",
    "neuro_synaptic_simulator.js",
    "neuro_synaptic_simulator.d.ts",
    "neuro_synaptic_simulator_bg.js",
    "neuro_synaptic_simulator_bg.wasm.d.ts"
  ],
  "keywords": [
    "neural-network",
    "wasm",
    "simd",
    "neuro-synaptic",
    "simulation"
  ],
  "author": "Neuro-Synaptic Simulator Team",
  "license": "MIT",
  "repository": {
    "type": "git",
    "url": "https://github.com/ruvnet/ruv-FANN.git"
  }
}
EOF

# Display build info
echo "📊 Build Summary:"
echo "   - WASM size: $(du -h pkg/neuro_synaptic_simulator_bg.wasm | cut -f1)"
echo "   - JS size: $(du -h pkg/neuro_synaptic_simulator.js | cut -f1)"
echo "   - Files generated:"
ls -la pkg/

echo "✅ Build complete! Output in ./pkg/"
echo ""
echo "📚 Usage:"
echo "   import init, { WasmSimulator } from './pkg/neuro_synaptic_simulator.js';"
echo "   await init();"
echo "   const sim = new WasmSimulator(config);"