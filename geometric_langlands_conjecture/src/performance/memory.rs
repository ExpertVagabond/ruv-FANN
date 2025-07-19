//! Memory optimization and management
//!
//! This module provides memory pooling, efficient allocation strategies,
//! and memory usage tracking for high-performance computing.

use std::sync::{Arc, Mutex};
use std::collections::{HashMap, VecDeque};
use std::alloc::{alloc, dealloc, Layout};
use std::ptr::NonNull;
use std::cell::RefCell;

/// Memory pool for efficient allocation
pub struct MemoryPool {
    /// Pools organized by size class
    pools: HashMap<usize, Pool>,
    /// Total allocated memory
    total_allocated: usize,
    /// Peak memory usage
    peak_usage: usize,
    /// Allocation statistics
    stats: AllocationStats,
}

/// Individual pool for a specific size class
struct Pool {
    /// Size of allocations in this pool
    size: usize,
    /// Free blocks (storing as usize for thread safety)
    free_blocks: VecDeque<usize>,
    /// Total blocks allocated
    total_blocks: usize,
    /// Blocks currently in use
    used_blocks: usize,
}

/// Allocation statistics
#[derive(Debug, Clone, Default)]
struct AllocationStats {
    /// Total allocations
    total_allocations: u64,
    /// Total deallocations
    total_deallocations: u64,
    /// Allocation hits (from pool)
    pool_hits: u64,
    /// Allocation misses (new allocation)
    pool_misses: u64,
}

/// Memory-mapped region for large allocations
struct MappedRegion {
    /// Base pointer (as usize for thread safety)
    ptr: usize,
    /// Size of the region
    size: usize,
    /// Layout for deallocation
    layout: Layout,
}

/// Memory optimizer for tracking and optimization
pub struct MemoryOptimizer {
    /// Memory pools
    pools: Arc<Mutex<MemoryPool>>,
    /// Large allocation tracker
    large_allocations: Arc<Mutex<HashMap<usize, MappedRegion>>>,
    /// Thread-local allocation cache
    thread_cache: Arc<Mutex<ThreadLocalCache>>,
}

/// Thread-local allocation cache
/// We use raw pointers (usize) instead of NonNull for thread safety
struct ThreadLocalCache {
    /// Small object cache (storing pointer addresses as usize for Send+Sync)
    small_objects: HashMap<usize, Vec<usize>>,
    /// Cache size limit
    max_cache_size: usize,
}

impl MemoryPool {
    /// Create new memory pool
    pub fn new() -> Self {
        let mut pools = HashMap::new();
        
        // Initialize pools for common sizes
        for size in &[32, 64, 128, 256, 512, 1024, 2048, 4096, 8192] {
            pools.insert(*size, Pool::new(*size));
        }
        
        Self {
            pools,
            total_allocated: 0,
            peak_usage: 0,
            stats: AllocationStats::default(),
        }
    }
    
    /// Allocate memory from pool
    pub fn allocate(&mut self, size: usize) -> Option<NonNull<u8>> {
        // Round up to nearest size class
        let size_class = self.find_size_class(size);
        
        // Get or create pool for this size class
        let pool = self.pools.entry(size_class).or_insert_with(|| Pool::new(size_class));
        
        // Try to get from pool
        if let Some(ptr) = pool.allocate() {
            self.stats.pool_hits += 1;
            self.stats.total_allocations += 1;
            pool.used_blocks += 1;
            
            self.total_allocated += size_class;
            self.peak_usage = self.peak_usage.max(self.total_allocated);
            
            Some(ptr)
        } else {
            // Allocate new block
            self.stats.pool_misses += 1;
            self.stats.total_allocations += 1;
            
            let layout = Layout::from_size_align(size_class, 8).ok()?;
            let ptr = unsafe { alloc(layout) };
            
            if ptr.is_null() {
                None
            } else {
                pool.total_blocks += 1;
                pool.used_blocks += 1;
                
                self.total_allocated += size_class;
                self.peak_usage = self.peak_usage.max(self.total_allocated);
                
                NonNull::new(ptr)
            }
        }
    }
    
    /// Deallocate memory back to pool
    pub fn deallocate(&mut self, ptr: NonNull<u8>, size: usize) {
        let size_class = self.find_size_class(size);
        
        if let Some(pool) = self.pools.get_mut(&size_class) {
            pool.deallocate(ptr);
            pool.used_blocks = pool.used_blocks.saturating_sub(1);
            
            self.stats.total_deallocations += 1;
            self.total_allocated = self.total_allocated.saturating_sub(size_class);
        }
    }
    
