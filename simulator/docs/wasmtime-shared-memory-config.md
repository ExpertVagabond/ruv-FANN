# Wasmtime Shared Memory Configuration Guide

## Overview

This document provides detailed configuration patterns for implementing shared memory in Wasmtime to support 256 concurrent WASM instances accessing a single 28MB memory pool.

## Wasmtime Shared Memory Requirements

### 1. Enabling WebAssembly Threads Feature

Shared memory in Wasmtime requires the WebAssembly threads proposal:

```rust
use wasmtime::*;

pub fn create_engine_with_shared_memory() -> Result<Engine> {
    let mut config = Config::new();
    
    // Enable threads proposal for shared memory support
    config.wasm_threads(true);
    
    // Enable SIMD for neural network operations
    config.wasm_simd(true);
    
    // Enable bulk memory operations for efficient copies
    config.wasm_bulk_memory(true);
    
    // Configure memory limits
    config.memory_pages_limit(448); // 28MB = 448 pages
    config.memory_growth_limit(0);   // No growth allowed
    
    // Enable parallel compilation for faster module loading
    config.parallel_compilation(true);
    
    Engine::new(&config)
}
```

### 2. Creating Shared Memory Instance

```rust
use wasmtime::*;

pub struct SharedMemoryManager {
    memory: Memory,
    store: Store<()>,
}

impl SharedMemoryManager {
    pub fn new(engine: &Engine) -> Result<Self> {
        let mut store = Store::new(engine, ());
        
        // Create shared memory type
        // Important: shared=true for concurrent access
        let memory_type = MemoryType::new(
            448,     // minimum pages (28MB)
            Some(448), // maximum pages (28MB) - prevents growth
            true      // shared memory flag
        );
        
        // Create the shared memory instance
        let memory = Memory::new(&mut store, memory_type)?;
        
        // Verify size
        assert_eq!(memory.data_size(&store), 29_360_128); // 28MB
        
        Ok(SharedMemoryManager { memory, store })
    }
    
    pub fn get_memory(&self) -> Memory {
        self.memory
    }
}
```

## Concurrent Instance Creation Pattern

### Creating 256 Instances with Shared Memory

```rust
use wasmtime::*;
use std::sync::{Arc, Mutex};

pub struct InstancePool {
    engine: Engine,
    module: Module,
    shared_memory: Memory,
    instances: Vec<Arc<Mutex<Instance>>>,
    stores: Vec<Arc<Mutex<Store<CoreContext>>>>,
}

#[derive(Default)]
pub struct CoreContext {
    core_id: u32,
    cycles: u64,
    memory_accesses: u64,
}

impl InstancePool {
    pub fn new(wasm_bytes: &[u8]) -> Result<Self> {
        let engine = create_engine_with_shared_memory()?;
        let module = Module::new(&engine, wasm_bytes)?;
        
        // Create shared memory
        let mut temp_store = Store::new(&engine, ());
        let memory_type = MemoryType::new(448, Some(448), true);
        let shared_memory = Memory::new(&mut temp_store, memory_type)?;
        
        let mut instances = Vec::with_capacity(256);
        let mut stores = Vec::with_capacity(256);
        
        // Create 256 instances
        for core_id in 0..256 {
            let context = CoreContext {
                core_id,
                cycles: 0,
                memory_accesses: 0,
            };
            
            let mut store = Store::new(&engine, context);
            
            // Configure store for deterministic execution
            store.set_fuel(u64::MAX)?;
            store.fuel_async_yield_interval(Some(10000))?;
            
            // Create imports with shared memory
            let mut imports = Imports::new();
            imports.define("env", "memory", shared_memory);
            
            // Instantiate with shared memory
            let instance = Instance::new(&mut store, &module, &imports)?;
            
            instances.push(Arc::new(Mutex::new(instance)));
            stores.push(Arc::new(Mutex::new(store)));
        }
        
        Ok(InstancePool {
            engine,
            module,
            shared_memory,
            instances,
            stores,
        })
    }
}
```

## Memory Access Synchronization

