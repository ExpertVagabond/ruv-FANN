# Neuro-Synaptic Simulator Quick Start Guide

Get up and running with the Neuro-Synaptic Chip Simulator in 5 minutes!

## Installation

```bash
# Clone and build
git clone https://github.com/ruvnet/ruv-FANN.git
cd ruv-FANN/simulator/neuro_synaptic_simulator
cargo build --release
```

## Basic Examples

### 1. Hello Neural Network

Run a simple feedforward network:

```bash
# Download example model
wget https://example.com/models/simple_mlp.wasm

# Run simulation
./target/release/neuro_synaptic_simulator run -w simple_mlp.wasm
```

Expected output:
```
[INFO] Loading WASM module: simple_mlp.wasm
[INFO] Model: 3-layer MLP (784-128-10)
[INFO] Memory usage: 412KB weights, 1.2MB activations
[INFO] Simulating on 256 cores...
[INFO] Inference completed in 0.23ms
[INFO] Throughput: 4347 inferences/sec
```

### 2. Parallel Batch Processing

Process 256 images simultaneously:

```bash
# Run with batch processing
./target/release/neuro_synaptic_simulator run \
    -w models/mobilenet_v2.wasm \
    --batch 256 \
    -o batch_results.json

# View results
cat batch_results.json | jq '.summary'
```

### 3. Power-Aware Simulation

Simulate with power constraints:

```bash
# Run with half the cores (lower power)
./target/release/neuro_synaptic_simulator \
    --cores 128 \
    run -w models/efficientnet.wasm

# Compare full vs reduced power
./quickstart_power_compare.sh
```

### 4. Real-Time Inference

Simulate real-time constraints:

```bash
# Run with timing requirements (10ms deadline)
./target/release/neuro_synaptic_simulator run \
    -w models/yolo_tiny.wasm \
    --real-time \
    --deadline-ms 10
```

## Common Use Cases

### CNN Image Classification

```bash
# ResNet-50 inference
./target/release/neuro_synaptic_simulator run \
    -w models/resnet50.wasm \
    --input images/cat.bin \
    -o resnet_output.json

# Extract predictions
cat resnet_output.json | jq '.predictions[:5]'
```

### RNN Text Processing

```bash
# LSTM sentiment analysis
./target/release/neuro_synaptic_simulator run \
    -w models/lstm_sentiment.wasm \
    --input "This product is amazing!" \
    --text-mode
```

### Transformer Models

```bash
# BERT-tiny inference
./target/release/neuro_synaptic_simulator run \
    -w models/bert_tiny.wasm \
    --sequence-length 128 \
    --memory 28
```

## Performance Testing

### Quick Benchmark

```bash
# Run built-in benchmarks
./target/release/neuro_synaptic_simulator test --bench

# Results:
# ┌─────────────────┬───────────┬────────────┬───────────┐
# │ Model           │ Latency   │ Throughput │ Power     │
# ├─────────────────┼───────────┼────────────┼───────────┤
# │ MobileNet V2    │ 2.3ms     │ 435 fps    │ 1.2W      │
# │ ResNet-50       │ 8.7ms     │ 115 fps    │ 1.8W      │
# │ BERT-tiny       │ 15.2ms    │ 66 fps     │ 1.9W      │
# └─────────────────┴───────────┴────────────┴───────────┘
```

### Custom Benchmark

```bash
# Create benchmark script
cat > benchmark.sh << 'EOF'
#!/bin/bash
for cores in 64 128 256; do
    echo "Testing with $cores cores..."
    ./target/release/neuro_synaptic_simulator \
        --cores $cores \
        run -w model.wasm \
        -t 10000 \
        -o results_${cores}.json
done
EOF

chmod +x benchmark.sh
./benchmark.sh
```

## Debugging

### Verbose Logging

```bash
# Enable debug output
./target/release/neuro_synaptic_simulator -v run \
    -w model.wasm

# Super verbose (trace level)
RUST_LOG=trace ./target/release/neuro_synaptic_simulator run \
    -w model.wasm
```