    /// Find appropriate size class
    fn find_size_class(&self, size: usize) -> usize {
        // Common size classes
        const SIZE_CLASSES: &[usize] = &[32, 64, 128, 256, 512, 1024, 2048, 4096, 8192];
        
        for &class in SIZE_CLASSES {
            if size <= class {
                return class;
            }
        }
        
        // For larger sizes, round up to next power of 2
        size.next_power_of_two()
    }
    
    /// Get memory statistics
    pub fn stats(&self) -> MemoryStatistics {
        MemoryStatistics {
            total_allocated: self.total_allocated,
            peak_usage: self.peak_usage,
            pool_efficiency: if self.stats.total_allocations > 0 {
                self.stats.pool_hits as f64 / self.stats.total_allocations as f64
            } else {
                0.0
            },
            fragmentation: self.calculate_fragmentation(),
        }
    }
    
    /// Calculate memory fragmentation
    fn calculate_fragmentation(&self) -> f64 {
        let mut total_capacity = 0;
        let mut total_used = 0;
        
        for pool in self.pools.values() {
            total_capacity += pool.total_blocks * pool.size;
            total_used += pool.used_blocks * pool.size;
        }
        
        if total_capacity > 0 {
            1.0 - (total_used as f64 / total_capacity as f64)
        } else {
            0.0
        }
    }
}

impl Pool {
    /// Create new pool for specific size
    fn new(size: usize) -> Self {
        Self {
            size,
            free_blocks: VecDeque::with_capacity(1024),
            total_blocks: 0,
            used_blocks: 0,
        }
    }
    
    /// Allocate from pool
    fn allocate(&mut self) -> Option<NonNull<u8>> {
        self.free_blocks.pop_front().and_then(|addr| {
            NonNull::new(addr as *mut u8)
        })
    }
    
    /// Return to pool
    fn deallocate(&mut self, ptr: NonNull<u8>) {
        // Limit pool size to prevent unbounded growth
        if self.free_blocks.len() < 10000 {
            self.free_blocks.push_back(ptr.as_ptr() as usize);
        } else {
            // Actually free the memory
            unsafe {
                let layout = Layout::from_size_align_unchecked(self.size, 8);
                dealloc(ptr.as_ptr(), layout);
            }
            self.total_blocks = self.total_blocks.saturating_sub(1);
        }
    }
}

impl MemoryOptimizer {
    /// Create new memory optimizer
    pub fn new() -> Self {
        Self {
            pools: Arc::new(Mutex::new(MemoryPool::new())),
            large_allocations: Arc::new(Mutex::new(HashMap::new())),
            thread_cache: Arc::new(Mutex::new(ThreadLocalCache::new())),
        }
    }
    
    /// Allocate optimized memory
    pub fn allocate(&self, size: usize) -> Option<NonNull<u8>> {
        if size <= 8192 {
            // Try thread-local cache first
            if let Some(ptr) = self.thread_cache.lock().unwrap().get(size) {
                return Some(ptr);
            }
            
            // Fall back to pool
            self.pools.lock().unwrap().allocate(size)
        } else {
            // Large allocation - use memory mapping
            self.allocate_large(size)
        }
    }
    
    /// Deallocate memory
    pub fn deallocate(&self, ptr: NonNull<u8>, size: usize) {
        if size <= 8192 {
            // Try to cache locally
            if !self.thread_cache.lock().unwrap().put(ptr, size) {
                // Cache full, return to pool
                self.pools.lock().unwrap().deallocate(ptr, size);
            }
        } else {
            // Large deallocation
            self.deallocate_large(ptr);
        }
    }
    
    /// Allocate large memory region
    fn allocate_large(&self, size: usize) -> Option<NonNull<u8>> {
        let layout = Layout::from_size_align(size, 4096).ok()?; // Page-aligned
        let ptr = unsafe { alloc(layout) };
        
        NonNull::new(ptr).map(|ptr| {
            let region = MappedRegion {
                ptr: ptr.as_ptr() as usize,
                size,
                layout,
            };
            
            self.large_allocations.lock().unwrap().insert(ptr.as_ptr() as usize, region);
            ptr
        })
    }
    
    /// Deallocate large memory region
    fn deallocate_large(&self, ptr: NonNull<u8>) {
        let mut allocations = self.large_allocations.lock().unwrap();
        
        if let Some(region) = allocations.remove(&(ptr.as_ptr() as usize)) {
            unsafe {
                dealloc(region.ptr as *mut u8, region.layout);
            }
        }
    }
    
