# ruv-FANN WASM Integration Guide for Neuro-Synaptic Simulator

## Overview

This document outlines the integration strategy for compiling ruv-FANN neural networks to WebAssembly (WASM) for execution on the neuro-synaptic chip simulator. The integration leverages ruv-FANN's existing WASM compilation support, SIMD optimizations, and efficient memory management to create high-performance neural network modules that fit within the chip's 28MB memory constraint.

## Key Integration Points

### 1. WASM Compilation Target

ruv-FANN already supports WASM compilation through feature flags:

```toml
# Enable WASM compilation with SIMD support
[features]
wasm = ["no_std", "serde", "logging", "dep:wasm-bindgen", "dep:js-sys", "dep:web-sys", "dep:console_error_panic_hook", "dep:wasm-bindgen-futures"]
webgpu = ["dep:wgpu", "dep:futures", "dep:bytemuck", "wasm", "logging"]
wasm-gpu = ["webgpu", "wasm"]
```

### 2. Neural Network Model Formats

ruv-FANN supports multiple serialization formats suitable for WASM deployment:

#### Binary Format (Recommended for WASM)
- **Format**: Bincode serialization
- **Advantages**: Compact size, fast loading, little-endian compatible
- **Usage**: Best for pre-trained models embedded in WASM modules

```rust
use ruv_fann::io::{write_binary, read_binary};

// Serialize network to binary
let mut buffer = Vec::new();
write_binary(&network, &mut buffer)?;

// Load in WASM module
let network = read_binary(&mut buffer.as_slice())?;
```

#### JSON Format
- **Format**: Human-readable JSON
- **Advantages**: Easy debugging, cross-platform
- **Usage**: Development and configuration

#### FANN Format
- **Format**: Classic FANN text format
- **Advantages**: Compatibility with existing FANN tools
- **Usage**: Legacy model import

### 3. SIMD Optimization Opportunities

The chip simulator can leverage WASM SIMD (128-bit vector operations) for neural computations:

#### Vector Operations
- Matrix multiplication with AVX2/AVX-512 emulation
- Vectorized activation functions (ReLU, Sigmoid, Tanh)
- Parallel gradient computation
- Batch normalization

#### SIMD Configuration
```rust
// Enable SIMD in WASM module
#[cfg(target_arch = "wasm32")]
use std::arch::wasm32::*;

// Vector dot product using WASM SIMD
pub fn simd_dot_product(a: &[f32], b: &[f32]) -> f32 {
    assert_eq!(a.len(), b.len());
    assert!(a.len() % 4 == 0); // Ensure alignment for v128
    
    let mut sum = f32x4_splat(0.0);
    let chunks = a.len() / 4;
    
    for i in 0..chunks {
        let offset = i * 4;
        let a_vec = v128_load(&a[offset] as *const f32 as *const v128);
        let b_vec = v128_load(&b[offset] as *const f32 as *const v128);
        
        // Fused multiply-add for better performance
        sum = f32x4_add(sum, f32x4_mul(a_vec, b_vec));
    }
    
    // Horizontal sum
    f32x4_extract_lane::<0>(sum) + 
    f32x4_extract_lane::<1>(sum) + 
    f32x4_extract_lane::<2>(sum) + 
    f32x4_extract_lane::<3>(sum)
}

// Vectorized ReLU activation
pub fn simd_relu(values: &mut [f32]) {
    assert!(values.len() % 4 == 0);
    
    let zero = f32x4_splat(0.0);
    let chunks = values.len() / 4;
    
    for i in 0..chunks {
        let offset = i * 4;
        let v = v128_load(&values[offset] as *const f32 as *const v128);
        let result = f32x4_max(v, zero);
        v128_store(&mut values[offset] as *mut f32 as *mut v128, result);
    }
}
```

### 4. Memory Layout for 28MB Constraint

Optimal memory allocation strategy for the chip's shared memory:

```
Total: 28MB (29,360,128 bytes)

┌─────────────────────────────┐
│ Model Weights (16MB)        │ - Static neural network parameters
├─────────────────────────────┤
│ Activations (8MB)           │ - Layer outputs and intermediates
├─────────────────────────────┤
│ I/O Buffers (2MB)           │ - Input/output data
├─────────────────────────────┤
│ Workspace (2MB)             │ - Temporary computation space
└─────────────────────────────┘
```

### 5. WASM Module Structure

Recommended structure for neural network WASM modules:

```rust
// Neural network WASM module API
#[wasm_bindgen]
pub struct NeuralModule {
    network: Network,
    input_buffer: Vec<f32>,
    output_buffer: Vec<f32>,
}

#[wasm_bindgen]
impl NeuralModule {
    #[wasm_bindgen(constructor)]
    pub fn new(model_data: &[u8]) -> Result<NeuralModule, JsValue> {
        // Load network from binary data
    }
    
    #[wasm_bindgen]
    pub fn inference(&mut self, input_ptr: *const f32, output_ptr: *mut f32) -> Result<(), JsValue> {
        // Run inference using shared memory pointers
    }
    
    #[wasm_bindgen]
    pub fn get_memory_usage(&self) -> u32 {
        // Report memory consumption
    }
}
```

### 6. Fuel-Based Instruction Counting

Integration with simulator's timing model using Wasmtime fuel:

