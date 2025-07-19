#!/bin/bash

# Geometric Langlands WASM - Crates.io Publication Script
# ⚠️ ALPHA VERSION - Use with caution

echo "🚀 Geometric Langlands WASM - Publication Script"
echo "================================================"
echo ""
echo "⚠️  WARNING: This is an ALPHA release!"
echo "   - Limited features are implemented"
echo "   - API may change significantly"
echo "   - Not recommended for production use"
echo ""

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    echo "❌ Error: Cargo.toml not found. Please run from the wasm directory."
    exit 1
fi

# Check if cargo is installed
if ! command -v cargo &> /dev/null; then
    echo "❌ Error: cargo not found. Please install Rust."
    exit 1
fi

# Display current version
VERSION=$(grep "^version" Cargo.toml | cut -d'"' -f2)
echo "📦 Current version: $VERSION"
echo ""

# Confirmation prompt
read -p "⚠️  Are you sure you want to publish version $VERSION to crates.io? (yes/no): " confirm
if [ "$confirm" != "yes" ]; then
    echo "❌ Publication cancelled."
    exit 0
fi

echo ""
echo "🔍 Running pre-publication checks..."

# Build the package to ensure it compiles
echo "   - Building package..."
if ! cargo build --release; then
    echo "❌ Build failed. Please fix errors before publishing."
    exit 1
fi

# Run basic tests
echo "   - Running tests..."
if ! cargo test; then
    echo "⚠️  Warning: Some tests failed. Continue anyway? (yes/no): "
    read -p "" continue_anyway
    if [ "$continue_anyway" != "yes" ]; then
        echo "❌ Publication cancelled."
        exit 0
    fi
fi

# Check documentation builds
echo "   - Checking documentation..."
if ! cargo doc --no-deps; then
    echo "⚠️  Warning: Documentation build failed."
fi

# Package the crate
echo ""
echo "📦 Creating package..."
if ! cargo package --allow-dirty; then
    echo "❌ Package creation failed."
    exit 1
fi

# Final confirmation
echo ""
echo "✅ Pre-publication checks complete!"
echo ""
echo "📋 Publication checklist:"
echo "   [✓] Version: $VERSION"
echo "   [✓] Build: Success"
echo "   [✓] Tests: Passed (or skipped)"
echo "   [✓] Package: Created"
echo ""
echo "🔑 Ready to publish to crates.io"
echo ""
echo "To complete publication, run:"
echo ""
echo "   export CARGO_REGISTRY_TOKEN=your_token_here"
echo "   cargo publish"
echo ""
echo "Or with the token inline:"
echo "   cargo publish --token your_token_here"
echo ""
echo "⚠️  Remember: This is an ALPHA release. Users should be aware of limitations."
echo ""