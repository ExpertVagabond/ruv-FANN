# micro_core

[![Crates.io](https://img.shields.io/crates/v/micro_core.svg)](https://crates.io/crates/micro_core)
[![Documentation](https://docs.rs/micro_core/badge.svg)](https://docs.rs/micro_core)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![Build Status](https://github.com/ruvnet/ruv-FANN/workflows/CI/badge.svg)](https://github.com/ruvnet/ruv-FANN/actions)

**Core neural operations and embedding for the Semantic Cartan Matrix system**

The `micro_core` crate provides fundamental types, traits, and operations for building modular micro-neural networks with orthogonal semantic embeddings. It forms the foundation of the rUv-FANN Semantic Cartan Matrix architecture.

## 🚀 Features

- **32-dimensional Root Space**: SIMD-aligned orthogonal vector embeddings
- **MicroNet Trait**: Standard interface for neural network agents
- **Projection Operations**: High-dimensional to root space mapping
- **no_std Compatible**: Embedded and WebAssembly deployment
- **SIMD Optimizations**: Platform-specific performance enhancements
- **rUv-FANN Integration**: Seamless neural network library compatibility

## 📦 Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
micro_core = "0.1.0"

# Optional features
micro_core = { version = "0.1.0", features = ["std", "simd"] }
```

## 🏗️ Architecture

### Core Types

#### RootVector
32-dimensional SIMD-aligned vector for semantic embeddings:

```rust
use micro_core::{RootVector, RootSpace};

// Create a new root vector
let mut vector = RootVector::new();
vector[0] = 1.0;
vector[1] = 0.5;

// Compute dot product (SIMD optimized)
let other = RootVector::from_slice(&[0.8, 0.6, 0.0, /* ... 29 more values */]);
let similarity = vector.dot(&other);

// Normalize to unit length
vector.normalize();
```

#### MicroNet Trait
Standard interface for neural network agents:

```rust
use micro_core::{MicroNet, RootVector, AgentType};

struct ReasoningAgent {
    // Agent implementation
}

impl MicroNet for ReasoningAgent {
    fn forward(&mut self, input: &RootVector) -> RootVector {
        // Neural network forward pass
        self.process_reasoning(input)
    }
    
    fn agent_type(&self) -> AgentType {
        AgentType::Reasoning
    }
    
    fn compatibility_score(&self, input: &RootVector) -> f32 {
        // Return compatibility score [0.0, 1.0]
        0.8
    }
}
```

### Projection Operations

Convert high-dimensional embeddings to 32D root space:

```rust
use micro_core::{project_to_root, embed_from_root};

// Project 768-dimensional BERT embedding to root space
let bert_embedding = vec![0.1, 0.2, /* ... 766 more values */];
let root_vector = project_to_root(&bert_embedding, &projection_matrix);

// Reconstruct approximate high-dimensional embedding
let reconstructed = embed_from_root(&root_vector, &embedding_matrix);
```

## 🎯 Agent Types

The system supports five specialized agent types:

1. **Reasoning**: Complex logical inference and problem solving
2. **Routing**: Input classification and agent selection (rank-1 attention)
3. **Feature**: Feature extraction and transformation
4. **Embedding**: Dimensional projection and semantic mapping
5. **Expert**: Domain-specific knowledge and specialized processing

## 🧮 Mathematical Foundation

### Cartan Matrix Theory

The core implements orthogonal constraints inspired by Cartan matrices from Lie algebra:

- **Root Space**: 32-dimensional orthogonal basis {α₁, α₂, ..., α₃₂}
- **Orthogonality**: ⟨αᵢ, αⱼ⟩ = 2δᵢⱼ (Cartan normalization)
- **Semantic Structure**: Each dimension represents distinct semantic concepts

### SIMD Optimizations

Platform-specific vectorized operations:

- **x86_64**: AVX2 256-bit registers (8 floats per operation)
- **wasm32**: SIMD128 registers (4 floats per operation)
- **ARM**: NEON 128-bit registers with fallback to scalar

## 🔧 Configuration

### Feature Flags

```toml
[features]
default = []
std = ["dep:std"]           # Enable standard library features
simd = []                   # Platform-specific SIMD optimizations
serde = ["dep:serde"]       # Serialization support
ruvfann = ["dep:ruv-fann"] # rUv-FANN integration
```

### no_std Usage

The crate works in `no_std` environments:

```rust
#![no_std]
extern crate alloc;

use micro_core::{RootVector, MicroNet};
use alloc::vec::Vec;

// All core functionality available in no_std
```

## 📊 Performance

### Benchmarks

| Operation | Native (ns) | SIMD (ns) | Speedup |
|-----------|-------------|-----------|---------|
| Dot Product (32D) | 120 | 30 | 4.0x |
| Normalization | 95 | 28 | 3.4x |
| Projection (768→32) | 1,200 | 300 | 4.0x |

### Memory Layout

- **RootVector**: 128 bytes (32 × f32), 16-byte aligned
- **Agent State**: ~18KB average per micro-net
- **WASM Binary**: 145KB optimized build

## 🔗 Integration

### rUv-FANN Bridge

Seamless integration with existing rUv-FANN infrastructure:

```rust
use micro_core::{RuvFannBridge, CartanConfig};

let config = CartanConfig {
    regularization_strength: 0.01,
    annealing_schedule: Schedule::Cosine,
    root_count: 32,
};

let bridge = RuvFannBridge::new(config);
let metrics = bridge.process_batch(&inputs);
```

### Dashboard Integration

Export metrics for visualization:

```rust
use micro_core::MetricsExporter;

let exporter = MetricsExporter::new();
let json_metrics = exporter.export_json(&swarm_state);

// JSON contains:
// - Root space activations
// - Attention matrices  
// - Performance metrics
// - Orthogonality measures
```

## 🧪 Testing

Run the test suite:

```bash
# Unit tests
cargo test

# With all features
cargo test --all-features

# Benchmarks
cargo bench

# WASM tests  
wasm-pack test --node
```

## 📚 Examples

See the [`examples/`](examples/) directory for:

- **Basic Usage**: Creating and using micro-nets
- **SIMD Operations**: Performance optimization examples
- **Integration**: rUv-FANN bridge usage
- **Serialization**: Saving and loading models

## 🤝 Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](../CONTRIBUTING.md) for guidelines.

## 📄 License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](../LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](../LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## 🔗 Related Crates

- [`micro_routing`](../micro_routing): Dynamic agent routing and context management
- [`micro_cartan_attn`](../micro_cartan_attn): Cartan matrix attention mechanisms
- [`micro_metrics`](../micro_metrics): Performance monitoring and metrics
- [`micro_swarm`](../micro_swarm): Swarm orchestration and coordination

---

**Part of the rUv-FANN Semantic Cartan Matrix system** 🧠✨