### Atomic Operations for Shared Memory

```rust
use std::sync::atomic::{AtomicU32, Ordering};

// WASM-compatible atomic operations
#[repr(C)]
pub struct AtomicCounter {
    value: AtomicU32,
}

impl AtomicCounter {
    pub fn new() -> Self {
        Self {
            value: AtomicU32::new(0),
        }
    }
    
    pub fn increment(&self) -> u32 {
        self.value.fetch_add(1, Ordering::SeqCst)
    }
    
    pub fn load(&self) -> u32 {
        self.value.load(Ordering::SeqCst)
    }
}

// Memory barrier for synchronization
pub fn memory_barrier() {
    std::sync::atomic::fence(Ordering::SeqCst);
}
```

### Thread-Safe Memory Region Access

```rust
use parking_lot::{RwLock, Mutex};
use std::sync::Arc;

pub struct MemoryRegionManager {
    // Read-write lock for weight region (many readers, rare writers)
    weights_lock: Arc<RwLock<()>>,
    
    // Per-core locks for activation regions
    activation_locks: Vec<Arc<Mutex<()>>>,
    
    // Shared memory reference
    memory: Memory,
}

impl MemoryRegionManager {
    pub fn new(memory: Memory) -> Self {
        let mut activation_locks = Vec::with_capacity(256);
        for _ in 0..256 {
            activation_locks.push(Arc::new(Mutex::new(())));
        }
        
        Self {
            weights_lock: Arc::new(RwLock::new(())),
            activation_locks,
            memory,
        }
    }
    
    pub fn read_weights<F, R>(&self, store: &Store<CoreContext>, f: F) -> R
    where
        F: FnOnce(&[u8]) -> R,
    {
        let _guard = self.weights_lock.read();
        let data = self.memory.data(store);
        f(&data[0..0x1000000]) // First 16MB
    }
    
    pub fn write_activation<F>(&self, store: &mut Store<CoreContext>, core_id: u32, f: F)
    where
        F: FnOnce(&mut [u8]),
    {
        let _guard = self.activation_locks[core_id as usize].lock();
        let data_mut = self.memory.data_mut(store);
        let offset = 0x1000000 + (core_id as usize * 0x8000);
        f(&mut data_mut[offset..offset + 0x8000]);
    }
}
```

## Wasmtime-Specific Optimizations

### 1. Memory Mapping and Page Protection

```rust
// Configure memory protection for safety
pub fn configure_memory_protection(config: &mut Config) {
    // Enable memory protection keys if available
    config.memory_guaranteed_dense_image_size(29_360_128); // 28MB
    
    // Use copy-on-write for initial memory
    config.memory_init_cow(true);
    
    // Configure guard pages
    config.memory_guard_size(65536); // 64KB guard pages
}
```

### 2. Fuel-Based Resource Limiting

```rust
pub fn configure_fuel_limits(store: &mut Store<CoreContext>) -> Result<()> {
    // Enable fuel consumption
    store.add_fuel(1_000_000)?; // 1M instructions per quantum
    
    // Set yield interval for cooperative multitasking
    store.fuel_async_yield_interval(Some(10_000))?;
    
    Ok(())
}

// Track fuel consumption per core
pub fn get_instruction_count(store: &Store<CoreContext>) -> Result<u64> {
    let consumed = store.fuel_consumed().unwrap_or(0);
    Ok(consumed)
}
```

### 3. Pooling Allocator Configuration

```rust
use wasmtime::PoolingAllocationConfig;

pub fn create_pooling_engine() -> Result<Engine> {
    let mut config = Config::new();
    
    // Configure pooling allocator for 256 instances
    let mut pool_config = PoolingAllocationConfig::default();
    pool_config.instance_count(256);
    pool_config.instance_memory_pages(448); // 28MB per instance
    pool_config.instance_table_elements(1024);
    
    config.allocation_strategy(InstanceAllocationStrategy::Pooling(pool_config));
    
    Engine::new(&config)
}
```

## Performance Considerations

