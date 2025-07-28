# Neuro-Synaptic Chip Simulator Architecture

## Overview

This document describes the architecture of the Neuro-Synaptic Chip Simulator, focusing on WASM memory management patterns and parallel execution strategies for simulating a 256-core ASIC with 28MB shared memory.

## Memory Architecture

### 28MB Shared Memory Layout

The simulator models a 28MB (29,360,128 bytes) shared memory pool accessible by all 256 processing units. This memory is partitioned for efficient neural network execution:

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

#### Memory Regions

1. **Model Weights (16MB)**: Static storage for neural network parameters
   - Read-only during inference
   - Shared across all cores for the same model
   - Aligned to 64-byte boundaries for SIMD operations

2. **Activations (8MB)**: Dynamic storage for intermediate computations
   - Partitioned per core: 32KB per core (8MB / 256)
   - Double-buffered for pipelined execution
   - Cache-line aligned for optimal memory access

3. **I/O Buffers (4MB)**: Input/output data storage
   - Input buffer: 2MB (supports batch processing)
   - Output buffer: 2MB (results collection)
   - Per-core allocation: 8KB input, 8KB output

### WASM Memory Management Patterns

#### 1. Wasmtime Shared Memory Configuration

```rust
use wasmtime::*;

pub struct SharedMemoryPool {
    engine: Engine,
    memory: Memory,
    memory_type: MemoryType,
}

impl SharedMemoryPool {
    pub fn new(store: &mut Store<()>) -> Result<Self> {
        let engine = Engine::default();
        
        // Configure shared memory with exact 28MB limit
        // 448 WASM pages × 65536 bytes/page = 29,360,128 bytes (28MB)
        let memory_type = MemoryType::new(
            Limits::new(448, Some(448)), // min and max pages
            true  // shared=true for multi-threaded access
        );
        
        let memory = Memory::new(store, memory_type)?;
        
        Ok(SharedMemoryPool {
            engine,
            memory,
            memory_type,
        })
    }
}
```

#### 2. Memory Access Patterns for 256 Cores

**Concurrent Read, Exclusive Write Pattern**:
- Multiple cores can read from shared weight regions simultaneously
- Each core has exclusive write access to its activation partition
- Synchronization barriers between neural network layers

```rust
// Memory layout calculation per core
const TOTAL_MEMORY: usize = 28 * 1024 * 1024;  // 28MB
const NUM_CORES: usize = 256;
const ACTIVATION_PER_CORE: usize = 32 * 1024;  // 32KB

struct CoreMemoryRegion {
    core_id: u32,
    activation_offset: usize,
    input_offset: usize,
    output_offset: usize,
}

impl CoreMemoryRegion {
    fn new(core_id: u32) -> Self {
        Self {
            core_id,
            // Each core gets 32KB in activation region
            activation_offset: 0x1000000 + (core_id as usize * ACTIVATION_PER_CORE),
            // Each core gets 8KB for input
            input_offset: 0x1800000 + (core_id as usize * 8192),
            // Each core gets 8KB for output  
            output_offset: 0x1A00000 + (core_id as usize * 8192),
        }
    }
}
```

#### 3. Zero-Copy Data Sharing

To minimize memory bandwidth usage, the simulator implements zero-copy patterns:

```rust
// Shared weight access - all cores read same memory
pub fn get_model_weights(memory: &Memory, store: &Store<()>) -> &[f32] {
    let data = memory.data(store);
    let weights_ptr = data.as_ptr() as *const f32;
    let weights_len = (16 * 1024 * 1024) / 4; // 16MB / sizeof(f32)
    unsafe {
        std::slice::from_raw_parts(weights_ptr, weights_len)
    }
}
```

## Parallel Execution Architecture

### Thread Pool Design for 256 Cores

The simulator uses a hybrid approach to efficiently simulate 256 cores on host machines with fewer physical cores:

```rust
use rayon::prelude::*;
use crossbeam::channel::{bounded, Sender, Receiver};

pub struct SimulatorThreadPool {
    // Logical cores (256)
    logical_cores: Vec<LogicalCore>,
    // Physical thread pool (sized to host CPU)
    thread_pool: rayon::ThreadPool,
    // Work distribution channels
    work_queue: (Sender<WorkItem>, Receiver<WorkItem>),
}

impl SimulatorThreadPool {
    pub fn new() -> Self {
        let num_threads = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(8);
            
        let thread_pool = rayon::ThreadPoolBuilder::new()
            .num_threads(num_threads)
            .thread_name(|i| format!("sim-worker-{}", i))
            .build()
            .unwrap();
            
        let (tx, rx) = bounded(256); // Queue for all logical cores
        
        Self {
            logical_cores: (0..256).map(|i| LogicalCore::new(i)).collect(),
            thread_pool,
            work_queue: (tx, rx),
        }
    }
}
```

### Synchronization Patterns

#### 1. Barrier Synchronization Between Layers

```rust
use std::sync::{Arc, Barrier};

pub struct LayerBarrier {
    barrier: Arc<Barrier>,
}

impl LayerBarrier {
    pub fn new(num_cores: usize) -> Self {
        Self {
            barrier: Arc::new(Barrier::new(num_cores)),
        }
    }
    
    pub fn wait(&self) {
        self.barrier.wait();
    }
}
```

#### 2. Lock-Free Memory Access

For high-frequency operations, the simulator uses atomic operations:

```rust
use std::sync::atomic::{AtomicU64, Ordering};

pub struct MemoryStats {
    reads: AtomicU64,
    writes: AtomicU64,
    cycles: AtomicU64,
}

impl MemoryStats {
    pub fn record_read(&self) {
        self.reads.fetch_add(1, Ordering::Relaxed);
    }
    
    pub fn record_write(&self) {
        self.writes.fetch_add(1, Ordering::Relaxed);
    }
}
```

