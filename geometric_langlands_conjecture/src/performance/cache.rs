//! High-performance caching system for mathematical computations
//!
//! This module provides a sophisticated caching system with multiple
//! eviction strategies, size limits, and TTL support.

use std::collections::{HashMap, VecDeque};
use std::hash::{Hash, Hasher};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use std::any::{Any, TypeId};
use nalgebra::{DMatrix, DVector};
use num_complex::Complex64;
use serde::{Serialize, Deserialize};

/// Cache key for computational results
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct CacheKey {
    /// Operation identifier
    pub operation: String,
    /// Input parameters hash
    pub params_hash: u64,
}

impl CacheKey {
    /// Create new cache key
    pub fn new(operation: &str, params: &[impl Hash]) -> Self {
        use std::collections::hash_map::DefaultHasher;
        let mut hasher = DefaultHasher::new();
        
        for param in params {
            param.hash(&mut hasher);
        }
        
        Self {
            operation: operation.to_string(),
            params_hash: hasher.finish(),
        }
    }
    
    /// Create key from matrix
    pub fn from_matrix(matrix: &DMatrix<Complex64>, operation: &str) -> Self {
        use std::collections::hash_map::DefaultHasher;
        let mut hasher = DefaultHasher::new();
        
        // Hash dimensions and a sample of elements
        matrix.nrows().hash(&mut hasher);
        matrix.ncols().hash(&mut hasher);
        
        // Sample elements for hash (avoid hashing entire matrix for performance)
        let sample_size = 10.min(matrix.len());
        for i in 0..sample_size {
            let idx = (i * matrix.len()) / sample_size;
            let elem = matrix.as_slice()[idx];
            elem.re.to_bits().hash(&mut hasher);
            elem.im.to_bits().hash(&mut hasher);
        }
        
        Self {
            operation: operation.to_string(),
            params_hash: hasher.finish(),
        }
    }
    
    /// Create key from vector
    pub fn from_vector(vector: &DVector<Complex64>, operation: &str) -> Self {
        use std::collections::hash_map::DefaultHasher;
        let mut hasher = DefaultHasher::new();
        
        vector.len().hash(&mut hasher);
        
        // Sample elements
        let sample_size = 10.min(vector.len());
        for i in 0..sample_size {
            let idx = (i * vector.len()) / sample_size;
            let elem = vector[idx];
            elem.re.to_bits().hash(&mut hasher);
            elem.im.to_bits().hash(&mut hasher);
        }
        
        Self {
            operation: operation.to_string(),
            params_hash: hasher.finish(),
        }
    }
    
    /// Create key from two matrices
    pub fn from_matrices(a: &DMatrix<Complex64>, b: &DMatrix<Complex64>, operation: &str) -> Self {
        use std::collections::hash_map::DefaultHasher;
        let mut hasher = DefaultHasher::new();
        
        // Hash both matrices
        a.nrows().hash(&mut hasher);
        a.ncols().hash(&mut hasher);
        b.nrows().hash(&mut hasher);
        b.ncols().hash(&mut hasher);
        
        // Sample from both
        let sample_size = 5.min(a.len().min(b.len()));
        for i in 0..sample_size {
            let idx_a = (i * a.len()) / sample_size;
            let idx_b = (i * b.len()) / sample_size;
            
            a.as_slice()[idx_a].re.to_bits().hash(&mut hasher);
            a.as_slice()[idx_a].im.to_bits().hash(&mut hasher);
            b.as_slice()[idx_b].re.to_bits().hash(&mut hasher);
            b.as_slice()[idx_b].im.to_bits().hash(&mut hasher);
        }
        
        Self {
            operation: operation.to_string(),
            params_hash: hasher.finish(),
        }
    }
}

impl std::fmt::Display for CacheKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{:x}", self.operation, self.params_hash)
    }
}

/// Cache entry with metadata
struct CacheEntry {
    /// Stored value
    value: Box<dyn Any + Send + Sync>,
    /// Type ID for type safety
    type_id: TypeId,
    /// Size in bytes
    size: usize,
    /// Last access time
    last_accessed: Instant,
    /// Creation time
    created: Instant,
    /// Access count
    access_count: u64,
}

/// Cache eviction strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CacheStrategy {
    /// Least Recently Used
    LRU,
    /// Least Frequently Used
    LFU,
    /// First In First Out
    FIFO,
    /// Adaptive Replacement Cache
    ARC,
}

/// Computation cache with configurable eviction
pub struct ComputationCache {
    /// Cache storage
    entries: HashMap<CacheKey, CacheEntry>,
    /// Access order for LRU
    access_order: VecDeque<CacheKey>,
    /// Maximum cache size in bytes
    max_size: usize,
    /// Current size in bytes
    current_size: usize,
    /// Eviction strategy
    strategy: CacheStrategy,
    /// TTL for entries
    ttl: Option<Duration>,
    /// Statistics
    hits: u64,
    misses: u64,
}

impl ComputationCache {
    /// Create new cache with size limit
    pub fn new(max_size: usize) -> Self {
        Self {
            entries: HashMap::new(),
            access_order: VecDeque::new(),
            max_size,
            current_size: 0,
            strategy: CacheStrategy::LRU,
            ttl: None,
            hits: 0,
            misses: 0,
        }
    }
    
    /// Set eviction strategy
    pub fn with_strategy(mut self, strategy: CacheStrategy) -> Self {
        self.strategy = strategy;
        self
    }
    
