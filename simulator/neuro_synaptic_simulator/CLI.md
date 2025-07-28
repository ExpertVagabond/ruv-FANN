# Neuro-Synaptic Simulator CLI Reference

Complete command-line interface documentation for the Neuro-Synaptic Chip Simulator.

## Synopsis

```
neuro_synaptic_simulator [GLOBAL_OPTIONS] <COMMAND> [COMMAND_OPTIONS]
```

## Global Options

These options apply to all commands:

### `-v, --verbose`
Enable verbose logging output. Provides detailed information about simulation progress, memory allocation, and timing.

**Example:**
```bash
neuro_synaptic_simulator -v run -w model.wasm
```

### `-c, --cores <COUNT>`
Number of cores to simulate (1-256). Default: 256

**Example:**
```bash
# Simulate with 128 cores
neuro_synaptic_simulator --cores 128 run -w model.wasm

# Minimal single-core simulation
neuro_synaptic_simulator -c 1 run -w tiny_model.wasm
```

### `-m, --memory <SIZE>`
Memory size in MB (1-28). Default: 28

**Example:**
```bash
# Simulate with reduced memory (16MB)
neuro_synaptic_simulator --memory 16 run -w small_model.wasm
```

### `--help`
Display help information

### `--version`
Display version information

## Commands

### `run` - Execute Neural Network Simulation

Run a neural network model compiled to WebAssembly on the simulated chip.

**Usage:**
```
neuro_synaptic_simulator run [OPTIONS]
```

**Options:**

#### `-w, --wasm-module <PATH>` (required)
Path to the WebAssembly module containing the neural network model.

```bash
neuro_synaptic_simulator run -w models/resnet50.wasm
```

#### `-t, --timesteps <COUNT>`
Number of timesteps to simulate. Default: 1000

```bash
# Run for 5000 timesteps
neuro_synaptic_simulator run -w model.wasm -t 5000
```

#### `-o, --output <FILE>`
Output file for JSON logs. If not specified, summary is printed to stdout.

```bash
# Export detailed logs
neuro_synaptic_simulator run -w model.wasm -o simulation_logs.json
```

**Full Example:**
```bash
neuro_synaptic_simulator -v --cores 256 run \
    --wasm-module models/bert_base.wasm \
    --timesteps 10000 \
    --output bert_simulation.json
```

**Output Format:**

When using `-o`, the JSON output follows this schema:

```json
{
  "simulation": {
    "version": "0.1.0",
    "timestamp": "2024-01-15T10:30:00Z",
    "configuration": {
      "cores": 256,
      "memory_mb": 28,
      "wasm_module": "model.wasm"
    },
    "results": {
      "total_timesteps": 1000,
      "execution_time_ms": 45.3,
      "throughput_fps": 22.1,
      "power_usage_watts": 1.87
    },
    "events": [
      {
        "timestamp_us": 0,
        "core_id": 0,
        "event_type": "task_start",
        "details": {}
      }
    ]
  }
}
```

### `test` - Run Test Suite

Execute the built-in test suite to verify simulator functionality.

**Usage:**
```
neuro_synaptic_simulator test [OPTIONS]
```

**Options:**

#### `-f, --filter <PATTERN>`
Run only tests matching the specified pattern.

```bash
# Run only memory tests
neuro_synaptic_simulator test --filter memory

# Run specific test
neuro_synaptic_simulator test -f "test_shared_memory_allocation"
```

#### `-b, --bench`
Enable benchmark mode for performance testing.

```bash
# Run performance benchmarks
neuro_synaptic_simulator test --bench
```

**Examples:**
```bash
# Run all tests
neuro_synaptic_simulator test

# Run memory subsystem tests in benchmark mode
neuro_synaptic_simulator test -f memory -b

# Run with verbose output
neuro_synaptic_simulator -v test
```

### `verify` - System Verification

Check system configuration and verify all subsystems are functioning correctly.

**Usage:**
```
neuro_synaptic_simulator verify [OPTIONS]
```

**Options:**

#### `--wasm`
Verify WASM runtime functionality.

```bash
neuro_synaptic_simulator verify --wasm
```

#### `--memory`
Verify memory subsystem configuration.

```bash
neuro_synaptic_simulator verify --memory
```