### 1. NUMA-Aware Memory Allocation

```rust
#[cfg(target_os = "linux")]
pub fn configure_numa_aware_memory() {
    use libc::{numa_available, numa_set_localalloc};
    
    unsafe {
        if numa_available() != -1 {
            // Prefer local NUMA node allocation
            numa_set_localalloc();
        }
    }
}
```

### 2. Cache-Aligned Memory Access

```rust
#[repr(align(64))] // Cache line alignment
pub struct CacheAlignedBuffer {
    data: [u8; 64],
}

pub fn ensure_cache_alignment(address: usize) -> usize {
    (address + 63) & !63 // Round up to 64-byte boundary
}
```

### 3. Batch Memory Operations

```rust
use wasmtime::Memory;

pub fn batch_memory_copy(
    memory: &Memory,
    store: &mut Store<CoreContext>,
    operations: Vec<(usize, usize, usize)>, // (src, dst, len)
) -> Result<()> {
    let data = memory.data_mut(store);
    
    for (src, dst, len) in operations {
        // Use optimized memcpy for large transfers
        if len > 4096 {
            unsafe {
                std::ptr::copy_nonoverlapping(
                    data.as_ptr().add(src),
                    data.as_mut_ptr().add(dst),
                    len
                );
            }
        } else {
            data.copy_within(src..src+len, dst);
        }
    }
    
    Ok(())
}
```

## Error Handling and Recovery

### Memory Exhaustion Handling

```rust
pub enum MemoryError {
    OutOfMemory,
    AccessViolation,
    AllocationFailed,
}

pub fn safe_memory_allocate(
    memory: &Memory,
    store: &Store<CoreContext>,
    size: usize,
) -> Result<usize, MemoryError> {
    let current_size = memory.data_size(store);
    let max_size = 29_360_128; // 28MB
    
    if current_size + size > max_size {
        return Err(MemoryError::OutOfMemory);
    }
    
    // Attempt to grow memory (will fail due to max limit)
    match memory.grow(store, (size + 65535) / 65536) {
        Ok(prev_pages) => Ok(prev_pages * 65536),
        Err(_) => Err(MemoryError::AllocationFailed),
    }
}
```

## Testing Shared Memory Configuration

### Unit Test Example

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_shared_memory_creation() {
        let engine = create_engine_with_shared_memory().unwrap();
        let mut store = Store::new(&engine, ());
        
        let memory_type = MemoryType::new(448, Some(448), true);
        let memory = Memory::new(&mut store, memory_type).unwrap();
        
        assert_eq!(memory.data_size(&store), 29_360_128);
        assert_eq!(memory.ty(&store).is_shared(), true);
    }
    
    #[test]
    fn test_concurrent_access() {
        use std::thread;
        
        let pool = Arc::new(InstancePool::new(&[]).unwrap());
        let mut handles = vec![];
        
        for i in 0..256 {
            let pool_clone = Arc::clone(&pool);
            let handle = thread::spawn(move || {
                // Simulate concurrent memory access
                let store = pool_clone.stores[i].lock();
                let data = pool_clone.shared_memory.data(&store);
                assert_eq!(data.len(), 29_360_128);
            });
            handles.push(handle);
        }
        
        for handle in handles {
            handle.join().unwrap();
        }
    }
}
```

## Best Practices

1. **Always use shared memory type** when creating memory for concurrent access
2. **Set both min and max pages** to the same value to prevent growth
3. **Use atomic operations** for synchronization primitives
4. **Implement proper locking** for exclusive memory regions
5. **Monitor fuel consumption** to track performance
6. **Use pooling allocators** for better instance management
7. **Test with ThreadSanitizer** to detect race conditions

## References

- [Wasmtime Shared Memory API](https://docs.wasmtime.dev/api/wasmtime/struct.Memory.html)
- [WebAssembly Threads Proposal](https://github.com/WebAssembly/threads/blob/master/proposals/threads/Overview.md)
- [Wasmtime Configuration Options](https://docs.wasmtime.dev/api/wasmtime/struct.Config.html)