### Memory Analysis

```bash
# Check memory usage
./target/release/neuro_synaptic_simulator verify --memory

# Analyze memory layout
./target/release/neuro_synaptic_simulator run \
    -w model.wasm \
    --dump-memory-map
```

### Timing Analysis

```bash
# Export detailed timing
./target/release/neuro_synaptic_simulator run \
    -w model.wasm \
    --timing-detail \
    -o timing.json

# Visualize timeline
python scripts/visualize_timeline.py timing.json
```

## Integration Examples

### With ruv-FANN

```bash
# Compile ruv-FANN model to WASM
cd ../ruv-FANN
cargo build --target wasm32-unknown-unknown --release
wasm-opt -O3 target/wasm32-unknown-unknown/release/model.wasm \
    -o model_opt.wasm

# Run in simulator
cd ../simulator/neuro_synaptic_simulator
./target/release/neuro_synaptic_simulator run -w model_opt.wasm
```

### With Python

```python
import subprocess
import json

# Run simulation from Python
result = subprocess.run([
    './target/release/neuro_synaptic_simulator',
    'run', '-w', 'model.wasm',
    '-o', 'output.json'
], capture_output=True)

# Parse results
with open('output.json') as f:
    data = json.load(f)
    print(f"Latency: {data['timing']['latency_ms']}ms")
    print(f"Power: {data['power']['average_watts']}W")
```

### CI/CD Integration

```yaml
# .github/workflows/simulate.yml
name: Neural Network Simulation

on: [push]

jobs:
  simulate:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v2
    - uses: actions-rs/toolchain@v1
    - name: Build simulator
      run: |
        cd simulator/neuro_synaptic_simulator
        cargo build --release
    - name: Run tests
      run: |
        ./target/release/neuro_synaptic_simulator test
    - name: Benchmark models
      run: |
        ./target/release/neuro_synaptic_simulator run \
          -w models/production.wasm \
          -o results.json
    - name: Check performance
      run: |
        python scripts/check_performance.py results.json
```

## Tips and Tricks

### 1. Optimize WASM Modules

```bash
# Use wasm-opt for smaller, faster modules
wasm-opt -O3 -c input.wasm -o optimized.wasm

# Enable SIMD
wasm-opt --enable-simd input.wasm -o simd.wasm
```

### 2. Profile Performance

```bash
# Generate flamegraph
cargo flamegraph --bin neuro_synaptic_simulator -- \
    run -w model.wasm
```

### 3. Reduce Memory Usage

```bash
# Use quantized models
python scripts/quantize_model.py model.onnx model_int8.wasm

# Run with reduced precision
./target/release/neuro_synaptic_simulator run \
    -w model_int8.wasm \
    --precision int8
```

### 4. Parallel Development

```bash
# Run multiple simulations in parallel
parallel -j 4 ./target/release/neuro_synaptic_simulator run \
    -w model.wasm -o result_{}.json ::: 1 2 3 4
```

## Troubleshooting

### Common Issues

**Issue**: "Out of memory" error
```bash
# Solution: Reduce batch size or use fewer cores
./target/release/neuro_synaptic_simulator --cores 64 run -w model.wasm
```

**Issue**: "WASM module too large"
```bash
# Solution: Compress or quantize model
wasm-opt --strip-debug large_model.wasm -o smaller_model.wasm
```

**Issue**: "Simulation too slow"
```bash
# Solution: Use release build and enable optimizations
cargo build --release --features "fast-math,parallel"
RUSTFLAGS="-C target-cpu=native" cargo build --release
```

## Next Steps

1. Check out the [examples/](examples/) directory for more complex scenarios
2. Read the [architecture documentation](../docs/architecture.md)
3. Join our [Discord community](https://discord.gg/neurochip)
4. Contribute to the [project](CONTRIBUTING.md)

Happy simulating! 🚀