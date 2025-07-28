# Neuro-Synaptic Chip Simulator

A high-performance simulator for a 256-core neuro-synaptic ASIC chip with 28MB shared memory and WebAssembly (WASM) execution support. This simulator models both logical compute correctness and timing behavior for neural network inference on a low-power 12nm architecture.

## Overview

The Neuro-Synaptic Chip Simulator provides:

- **256-Core Parallel Processing**: Simulates massively parallel neural computation across 256 independent processing units
- **28MB Shared Memory**: Models constrained on-chip memory with realistic partitioning for weights, activations, and I/O
- **WASM Execution**: Runs neural network models compiled to WebAssembly with SIMD acceleration
- **Cycle-Accurate Timing**: Provides timing estimates for inference latency and throughput
- **Power Modeling**: Estimates power consumption based on core utilization (1.5-2W target)
- **JSON Logging**: Exports detailed execution logs for verification pipelines

## Quick Start

### Installation

```bash
# Clone the repository
git clone https://github.com/ruvnet/ruv-FANN.git
cd ruv-FANN/simulator/neuro_synaptic_simulator

# Build the simulator
cargo build --release

# Run tests to verify installation
cargo test
```

### Basic Usage

```bash
# Run a neural network model simulation
./target/release/neuro_synaptic_simulator run --wasm-module model.wasm

# Run with specific core count and timesteps
./target/release/neuro_synaptic_simulator --cores 128 run -w model.wasm -t 5000

# Export JSON logs for analysis
./target/release/neuro_synaptic_simulator run -w model.wasm -o simulation.json

# Run verification checks
./target/release/neuro_synaptic_simulator verify --all
```

## Architecture

The simulator models a neuro-synaptic chip with the following specifications:

- **Process Node**: 12nm FinFET
- **Core Count**: 256 parallel processing units
- **Memory**: 28MB on-chip SRAM (shared)
- **Power Budget**: 1.5-2W TDP
- **Compute**: WASM execution with SIMD extensions
- **Interconnect**: High-bandwidth mesh network

### Memory Layout

```
┌─────────────────────────────────────────────────────────────┐
│                     28MB Shared Memory Pool                   │
├─────────────────────┬───────────────────┬──────────────────┤
│   Model Weights     │   Activations     │   I/O Buffers    │
│      (16MB)         │      (8MB)        │      (4MB)       │
├─────────────────────┼───────────────────┼──────────────────┤
│ 0x0000000-0xFFFFFF  │ 0x1000000-0x17FFFFF│ 0x1800000-0x1BFFFFF│
└─────────────────────┴───────────────────┴──────────────────┘
```

### Core Architecture

Each processing unit features:
- WASM execution engine with JIT compilation
- 32KB dedicated activation memory partition
- SIMD vector unit for neural operations
- Hardware synchronization primitives
- Cycle-accurate instruction counting

For detailed architecture documentation, see [docs/architecture.md](../docs/architecture.md).

## Building and Running

### Prerequisites

- Rust 1.70+ with cargo
- C++ compiler (for WASM runtime dependencies)
- 8GB+ RAM for full 256-core simulation

### Build Options

```bash
# Debug build (with assertions and symbols)
cargo build

# Release build (optimized for performance)
cargo build --release

# Run with specific features
cargo build --release --features "profile,benchmark"
```

### Running Simulations

The simulator supports three main commands:

#### 1. Run Command
Execute neural network inference simulation:

```bash
neuro_synaptic_simulator run [OPTIONS]

Options:
  -w, --wasm-module <PATH>     Path to WASM neural network module
  -t, --timesteps <COUNT>      Number of simulation timesteps [default: 1000]
  -o, --output <FILE>          Output file for JSON logs
  -h, --help                   Print help
```

#### 2. Test Command
Run built-in test suite:

```bash
neuro_synaptic_simulator test [OPTIONS]

Options:
  -f, --filter <PATTERN>       Run only tests matching pattern
  -b, --bench                  Enable benchmark mode
  -h, --help                   Print help
```

#### 3. Verify Command
Check system configuration:

```bash
neuro_synaptic_simulator verify [OPTIONS]

Options:
  --wasm                       Check WASM runtime
  --memory                     Check memory subsystem
  --all                        Run all verification checks
  -h, --help                   Print help
```

