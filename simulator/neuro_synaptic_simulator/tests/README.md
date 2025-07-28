# RUV-FANN Simulator Tests

This directory contains comprehensive integration tests and performance benchmarks for the RUV-FANN Neuro-Synaptic Simulator.

## Structure

```
tests/
├── integration/       # Integration tests
│   ├── cli_tests.rs   # CLI interface tests
│   └── parallel_tests.rs # 256-core parallel execution tests
├── benchmarks/        # Performance benchmarks
│   └── performance_bench.rs # Criterion benchmarks
└── fixtures/          # Test data
    └── wasm_modules/  # WASM test modules
        ├── add.wat    # Simple addition
        ├── multiply.wat # Simple multiplication
        ├── neural_net.wat # Neural network simulation
        ├── parallel_compute.wat # Parallel computation
        ├── memory_intensive.wat # Memory stress test
        └── compute.wat # General computation
```

## Running Tests

### Integration Tests

```bash
# Run all integration tests
cargo test

# Run specific test module
cargo test --test integration

# Run with verbose output
cargo test -- --nocapture

# Run specific test
cargo test test_256_core_utilization
```

### Performance Benchmarks

```bash
# Run all benchmarks
cargo bench

# Run specific benchmark group
cargo bench simulator_operations

# Generate HTML reports (saved to target/criterion/)
cargo bench -- --verbose
```

## Test Coverage

### CLI Tests (`cli_tests.rs`)
- Basic CLI invocation and help
- Version information
- Running simple WASM modules
- 256-core parallel execution
- Custom configuration files
- Batch processing
- Error handling for invalid WASM
- Memory limit enforcement
- Visualization output
- Performance profiling

### Parallel Execution Tests (`parallel_tests.rs`)
- Verifies 256-core utilization
- Core scaling performance (1 to 256 cores)
- Parallel WASM module execution
- Memory consistency across threads
- Error handling in parallel execution
- Load balancing
- Concurrent simulator access

### Performance Benchmarks (`performance_bench.rs`)
- Simulator operations with various input sizes
- Parallel scaling (1, 4, 16, 64, 128, 256 cores)
- Neural network forward passes
- WASM module execution
- Memory allocation patterns
- Visualization rendering
- Batch processing
- Activation function performance

## Building WASM Test Modules

The test WASM modules are written in WAT (WebAssembly Text) format. To compile them:

```bash
# Install wabt tools if needed
sudo apt-get install wabt  # Ubuntu/Debian
brew install wabt          # macOS

# Build all WASM modules
cd tests/fixtures
./build_wasm.sh
```

## Expected Performance

Based on benchmarks with 256-core parallel execution:

- **Single Input Processing**: < 1ms
- **Batch Processing (1M items)**: < 100ms with 256 cores
- **Neural Network (256x512x256x128x64)**: < 10ms forward pass
- **WASM Execution**: < 0.1ms per module
- **Scaling Efficiency**: > 70% up to 64 cores

## Troubleshooting

### Test Failures

1. **WASM modules not found**: Run `build_wasm.sh` to compile WAT files
2. **Memory limit exceeded**: Increase test memory limits in configuration
3. **Parallel test flakiness**: May indicate race conditions - check synchronization

### Performance Issues

1. **Poor scaling**: Check CPU core availability and system load
2. **Slow benchmarks**: Ensure release mode compilation (`cargo bench` uses release by default)
3. **High memory usage**: Monitor with `cargo bench -- --profile-time 10`

## Adding New Tests

1. **Integration Tests**: Add new test functions to existing files or create new modules
2. **Benchmarks**: Add benchmark functions to `performance_bench.rs`
3. **WASM Modules**: Create new `.wat` files and update `build_wasm.sh`

## Continuous Integration

These tests are designed to run in CI/CD pipelines:

```yaml
# Example GitHub Actions workflow
- name: Run tests
  run: |
    cargo test --all-features
    cargo bench --no-run  # Just build benchmarks
```