#### `--all`
Run all verification checks.

```bash
neuro_synaptic_simulator verify --all
```

**Examples:**
```bash
# Basic verification
neuro_synaptic_simulator verify

# Full system check with verbose output
neuro_synaptic_simulator -v verify --all

# Check specific subsystem
neuro_synaptic_simulator verify --wasm
```

**Verification Output:**
```
[INFO] Verifying system configuration...
[✓] WASM runtime: Wasmtime 17.0.0
[✓] SIMD support: Enabled
[✓] Memory allocation: 28MB available
[✓] Thread pool: 16 workers
[✓] All systems operational
```

## Environment Variables

The simulator respects several environment variables:

### `RUST_LOG`
Control logging verbosity. Overrides `-v` flag.

```bash
# Set trace-level logging
RUST_LOG=trace neuro_synaptic_simulator run -w model.wasm

# Set module-specific logging
RUST_LOG=neuro_synaptic_simulator::wasm=debug neuro_synaptic_simulator run -w model.wasm
```

### `RAYON_NUM_THREADS`
Control the number of threads in the thread pool.

```bash
# Use 8 threads regardless of CPU count
RAYON_NUM_THREADS=8 neuro_synaptic_simulator run -w model.wasm
```

### `NSS_MEMORY_LIMIT`
Override the memory limit (in MB).

```bash
# Force 16MB limit
NSS_MEMORY_LIMIT=16 neuro_synaptic_simulator run -w small_model.wasm
```

## Exit Codes

The simulator uses standard exit codes:

- `0`: Success
- `1`: General error
- `2`: Invalid arguments
- `3`: WASM module load failure
- `4`: Out of memory
- `5`: Verification failure

## Advanced Usage

### Piping and Redirection

```bash
# Pipe JSON output to analysis tool
neuro_synaptic_simulator run -w model.wasm -o - | jq '.results'

# Save both stdout and stderr
neuro_synaptic_simulator -v run -w model.wasm 2>&1 | tee simulation.log
```

### Batch Processing

```bash
# Process multiple models
for model in models/*.wasm; do
    neuro_synaptic_simulator run -w "$model" -o "${model%.wasm}_results.json"
done
```

### Integration with Scripts

```bash
#!/bin/bash
# benchmark.sh - Compare different core counts

for cores in 32 64 128 256; do
    echo "Testing with $cores cores..."
    time neuro_synaptic_simulator --cores $cores run \
        -w model.wasm \
        -t 1000 \
        -o "results_${cores}cores.json"
done
```

### Docker Usage

```dockerfile
FROM rust:1.70
COPY . /app
WORKDIR /app
RUN cargo build --release
ENTRYPOINT ["./target/release/neuro_synaptic_simulator"]
```

```bash
# Run in container
docker run -v $(pwd)/models:/models neurochip-sim run -w /models/model.wasm
```

## Performance Tips

1. **Use Release Build**: Always use `--release` for production simulations
2. **Optimize WASM**: Pre-optimize WASM modules with `wasm-opt`
3. **Adjust Core Count**: Match simulated cores to available CPU threads
4. **Memory Mapping**: Ensure models fit within memory constraints
5. **Batch Operations**: Process multiple inferences in one simulation

## Troubleshooting

### Common Errors

**"WASM module not found"**
- Check file path is correct
- Ensure file has `.wasm` extension
- Verify file permissions

**"Out of memory"**
- Reduce model size or use quantization
- Lower the number of simulated cores
- Use `--memory` flag to adjust limit

**"Simulation timeout"**
- Reduce timesteps with `-t` flag
- Check for infinite loops in WASM
- Enable verbose logging to debug

### Debug Commands

```bash
# Maximum verbosity
RUST_LOG=trace neuro_synaptic_simulator -v run -w model.wasm

# Test WASM loading only
neuro_synaptic_simulator verify --wasm

# Memory usage analysis
neuro_synaptic_simulator -v verify --memory
```

## See Also

- [README.md](README.md) - Project overview
- [QUICKSTART.md](QUICKSTART.md) - Quick start guide
- [Architecture Documentation](../docs/architecture.md) - Technical details
- [CONTRIBUTING.md](CONTRIBUTING.md) - Contribution guidelines