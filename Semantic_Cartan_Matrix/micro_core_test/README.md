# Micro Core - Semantic Cartan Matrix for rUv-FANN

A `no_std` Rust implementation of the Semantic Cartan Matrix architecture for integration with rUv-FANN neural networks.

## Features

- **32-dimensional root vector space** with SIMD alignment for efficient computation
- **MicroNet trait** for neural network agents with rank-1 routing optimization
- **Projection and embedding functions** for dimensional reduction
- **Streaming updates** using Oja's algorithm for online adaptation
- **WASM support** for browser deployment
- **No unsafe code** - fully memory safe implementation

## Architecture

The Semantic Cartan Matrix provides:

1. **Explicit symmetry** - Orthogonal semantic axes reduce interference between micro-nets
2. **Built-in compression** - Rank-1 heads provide efficient routing layers
3. **Interpretability hooks** - Fixed 32-root lattice enables decision tracing
4. **Physics-style regularizers** - Energy minimization aligns with thermodynamic governance

## Usage

```rust
use micro_core::{RuvFannBridge, project_to_root, RootSpace};

// Create the integration bridge
let mut bridge = RuvFannBridge::new();

// Project high-dimensional tokens to root space
let token = vec![0.5; 768]; // 768-dim token
let root_vec = project_to_root(&token, &bridge.root_space);

// Create specialized agents
let routing_agent = bridge.create_routing_agent(1);
let reasoning_agent = bridge.create_reasoning_agent(2);

// Step regularization during training
bridge.step_regularization(epoch);

// Export metrics for visualization
let metrics = bridge.export_metrics();
println!("Cartan loss: {}", metrics.cartan_loss);
```

## Building

### Native build:
```bash
cargo build --release
```

### WASM build:
```bash
cargo build --release --target wasm32-unknown-unknown --features wasm
```

### Optimized WASM:
```bash
RUSTFLAGS="-C target-feature=+simd128" cargo build --release --target wasm32-unknown-unknown --features wasm,simd
wasm-opt -O3 target/wasm32-unknown-unknown/release/micro_core.wasm -o micro_core.wasm
```

## Integration with rUv-FANN

The crate provides a plug-in interface via the `RuvFannBridge`:

1. Initialize the bridge with your desired configuration
2. Use `project_to_root()` to transform tokens to the semantic space
3. Create agents with `create_routing_agent()` or `create_reasoning_agent()`
4. Monitor training with `export_metrics()` for dashboard visualization

## Testing

Run the test suite:
```bash
cargo test
cargo test --features wasm
```

Run benchmarks:
```bash
cargo bench
```

## License

Licensed under either Apache-2.0 or MIT at your option.