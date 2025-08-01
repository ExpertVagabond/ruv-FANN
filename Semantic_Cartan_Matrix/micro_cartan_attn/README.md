# micro_cartan_attn

[![Crates.io](https://img.shields.io/crates/v/micro_cartan_attn.svg)](https://crates.io/crates/micro_cartan_attn)
[![Documentation](https://docs.rs/micro_cartan_attn/badge.svg)](https://docs.rs/micro_cartan_attn)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![Build Status](https://github.com/ruvnet/ruv-FANN/workflows/CI/badge.svg)](https://github.com/ruvnet/ruv-FANN/actions)

**Cartan matrix attention mechanisms and orthogonal regularization**

The `micro_cartan_attn` crate implements Cartan matrix-inspired attention mechanisms for maintaining orthogonal semantic representations in micro-neural networks. It provides structured attention patterns based on Lie algebra principles.

## 🚀 Features

- **Cartan Matrix Theory**: Lie algebra-inspired orthogonal constraints
- **Multi-Head Attention**: Standard and Cartan-regularized attention layers
- **Rank-1 Routing Heads**: Efficient attention routing with O(1) complexity
- **Orthogonalization Methods**: Gram-Schmidt, QR decomposition, symmetric orthogonalization
- **Regularization Scheduling**: Adaptive constraint strength during training
- **SIMD Optimizations**: Vectorized attention computations
- **no_std Compatible**: Embedded and WebAssembly deployment

## 📦 Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
micro_cartan_attn = "0.1.0"
micro_core = "0.1.0"

# Optional features
micro_cartan_attn = { version = "0.1.0", features = ["std", "training"] }
```

## 🏗️ Mathematical Foundation

### Cartan Matrix Theory

The crate implements attention mechanisms based on Cartan matrices from Lie algebra:

- **Root System**: {α₁, α₂, ..., α₃₂} orthogonal basis vectors
- **Cartan Matrix**: C_{ij} = 2⟨αᵢ, αⱼ⟩/⟨αⱼ, αⱼ⟩
- **Orthogonality**: ⟨αᵢ, αⱼ⟩ = 2δᵢⱼ (normalized constraint)

### Supported Cartan Types

```rust
use micro_cartan_attn::{CartanType, CartanMatrix};

// Identity matrix (fully orthogonal)
let identity = CartanMatrix::new(CartanType::Identity, 32);

// A_n type (cyclic structure)
let a_type = CartanMatrix::new(CartanType::A(31), 32);

// D_n type (branched structure)  
let d_type = CartanMatrix::new(CartanType::D(16), 32);

// E8 type (exceptional structure)
let e8_type = CartanMatrix::new(CartanType::E8, 32);
```

## 🧠 Attention Mechanisms

### Cartan-Regularized Attention

Multi-head attention with orthogonal constraints:

```rust
use micro_cartan_attn::{CartanAttention, AttentionConfig};
use micro_core::RootVector;

let config = AttentionConfig {
    num_heads: 8,
    head_dim: 32,
    cartan_type: CartanType::Identity,
    regularization_strength: 0.01,
    dropout: 0.1,
};

let mut attention = CartanAttention::new(config);

// Forward pass with regularization
let query = RootVector::from_slice(&[/* ... */]);
let key = RootVector::from_slice(&[/* ... */]);
let value = RootVector::from_slice(&[/* ... */]);

let (output, attention_weights) = attention.forward(&query, &key, &value);

// Attention weights automatically satisfy Cartan constraints
```

### Rank-1 Routing Heads

Efficient attention heads that collapse to rank-1 for routing:

```rust
use micro_cartan_attn::{Rank1Head, RoutingConfig};

let config = RoutingConfig {
    input_dim: 32,
    temperature: 2.0,
    sparsity_threshold: 0.05,
};

let mut routing_head = Rank1Head::new(config);

// Compute routing scores (much faster than full attention)
let input = RootVector::from_slice(&[/* ... */]);
let routing_scores = routing_head.compute_routing_scores(&input);

// Scores sum to 1.0 and indicate agent preferences
for (i, score) in routing_scores.iter().enumerate() {
    if *score > 0.1 {
        println!("Route to agent {}: {:.3}", i, score);
    }
}
```

### Standard Multi-Head Attention

Compatible with transformer architectures:

```rust
use micro_cartan_attn::{MultiHeadAttention, StandardConfig};

let config = StandardConfig {
    num_heads: 8,
    head_dim: 64,
    seq_len: 128,
    causal_mask: false,
};

let mut attention = MultiHeadAttention::new(config);

// Standard attention computation
let (output, weights) = attention.forward(&query, &key, &value);
```

## 🔧 Orthogonalization Methods

### Gram-Schmidt Process

Classical orthogonalization with optional pivoting:

```rust
use micro_cartan_attn::{GramSchmidt, OrthogonalizationConfig};

let config = OrthogonalizationConfig {
    tolerance: 1e-6,
    max_iterations: 100,
    pivoting: true,
};

let mut orthogonalizer = GramSchmidt::new(config);

// Orthogonalize a set of vectors
let mut vectors = vec![
    RootVector::from_slice(&[1.0, 1.0, 0.0, /* ... */]),
    RootVector::from_slice(&[1.0, 0.0, 1.0, /* ... */]),
    RootVector::from_slice(&[0.0, 1.0, 1.0, /* ... */]),
];

orthogonalizer.orthogonalize(&mut vectors);

// Verify orthogonality
for i in 0..vectors.len() {
    for j in i+1..vectors.len() {
        let dot_product = vectors[i].dot(&vectors[j]);
        assert!(dot_product.abs() < 1e-6);
    }
}
```

### QR Decomposition

More numerically stable orthogonalization:

```rust
use micro_cartan_attn::QROrthogonalizer;

let mut orthogonalizer = QROrthogonalizer::new();

// Decompose matrix into orthogonal Q and upper triangular R
let (q_matrix, r_matrix) = orthogonalizer.decompose(&input_matrix);

// Extract orthogonal vectors from Q matrix
let orthogonal_vectors = q_matrix.columns().collect::<Vec<_>>();
```

### Symmetric Orthogonalization

Preserves vector norms while enforcing orthogonality:

```rust
use micro_cartan_attn::SymmetricOrthogonalizer;

let mut orthogonalizer = SymmetricOrthogonalizer::new();

// Symmetric orthogonalization: V' = VS^(-1/2)
// where S = V^T V is the overlap matrix
let orthogonal_vectors = orthogonalizer.orthogonalize(&input_vectors);

// Vectors maintain their original norms
```

## 📈 Regularization and Training

### Cartan Loss Function

Regularization term for training with orthogonal constraints:

```rust
use micro_cartan_attn::{CartanLoss, RegularizationConfig};

let config = RegularizationConfig {
    target_cartan_type: CartanType::Identity,
    strength: 0.01,
    schedule: Schedule::Cosine {
        max_strength: 0.1,
        min_strength: 0.001,
        period: 1000,
    },
};

let cartan_loss = CartanLoss::new(config);

// Compute regularization loss
let attention_weights = /* ... attention layer outputs ... */;
let regularization_loss = cartan_loss.compute_loss(&attention_weights);

// Add to total training loss
let total_loss = task_loss + regularization_loss;
```

### Annealing Schedules

Adaptive regularization strength during training:

```rust
use micro_cartan_attn::{Schedule, AnnealingScheduler};

// Cosine annealing
let cosine_schedule = Schedule::Cosine {
    max_strength: 0.1,
    min_strength: 0.001,
    period: 5000,
};

// Linear annealing
let linear_schedule = Schedule::Linear {
    start_strength: 0.0,
    end_strength: 0.05,
    steps: 10000,
};

// Exponential decay
let exp_schedule = Schedule::Exponential {
    initial_strength: 0.1,
    decay_rate: 0.95,
    decay_steps: 1000,
};

let mut scheduler = AnnealingScheduler::new(cosine_schedule);

// Update regularization strength during training
for step in 0..training_steps {
    let current_strength = scheduler.get_strength(step);
    cartan_loss.set_strength(current_strength);
}
```

## 🎯 Performance Optimizations

### SIMD Attention

Vectorized attention computations:

```rust
use micro_cartan_attn::{SIMDAttention, SIMDConfig};

let config = SIMDConfig {
    use_simd: true,
    batch_size: 32,
    vectorization: VectorizationType::Auto, // Auto-detect best SIMD
};

let mut simd_attention = SIMDAttention::new(config);

// 3-4x speedup with SIMD
let output = simd_attention.forward_batch(&queries, &keys, &values);
```

### Memory-Efficient Attention

Reduced memory attention for long sequences:

```rust
use micro_cartan_attn::{MemoryEfficientAttention, MemoryConfig};

let config = MemoryConfig {
    chunk_size: 256,
    gradient_checkpointing: true,
    flash_attention: true,
};

let mut efficient_attention = MemoryEfficientAttention::new(config);

// Process long sequences with constant memory
let output = efficient_attention.forward(&query, &key, &value);
```

## 📊 Metrics and Analysis

### Attention Analysis

Tools for analyzing attention patterns:

```rust
use micro_cartan_attn::{AttentionAnalyzer, AnalysisConfig};

let config = AnalysisConfig {
    compute_entropy: true,
    compute_sparsity: true,
    compute_rank: true,
    save_heatmaps: true,
};

let analyzer = AttentionAnalyzer::new(config);

// Analyze attention patterns
let analysis = analyzer.analyze(&attention_weights);

println!("Attention entropy: {:.3}", analysis.entropy);
println!("Sparsity: {:.3}", analysis.sparsity);
println!("Effective rank: {:.1}", analysis.effective_rank);

// Export heatmap for visualization
let heatmap_data = analysis.export_heatmap();
```

### Orthogonality Metrics

Measure adherence to orthogonal constraints:

```rust
use micro_cartan_attn::{OrthogonalityMetrics, MetricsConfig};

let config = MetricsConfig {
    tolerance: 1e-6,
    compute_condition_number: true,
};

let metrics = OrthogonalityMetrics::new(config);

// Measure orthogonality of vector set
let orthogonality_score = metrics.compute_orthogonality(&vectors);
let condition_number = metrics.compute_condition_number(&vectors);

println!("Orthogonality score: {:.6}", orthogonality_score);
println!("Condition number: {:.3}", condition_number);
```

## 🧪 Testing and Validation

### Property-Based Tests

Ensure mathematical properties hold:

```rust
use micro_cartan_attn::testing::*;
use quickcheck::TestResult;

#[quickcheck]
fn test_orthogonality_preserved(vectors: Vec<RootVector>) -> TestResult {
    if vectors.len() < 2 { return TestResult::discard(); }
    
    let mut orthogonalizer = GramSchmidt::new(Default::default());
    let mut orthogonal = vectors.clone();
    orthogonalizer.orthogonalize(&mut orthogonal);
    
    // Check orthogonality
    for i in 0..orthogonal.len() {
        for j in i+1..orthogonal.len() {
            let dot = orthogonal[i].dot(&orthogonal[j]);
            if dot.abs() > 1e-6 {
                return TestResult::failed();
            }
        }
    }
    
    TestResult::passed()
}
```

### Integration Tests

```bash
# Test with micro_core integration
cargo test --features integration-tests

# Test all orthogonalization methods
cargo test test_orthogonalization

# Benchmark attention performance
cargo bench attention_benchmarks
```

## 🔧 Configuration

### Feature Flags

```toml
[features]
default = ["gram-schmidt"]
std = ["dep:std"]
training = ["dep:candle-core", "dep:tch"]
simd = ["dep:wide"]
analysis = ["dep:plotters"]
blas = ["dep:blas"]
```

### Runtime Configuration

```rust
use micro_cartan_attn::{CartanConfig, GlobalConfig};

let config = CartanConfig {
    default_cartan_type: CartanType::Identity,
    regularization_strength: 0.01,
    orthogonalization_method: OrthogonalizationMethod::GramSchmidt,
    simd_threshold: 1000, // Use SIMD for vectors > 1000 elements
    numerical_tolerance: 1e-8,
};

// Set global configuration
GlobalConfig::set(config);
```

## 📚 Examples

See the [`examples/`](examples/) directory for:

- **Basic Attention**: Simple attention mechanism usage
- **Cartan Regularization**: Training with orthogonal constraints
- **Rank-1 Routing**: Efficient routing head implementation
- **SIMD Optimization**: Performance optimization examples
- **Analysis Tools**: Attention pattern analysis and visualization

## 🤝 Integration

### With micro_core

```rust
use micro_core::RootVector;
use micro_cartan_attn::CartanAttention;

// Seamless integration with root vectors
let attention = CartanAttention::new(config);
let output = attention.forward(&query, &key, &value);
```

### With micro_routing

```rust
use micro_routing::Router;
use micro_cartan_attn::Rank1Head;

// Use rank-1 heads for efficient routing
let routing_head = Rank1Head::new(config);
// Router uses attention-based agent selection
```

## 📄 License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](../LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](../LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## 🔗 Related Crates

- [`micro_core`](../micro_core): Core types and vector operations
- [`micro_routing`](../micro_routing): Dynamic routing using attention mechanisms
- [`micro_metrics`](../micro_metrics): Performance monitoring and analysis
- [`micro_swarm`](../micro_swarm): High-level orchestration with attention-based coordination

---

**Part of the rUv-FANN Semantic Cartan Matrix system** 🧠🔮