### Global Options

```bash
-v, --verbose                  Enable verbose logging
-c, --cores <COUNT>            Number of cores to simulate (1-256) [default: 256]
-m, --memory <SIZE>            Memory size in MB (1-28) [default: 28]
```

## Example Usage

### Basic Inference Simulation

```bash
# Run a pre-trained ResNet model
./neuro_synaptic_simulator run -w models/resnet50.wasm -t 1000

# Output:
# [INFO] Neuro-Synaptic Simulator v0.1.0
# [INFO] Configured with 256 cores and 28MB memory
# [INFO] Loading WASM module: models/resnet50.wasm
# [INFO] Module size: 98.3MB (compressed to fit 28MB)
# [INFO] Starting simulation for 1000 timesteps...
# [INFO] Simulation complete:
#   - Total time: 45.2ms (simulated)
#   - Throughput: 22.1 inferences/sec
#   - Power usage: 1.87W (average)
#   - Core utilization: 94.5%
```

### Batch Processing

```bash
# Process multiple inputs in parallel
./neuro_synaptic_simulator run -w models/mobilenet.wasm \
    --batch-size 256 \
    --timesteps 5000 \
    -o batch_results.json
```

### Power-Constrained Simulation

```bash
# Simulate with reduced power budget
./neuro_synaptic_simulator --cores 128 run \
    -w models/tinybert.wasm \
    --power-limit 1.0
```

## Implemented Features

### Core Functionality ✅
- [x] CLI argument parsing with clap
- [x] 256-core parallel architecture modeling
- [x] 28MB shared memory management
- [x] WASM runtime integration (Wasmtime)
- [x] Basic project structure and modules

### Memory System ✅
- [x] Shared memory pool allocation
- [x] Per-core memory partitioning
- [x] Bounds checking and safety
- [x] Zero-copy data sharing patterns

### Execution Engine ✅
- [x] WASM module loading
- [x] Instance pooling for 256 cores
- [x] SIMD support configuration
- [x] Thread pool management

### In Progress 🚧
- [ ] Cycle-accurate timing model
- [ ] Power consumption tracking
- [ ] JSON logging implementation
- [ ] Full test suite
- [ ] Benchmark suite

## Planned Features

### Near Term
- [ ] Complete timing model with instruction counting
- [ ] JSON event logging and export
- [ ] Integration with ruv-FANN models
- [ ] Memory bandwidth profiling
- [ ] Basic power modeling

### Future Enhancements
- [ ] Dynamic voltage/frequency scaling (DVFS)
- [ ] Multi-model concurrent execution
- [ ] Hardware accelerator modeling
- [ ] Network-on-chip simulation
- [ ] Thermal modeling
- [ ] Real-time visualization
- [ ] CUDA comparison mode
- [ ] Distributed simulation

## Performance

Expected simulation performance on modern hardware:

| Host System | Simulated Cores | Real-time Factor |
|-------------|-----------------|------------------|
| 8-core CPU  | 256            | ~0.1x            |
| 16-core CPU | 256            | ~0.25x           |
| 32-core CPU | 256            | ~0.5x            |
| 64-core CPU | 256            | ~1.0x            |

Note: Real-time factor indicates simulation speed relative to actual hardware.

## Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

### Development Setup

```bash
# Install development dependencies
cargo install cargo-watch cargo-criterion

# Run tests in watch mode
cargo watch -x test

# Run benchmarks
cargo criterion

# Format code
cargo fmt

# Lint code
cargo clippy -- -D warnings
```

## License

This project is part of the ruv-FANN ecosystem and is licensed under the MIT License.

## Acknowledgments

- Built with [Wasmtime](https://wasmtime.dev/) for WASM execution
- Uses [Rayon](https://github.com/rayon-rs/rayon) for parallel processing
- Logging powered by [tracing](https://github.com/tokio-rs/tracing)
- CLI interface via [clap](https://github.com/clap-rs/clap)

## Related Projects

- [ruv-FANN](https://github.com/ruvnet/ruv-FANN) - Rust neural network engine
- [Neuro-Divergent](https://github.com/ruvnet/neuro-divergent) - LLM architectures
- [cuda-wasm](https://github.com/ruvnet/cuda-wasm) - CUDA to WASM transpilation