```rust
// Configure fuel consumption for timing
let mut config = Config::new();
config.consume_fuel(true);

// Set initial fuel (instructions)
store.add_fuel(1_000_000_000)?;

// Execute and measure consumed fuel
let initial_fuel = store.fuel_consumed().unwrap_or(0);
instance.call_inference()?;
let consumed = store.fuel_consumed().unwrap_or(0) - initial_fuel;

// Map to simulated cycles
let cycles = consumed * CYCLES_PER_INSTRUCTION;
```

#### Fuel Consumption Characteristics

**Instruction-to-Fuel Mapping** (Approximate):
- Basic arithmetic ops: 1 fuel unit
- Memory loads/stores: 2-3 fuel units  
- SIMD operations: 4-8 fuel units
- Function calls: 10-20 fuel units
- Indirect calls: 20-30 fuel units

**Timing Model Calibration**:
```rust
// Calibration constants for 12nm ASIC @ 1GHz
const CYCLES_PER_FUEL_UNIT: u64 = 2; // Conservative estimate
const NANOSECONDS_PER_CYCLE: f64 = 1.0; // 1GHz clock

// Convert fuel to time
let execution_time_ns = consumed * CYCLES_PER_FUEL_UNIT as f64 * NANOSECONDS_PER_CYCLE;
let execution_time_us = execution_time_ns / 1000.0;
```

**Performance Considerations**:
- Fuel tracking adds ~2x overhead to execution
- Use epoch-based interruption for production if precise timing not required
- Consider "slacked fuel metering" for reduced overhead with approximate timing

### 7. Compilation Pipeline

Step-by-step process to compile ruv-FANN models to WASM:

#### 1. Train Model in Rust
```rust
use ruv_fann::{Network, ActivationFunction};

let mut network = Network::new(&[784, 128, 10])?;
network.train(&training_data, epochs)?;
```

#### 2. Serialize to Binary
```rust
let mut model_data = Vec::new();
write_binary(&network, &mut model_data)?;
```

#### 3. Build WASM Module
```bash
# Build with WASM target and SIMD
cargo build --target wasm32-unknown-unknown \
    --features "wasm,simd" \
    --release
    
# Optimize WASM binary
wasm-opt -O3 --enable-simd \
    target/wasm32-unknown-unknown/release/neural_module.wasm \
    -o neural_module_opt.wasm
```

#### 4. Embed Model Data
```rust
// Include model data in WASM module
const MODEL_DATA: &[u8] = include_bytes!("model.bin");
```

### 8. Performance Optimization Strategies

#### Memory Access Patterns
- Sequential access for weight matrices
- Tiled operations for cache efficiency
- Minimize cross-core memory conflicts

#### Parallelization
- Layer-wise parallelism across cores
- Batch processing for multiple inputs
- Pipeline parallelism for deep networks

#### SIMD Utilization
- Vectorize inner loops
- Use fused multiply-add operations
- Align data for optimal SIMD access

### 9. Example Integration

Complete example of preparing a ruv-FANN model for the simulator:

```rust
// train_and_export.rs
use ruv_fann::{Network, ActivationFunction, TrainingData};
use ruv_fann::io::write_binary;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create and train network
    let mut network = Network::new(&[784, 256, 128, 10])?;
    network.set_activation_function(ActivationFunction::ReLU)?;
    
    let training_data = TrainingData::from_file("mnist.data")?;
    network.train(&training_data, 100)?;
    
    // Verify memory usage fits in chip
    let weight_count = network.get_total_connections();
    let memory_usage = weight_count * 4; // f32 weights
    assert!(memory_usage < 16 * 1024 * 1024, "Model too large!");
    
    // Export to binary
    let mut file = std::fs::File::create("mnist_model.bin")?;
    write_binary(&network, &mut file)?;
    
    println!("Model exported: {} bytes", file.metadata()?.len());
    Ok(())
}
```

```rust
// wasm_module.rs
use wasm_bindgen::prelude::*;
use ruv_fann::{Network};
use ruv_fann::io::read_binary;

const MODEL_DATA: &[u8] = include_bytes!("../mnist_model.bin");

#[wasm_bindgen]
pub struct MnistClassifier {
    network: Network,
}

#[wasm_bindgen]
impl MnistClassifier {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<MnistClassifier, JsValue> {
        let network = read_binary(&mut MODEL_DATA)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        Ok(MnistClassifier { network })
    }
    
    #[wasm_bindgen]
    pub fn classify(&mut self, input: &[f32]) -> Vec<f32> {
        self.network.run(input).unwrap_or_default()
    }
}
```

### 10. Testing Integration

Verification steps for WASM modules:

1. **Memory Constraints**: Verify total memory < 28MB
2. **Instruction Count**: Profile fuel consumption
3. **Correctness**: Compare outputs with native execution
4. **Performance**: Measure inference time
5. **SIMD Usage**: Verify vector instructions are used

### 11. Future Enhancements

#### ONNX Support
- Add ONNX import/export for model portability
- Convert ONNX models to ruv-FANN format

#### Quantization
- INT8 quantization for reduced memory usage
- Mixed precision computation support

#### Model Compression
- Weight pruning and sparsity
- Knowledge distillation for smaller models

#### Hardware-Specific Optimizations
- Custom WASM instructions for neural ops
- Direct memory mapping for zero-copy inference

## Conclusion

The integration of ruv-FANN with the neuro-synaptic simulator through WASM provides a robust foundation for executing neural networks within the chip's constraints. The combination of efficient serialization formats, SIMD optimizations, and careful memory management enables high-performance inference while maintaining the safety and portability benefits of WebAssembly.