## WASM Instance Management

### Instance Pooling for 256 Concurrent Executions

```rust
use wasmtime::*;

pub struct WasmInstancePool {
    module: Module,
    instances: Vec<Instance>,
    stores: Vec<Store<CoreState>>,
}

impl WasmInstancePool {
    pub fn new(engine: &Engine, wasm_bytes: &[u8]) -> Result<Self> {
        // Compile module once
        let module = Module::new(engine, wasm_bytes)?;
        
        let mut instances = Vec::with_capacity(256);
        let mut stores = Vec::with_capacity(256);
        
        // Create 256 instances sharing the same memory
        for core_id in 0..256 {
            let mut store = Store::new(engine, CoreState { core_id });
            
            // Import shared memory
            let imports = [
                Extern::Memory(shared_memory.clone())
            ];
            
            let instance = Instance::new(&mut store, &module, &imports)?;
            instances.push(instance);
            stores.push(store);
        }
        
        Ok(Self { module, instances, stores })
    }
}
```

### WASM Execution Configuration

#### Optimizations for Neural Network Workloads

```rust
pub fn create_optimized_config() -> Config {
    let mut config = Config::new();
    
    // Enable SIMD for vector operations
    config.wasm_simd(true);
    
    // Enable shared memory for multi-core access
    config.wasm_threads(true);
    
    // Enable bulk memory operations
    config.wasm_bulk_memory(true);
    
    // Optimize for throughput
    config.cranelift_opt_level(OptLevel::Speed);
    
    // Enable parallel compilation
    config.parallel_compilation(true);
    
    // Set memory limits
    config.memory_reservation(28 * 1024 * 1024);
    
    config
}
```

## Performance Optimization Strategies

### 1. Memory Bandwidth Optimization

- **Coalesced Access**: Cores access memory in aligned chunks
- **Prefetching**: Predictive loading of next layer's weights
- **Double Buffering**: Overlap computation with memory transfers

### 2. SIMD Utilization

```rust
// Example: Vectorized matrix multiplication using WASM SIMD
#[target_feature(enable = "simd128")]
pub fn simd_matmul(a: &[f32], b: &[f32], c: &mut [f32], n: usize) {
    use std::arch::wasm32::*;
    
    for i in (0..n).step_by(4) {
        let a_vec = f32x4_load(&a[i]);
        let b_vec = f32x4_load(&b[i]);
        let c_vec = f32x4_mul(a_vec, b_vec);
        f32x4_store(&mut c[i], c_vec);
    }
}
```

### 3. Cache-Aware Data Layout

- Weights stored in row-major order for sequential access
- Activation tensors aligned to cache line boundaries (64 bytes)
- Hot data (frequently accessed weights) kept in lower memory addresses

## Memory Safety and Bounds Checking

### Compile-Time Guarantees

```rust
use std::marker::PhantomData;

pub struct BoundedMemoryRegion<'a> {
    base: *mut u8,
    size: usize,
    _phantom: PhantomData<&'a mut [u8]>,
}

impl<'a> BoundedMemoryRegion<'a> {
    pub fn new(memory: &'a mut [u8], offset: usize, size: usize) -> Option<Self> {
        if offset + size <= memory.len() {
            Some(Self {
                base: unsafe { memory.as_mut_ptr().add(offset) },
                size,
                _phantom: PhantomData,
            })
        } else {
            None // Out of bounds
        }
    }
    
    pub fn write_f32(&mut self, offset: usize, value: f32) -> Result<(), MemoryError> {
        if offset + 4 <= self.size {
            unsafe {
                let ptr = self.base.add(offset) as *mut f32;
                ptr.write(value);
            }
            Ok(())
        } else {
            Err(MemoryError::OutOfBounds)
        }
    }
}
```

## Timing Model Integration

### Cycle-Accurate Memory Access Timing

```rust
pub struct MemoryAccessTimer {
    l1_latency: u32,  // 1 cycle
    l2_latency: u32,  // 4 cycles
    dram_latency: u32, // 100 cycles
}

impl MemoryAccessTimer {
    pub fn calculate_access_time(&self, address: usize, access_size: usize) -> u32 {
        // Simplified model: addresses < 1MB are "cached"
        if address < 1024 * 1024 {
            self.l1_latency * (access_size / 64) // 64-byte cache lines
        } else if address < 4 * 1024 * 1024 {
            self.l2_latency * (access_size / 64)
        } else {
            self.dram_latency * (access_size / 64)
        }
    }
}
```

## Integration with ruv-FANN

The simulator integrates with ruv-FANN compiled to WASM:

1. **Model Loading**: ruv-FANN models are serialized and loaded into the weights region
2. **Execution**: Each core runs ruv-FANN inference functions via WASM exports
3. **Memory Layout**: ruv-FANN's memory layout is respected within the 28MB constraint

## Next Steps

1. Implement detailed memory access profiling
2. Add support for dynamic memory allocation within constraints
3. Optimize for specific neural network architectures (CNN, RNN, Transformer)
4. Implement memory compression for larger models
5. Add telemetry for memory bandwidth utilization

## References

- [WebAssembly Threads Proposal](https://github.com/WebAssembly/threads)
- [Wasmtime Shared Memory Documentation](https://docs.wasmtime.dev/api/wasmtime/struct.SharedMemory.html)
- [WASM SIMD Operations](https://webassembly.github.io/spec/simd/index.html)