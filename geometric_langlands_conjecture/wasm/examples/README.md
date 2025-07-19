# Geometric Langlands WASM Examples

This directory contains examples demonstrating the **currently working features** of the geometric-langlands-wasm package.

## ⚠️ Important Note

This package is in **alpha stage**. These examples show only the features that are currently implemented and tested. Many theoretical features are still under development.

## Running Examples

### Prerequisites

1. Build the WASM package:
```bash
wasm-pack build --target web --out-dir pkg
```

2. Serve the examples:
```bash
python3 -m http.server 8000
# or
npx http-server .
```

3. Open in browser:
- Basic example: http://localhost:8000/examples/basic.html
- Interactive demo: http://localhost:8000/web/

## Available Examples

### 1. basic.js - Core Features
Demonstrates:
- ✅ Creating reductive groups (GL, SL, Sp, SO)
- ✅ Creating automorphic forms (cuspidal, Eisenstein)
- ✅ Creating Galois representations
- ✅ Computing L-function values
- ✅ Basic correspondence framework

### 2. web/index.html - Interactive Demo
Full browser demonstration with:
- ✅ Interactive parameter selection
- ✅ Real-time computation
- ✅ Performance metrics
- ✅ Basic visualizations

## What's Working

### Mathematical Objects
- **Reductive Groups**: GL(n), SL(n), Sp(2n), SO(n) with basic properties
- **Automorphic Forms**: Cuspidal forms and Eisenstein series construction
- **Galois Representations**: Artin and Weil-Deligne representations
- **L-functions**: Basic Dirichlet and Artin L-function evaluation

### Computations
- Group rank and dimension calculations
- L-function value computation at specific points
- Basic correspondence framework (simplified version)
- WASM performance optimizations

## What's Not Yet Implemented

The following features are planned but not yet available:
- Advanced geometric structures (Shimura varieties, etc.)
- Complete spectral decomposition algorithms
- Full categorical equivalences
- GPU acceleration via WebGPU
- Complex mathematical visualizations
- Complete test coverage

## Performance Notes

Current performance characteristics:
- WASM bundle size: ~80KB (optimized)
- Initialization time: <100ms
- Basic computations: <10ms
- Memory efficient for small rank groups

## Troubleshooting

### WASM Not Loading
- Ensure you're serving files from a web server (not file://)
- Check browser console for CORS errors
- Verify wasm-pack build completed successfully

### Computation Errors
- Keep group dimensions small (≤10) for now
- Some theoretical computations may throw "not implemented" errors
- Check browser console for detailed error messages

## Contributing

This is research software in active development. Contributions welcome, especially:
- Implementation of planned features
- Performance optimizations
- Additional examples
- Bug fixes

See [CONTRIBUTING.md](../../../CONTRIBUTING.md) for details.