    /// Set TTL for entries
    pub fn with_ttl(mut self, ttl: Duration) -> Self {
        self.ttl = Some(ttl);
        self
    }
    
    /// Get value from cache
    pub fn get<T: Clone + 'static>(&self, key: &CacheKey) -> Option<T> {
        if let Some(entry) = self.entries.get(key) {
            // Check TTL
            if let Some(ttl) = self.ttl {
                if entry.created.elapsed() > ttl {
                    return None;
                }
            }
            
            // Type check
            if entry.type_id != TypeId::of::<T>() {
                return None;
            }
            
            // Update access metadata (would need &mut self)
            // For now, just return the value
            if let Some(value) = entry.value.downcast_ref::<T>() {
                return Some(value.clone());
            }
        }
        
        None
    }
    
    /// Put value in cache
    pub fn put<T: Any + Send + Sync + 'static>(&mut self, key: CacheKey, value: T) {
        let size = std::mem::size_of_val(&value);
        
        // Evict if necessary
        while self.current_size + size > self.max_size && !self.entries.is_empty() {
            self.evict_one();
        }
        
        // Create entry
        let entry = CacheEntry {
            value: Box::new(value),
            type_id: TypeId::of::<T>(),
            size,
            last_accessed: Instant::now(),
            created: Instant::now(),
            access_count: 0,
        };
        
        // Update tracking
        self.current_size += size;
        self.access_order.push_back(key.clone());
        self.entries.insert(key, entry);
    }
    
    /// Evict one entry based on strategy
    fn evict_one(&mut self) {
        let key_to_evict = match self.strategy {
            CacheStrategy::LRU => self.evict_lru(),
            CacheStrategy::LFU => self.evict_lfu(),
            CacheStrategy::FIFO => self.evict_fifo(),
            CacheStrategy::ARC => self.evict_arc(),
        };
        
        if let Some(key) = key_to_evict {
            if let Some(entry) = self.entries.remove(&key) {
                self.current_size -= entry.size;
                self.access_order.retain(|k| k != &key);
            }
        }
    }
    
    /// LRU eviction
    fn evict_lru(&mut self) -> Option<CacheKey> {
        self.access_order.pop_front()
    }
    
    /// LFU eviction
    fn evict_lfu(&mut self) -> Option<CacheKey> {
        self.entries
            .iter()
            .min_by_key(|(_, entry)| entry.access_count)
            .map(|(key, _)| key.clone())
    }
    
    /// FIFO eviction
    fn evict_fifo(&mut self) -> Option<CacheKey> {
        self.entries
            .iter()
            .min_by_key(|(_, entry)| entry.created)
            .map(|(key, _)| key.clone())
    }
    
    /// ARC eviction (simplified)
    fn evict_arc(&mut self) -> Option<CacheKey> {
        // For now, fall back to LRU
        self.evict_lru()
    }
    
    /// Clear all entries
    pub fn clear(&mut self) {
        self.entries.clear();
        self.access_order.clear();
        self.current_size = 0;
    }
    
    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        CacheStats {
            hits: self.hits,
            misses: self.misses,
            hit_rate: if self.hits + self.misses > 0 {
                self.hits as f64 / (self.hits + self.misses) as f64
            } else {
                0.0
            },
            size: self.current_size,
            entries: self.entries.len(),
        }
    }
}

/// Cache statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub hit_rate: f64,
    pub size: usize,
    pub entries: usize,
}

/// Thread-safe cache wrapper
pub struct ThreadSafeCache {
    inner: Arc<RwLock<ComputationCache>>,
}

impl ThreadSafeCache {
    /// Create new thread-safe cache
    pub fn new(max_size: usize) -> Self {
        Self {
            inner: Arc::new(RwLock::new(ComputationCache::new(max_size))),
        }
    }
    
    /// Get value from cache
    pub fn get<T: Clone + 'static>(&self, key: &CacheKey) -> Option<T> {
        self.inner.read().unwrap().get(key)
    }
    
    /// Put value in cache
    pub fn put<T: Any + Send + Sync + 'static>(&self, key: CacheKey, value: T) {
        self.inner.write().unwrap().put(key, value)
    }
    
    /// Clear cache
    pub fn clear(&self) {
        self.inner.write().unwrap().clear()
    }
    
    /// Get statistics
    pub fn stats(&self) -> CacheStats {
        self.inner.read().unwrap().stats()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_cache_basic() {
        let mut cache = ComputationCache::new(1024);
        
        let key = CacheKey::new("test", &[1, 2, 3]);
        cache.put(key.clone(), 42);
        
        assert_eq!(cache.get::<i32>(&key), Some(42));
        assert_eq!(cache.get::<String>(&key), None); // Wrong type
    }
    
    #[test]
    fn test_cache_eviction() {
        let mut cache = ComputationCache::new(100);
        
        // Fill cache
        for i in 0..20 {
            let key = CacheKey::new("test", &[i]);
            cache.put(key, vec![0u8; 10]); // 10 bytes each
        }
        
        // Should have evicted some
        assert!(cache.entries.len() < 20);
        assert!(cache.current_size <= 100);
    }
    
    #[test]
    fn test_matrix_cache_key() {
        let matrix = DMatrix::<Complex64>::identity(10, 10);
        let key1 = CacheKey::from_matrix(&matrix, "test");
        let key2 = CacheKey::from_matrix(&matrix, "test");
        
        assert_eq!(key1, key2);
    }
}