    /// Get current memory usage
    pub fn current_usage(&self) -> usize {
        self.pools.lock().unwrap().total_allocated
    }
    
    /// Force garbage collection
    pub fn collect(&self) {
        // Clear thread-local caches
        self.thread_cache.lock().unwrap().clear();
        
        // Trim pools
        let mut pools = self.pools.lock().unwrap();
        for pool in pools.pools.values_mut() {
            // Free excess blocks
            while pool.free_blocks.len() > 100 {
                if let Some(ptr) = pool.free_blocks.pop_back() {
                    unsafe {
                        let layout = Layout::from_size_align_unchecked(pool.size, 8);
                        dealloc(ptr as *mut u8, layout);
                    }
                    pool.total_blocks = pool.total_blocks.saturating_sub(1);
                }
            }
        }
    }
    
    /// Get memory statistics
    pub fn statistics(&self) -> MemoryStatistics {
        self.pools.lock().unwrap().stats()
    }
}

impl ThreadLocalCache {
    fn new() -> Self {
        Self {
            small_objects: HashMap::new(),
            max_cache_size: 1024 * 1024, // 1MB per thread
        }
    }
    
    fn get(&mut self, size: usize) -> Option<NonNull<u8>> {
        self.small_objects.get_mut(&size)?.pop().and_then(|addr| {
            NonNull::new(addr as *mut u8)
        })
    }
    
    fn put(&mut self, ptr: NonNull<u8>, size: usize) -> bool {
        let cache = self.small_objects.entry(size).or_insert_with(Vec::new);
        
        if cache.len() < 100 {
            cache.push(ptr.as_ptr() as usize);
            true
        } else {
            false
        }
    }
    
    fn clear(&mut self) {
        self.small_objects.clear();
    }
}

/// Memory usage statistics
#[derive(Debug, Clone)]
pub struct MemoryStatistics {
    /// Currently allocated
    pub total_allocated: usize,
    /// Peak usage
    pub peak_usage: usize,
    /// Pool hit rate
    pub pool_efficiency: f64,
    /// Fragmentation ratio
    pub fragmentation: f64,
}

/// Smart pointer for pooled allocations
pub struct PooledBox<T> {
    ptr: NonNull<T>,
    optimizer: Arc<MemoryOptimizer>,
}

impl<T> PooledBox<T> {
    /// Create new pooled box
    pub fn new(value: T, optimizer: Arc<MemoryOptimizer>) -> Option<Self> {
        let size = std::mem::size_of::<T>();
        let ptr = optimizer.allocate(size)?;
        
        unsafe {
            ptr.cast::<T>().as_ptr().write(value);
        }
        
        Some(Self {
            ptr: ptr.cast(),
            optimizer,
        })
    }
}

impl<T> std::ops::Deref for PooledBox<T> {
    type Target = T;
    
    fn deref(&self) -> &Self::Target {
        unsafe { self.ptr.as_ref() }
    }
}

impl<T> std::ops::DerefMut for PooledBox<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.ptr.as_mut() }
    }
}

impl<T> Drop for PooledBox<T> {
    fn drop(&mut self) {
        unsafe {
            std::ptr::drop_in_place(self.ptr.as_ptr());
        }
        
        let size = std::mem::size_of::<T>();
        self.optimizer.deallocate(self.ptr.cast(), size);
    }
}

// Make PooledBox Send + Sync if T is
unsafe impl<T: Send> Send for PooledBox<T> {}
unsafe impl<T: Sync> Sync for PooledBox<T> {}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_memory_pool() {
        let mut pool = MemoryPool::new();
        
        // Allocate and deallocate
        let ptr1 = pool.allocate(64).unwrap();
        let ptr2 = pool.allocate(64).unwrap();
        
        assert_ne!(ptr1, ptr2);
        
        pool.deallocate(ptr1, 64);
        
        // Should reuse the deallocated block
        let ptr3 = pool.allocate(64).unwrap();
        assert_eq!(ptr1, ptr3);
        
        // Cleanup
        pool.deallocate(ptr2, 64);
        pool.deallocate(ptr3, 64);
    }
    
    #[test]
    fn test_memory_optimizer() {
        let optimizer = Arc::new(MemoryOptimizer::new());
        
        // Test small allocation
        let small = optimizer.allocate(100).unwrap();
        optimizer.deallocate(small, 100);
        
        // Test large allocation
        let large = optimizer.allocate(10000).unwrap();
        optimizer.deallocate(large, 10000);
        
        // Test pooled box
        let boxed = PooledBox::new(42, optimizer.clone()).unwrap();
        assert_eq!(*boxed, 42);
    }
}