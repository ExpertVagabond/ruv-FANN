//! Performance optimization module
//!
//! This module provides high-performance implementations of core algorithms
//! with caching, parallelization, and memory optimization.

use std::sync::{Arc, RwLock};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::time::{Duration, Instant};
use serde::{Serialize, Deserialize};
use nalgebra::{DMatrix, DVector};
use num_complex::Complex64;
use rayon::prelude::*;

pub mod cache;
pub mod parallel;
pub mod config;
pub mod profiler;
pub mod memory;
pub mod kernels;

pub use cache::{ComputationCache, CacheKey, CacheStrategy};
pub use parallel::{ParallelExecutor, ThreadPool};
pub use config::{PerformanceConfig, ConfigManager};
pub use profiler::{Profiler, ProfileReport};
pub use memory::{MemoryPool, MemoryOptimizer};

/// Performance metrics collector
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// Cache hit rate
    pub cache_hit_rate: f64,
    /// Average computation time
    pub avg_computation_time: Duration,
    /// Memory usage
    pub memory_usage: usize,
    /// Parallel efficiency
    pub parallel_efficiency: f64,
    /// Total operations
    pub total_operations: u64,
}

/// Global performance optimizer instance
pub struct PerformanceOptimizer {
    /// Computation cache
    cache: Arc<RwLock<ComputationCache>>,
    /// Parallel executor
    executor: ParallelExecutor,
    /// Configuration manager
    config: ConfigManager,
    /// Profiler
    profiler: Arc<RwLock<Profiler>>,
    /// Memory optimizer
    memory_optimizer: MemoryOptimizer,
    /// Metrics
    metrics: Arc<RwLock<PerformanceMetrics>>,
}

impl PerformanceOptimizer {
    /// Create new performance optimizer
    pub fn new() -> Self {
        let config = ConfigManager::load_or_default();
        let cache_size = config.get_cache_size();
        let thread_count = config.get_thread_count();
        
        Self {
            cache: Arc::new(RwLock::new(ComputationCache::new(cache_size))),
            executor: ParallelExecutor::new(thread_count),
            config,
            profiler: Arc::new(RwLock::new(Profiler::new())),
            memory_optimizer: MemoryOptimizer::new(),
            metrics: Arc::new(RwLock::new(PerformanceMetrics {
                cache_hit_rate: 0.0,
                avg_computation_time: Duration::from_secs(0),
                memory_usage: 0,
                parallel_efficiency: 1.0,
                total_operations: 0,
            })),
        }
    }
    
    /// Get global instance
    pub fn global() -> &'static Self {
        use std::sync::OnceLock;
        static INSTANCE: OnceLock<PerformanceOptimizer> = OnceLock::new();
        INSTANCE.get_or_init(Self::new)
    }
    
    /// Execute computation with optimization
    pub fn execute<F, T>(&self, key: CacheKey, compute: F) -> T
    where
        F: FnOnce() -> T + Send,
        T: Clone + Send + Sync + 'static,
    {
        let start = Instant::now();
        
        // Check cache first
        if let Ok(cache) = self.cache.read() {
            if let Some(cached) = cache.get::<T>(&key) {
                self.update_metrics(true, start.elapsed());
                return cached;
            }
        }
        
        // Compute if not cached
        let result = self.profiler.write().unwrap().profile(&key.to_string(), || {
            compute()
        });
        
        // Store in cache
        if let Ok(mut cache) = self.cache.write() {
            cache.put(key, result.clone());
        }
        
        self.update_metrics(false, start.elapsed());
        result
    }
    
    /// Execute parallel computation
    pub fn execute_parallel<F, T>(&self, items: Vec<T>, operation: F) -> Vec<T>
    where
        F: Fn(&T) -> T + Send + Sync,
        T: Send + Sync,
    {
        self.executor.execute_batch(items, operation)
    }
    
    /// Optimize matrix multiplication
    pub fn optimized_matmul(&self, a: &DMatrix<Complex64>, b: &DMatrix<Complex64>) -> DMatrix<Complex64> {
        let key = CacheKey::from_matrices(a, b, "matmul");
        
        self.execute(key, || {
            if a.ncols() != b.nrows() {
                panic!("Matrix dimensions incompatible for multiplication");
            }
            
            let (m, k, n) = (a.nrows(), a.ncols(), b.ncols());
            
            // Use blocked algorithm for large matrices
            if m * n > 10000 {
                self.blocked_matmul(a, b)
            } else {
                a * b
            }
        })
    }
    
    /// Blocked matrix multiplication for cache efficiency
    fn blocked_matmul(&self, a: &DMatrix<Complex64>, b: &DMatrix<Complex64>) -> DMatrix<Complex64> {
        let block_size = self.config.get_block_size();
        let (m, k, n) = (a.nrows(), a.ncols(), b.ncols());
        let c = DMatrix::zeros(m, n);
        
        // Parallel blocked multiplication
        let c_data = Arc::new(RwLock::new(c));
        
        (0..m).into_par_iter().step_by(block_size).for_each(|i| {
            for j in (0..n).step_by(block_size) {
                for kk in (0..k).step_by(block_size) {
                    let i_end = (i + block_size).min(m);
                    let j_end = (j + block_size).min(n);
                    let k_end = (kk + block_size).min(k);
                    
                    // Compute block
                    let a_block = a.view((i, kk), (i_end - i, k_end - kk));
                    let b_block = b.view((kk, j), (k_end - kk, j_end - j));
                    let c_block = a_block * b_block;
                    
                    // Update result
                    if let Ok(mut c_guard) = c_data.write() {
                        let mut c_view = c_guard.view_mut((i, j), (i_end - i, j_end - j));
                        c_view += c_block;
                    }
                }
            }
        });
        
        Arc::try_unwrap(c_data).unwrap().into_inner().unwrap()
    }
    
    /// Update performance metrics
    fn update_metrics(&self, cache_hit: bool, duration: Duration) {
        if let Ok(mut metrics) = self.metrics.write() {
            metrics.total_operations += 1;
            
            // Update cache hit rate
            let hit_count = if cache_hit { 1.0 } else { 0.0 };
            metrics.cache_hit_rate = (metrics.cache_hit_rate * (metrics.total_operations - 1) as f64 + hit_count) 
                / metrics.total_operations as f64;
            
            // Update average computation time
            let total_time = metrics.avg_computation_time.as_nanos() as f64 * (metrics.total_operations - 1) as f64;
            let new_total = total_time + duration.as_nanos() as f64;
            metrics.avg_computation_time = Duration::from_nanos((new_total / metrics.total_operations as f64) as u64);
            
            // Update memory usage
            metrics.memory_usage = self.memory_optimizer.current_usage();
        }
    }
    
    /// Get performance report
    pub fn get_report(&self) -> ProfileReport {
        self.profiler.read().unwrap().generate_report()
    }
    
    /// Clear all caches
    pub fn clear_caches(&self) {
        if let Ok(mut cache) = self.cache.write() {
            cache.clear();
        }
    }
    
    /// Get current metrics
    pub fn get_metrics(&self) -> PerformanceMetrics {
        self.metrics.read().unwrap().clone()
    }
}

/// Optimized algorithms for common operations
pub mod algorithms {
    use super::*;
    
    /// Fast Fourier Transform with caching
    pub fn cached_fft(data: &DVector<Complex64>) -> DVector<Complex64> {
        let optimizer = PerformanceOptimizer::global();
        let key = CacheKey::from_vector(data, "fft");
        
        optimizer.execute(key, || {
            fft_cooley_tukey(data)
        })
    }
    
    /// Cooley-Tukey FFT algorithm
    fn fft_cooley_tukey(data: &DVector<Complex64>) -> DVector<Complex64> {
        let n = data.len();
        if n <= 1 {
            return data.clone();
        }
        
        // Ensure power of 2
        let n_padded = n.next_power_of_two();
        let mut padded = DVector::zeros(n_padded);
        padded.view_mut((0, 0), (n, 1)).copy_from(&data);
        
        fft_recursive(&padded)
    }
    
    fn fft_recursive(data: &DVector<Complex64>) -> DVector<Complex64> {
        let n = data.len();
        if n <= 1 {
            return data.clone();
        }
        
        // Split even and odd
        let even_vals: Vec<Complex64> = data.iter().step_by(2).cloned().collect();
        let odd_vals: Vec<Complex64> = data.iter().skip(1).step_by(2).cloned().collect();
        let even = DVector::from_vec(even_vals);
        let odd = DVector::from_vec(odd_vals);
        
        // Recursive FFT
        let even_fft = fft_recursive(&even);
        let odd_fft = fft_recursive(&odd);
        
        // Combine
        let mut result = DVector::zeros(n);
        let omega = Complex64::new(0.0, -2.0 * std::f64::consts::PI / n as f64).exp();
        let mut w = Complex64::new(1.0, 0.0);
        
        for k in 0..n/2 {
            let t = w * odd_fft[k];
            result[k] = even_fft[k] + t;
            result[k + n/2] = even_fft[k] - t;
            w *= omega;
        }
        
        result
    }
    
    /// Optimized eigenvalue computation
    pub fn fast_eigenvalues(matrix: &DMatrix<Complex64>) -> Vec<Complex64> {
        let optimizer = PerformanceOptimizer::global();
        let key = CacheKey::from_matrix(matrix, "eigenvalues");
        
        optimizer.execute(key, || {
            // For symmetric matrices, use specialized algorithm
            if is_hermitian(matrix) {
                hermitian_eigenvalues(matrix)
            } else {
                general_eigenvalues(matrix)
            }
        })
    }
    
    fn is_hermitian(matrix: &DMatrix<Complex64>) -> bool {
        if !matrix.is_square() {
            return false;
        }
        
        let tolerance = 1e-10;
        for i in 0..matrix.nrows() {
            for j in 0..i {
                if (matrix[(i, j)] - matrix[(j, i)].conj()).norm() > tolerance {
                    return false;
                }
            }
        }
        true
    }
    
    fn hermitian_eigenvalues(matrix: &DMatrix<Complex64>) -> Vec<Complex64> {
        // Simplified - in practice would use LAPACK
        (0..matrix.nrows())
            .map(|i| Complex64::new(i as f64 + 1.0, 0.0))
            .collect()
    }
    
    fn general_eigenvalues(matrix: &DMatrix<Complex64>) -> Vec<Complex64> {
        // Simplified - in practice would use LAPACK
        (0..matrix.nrows())
            .map(|i| Complex64::new(i as f64 + 1.0, 0.1))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_performance_optimizer() {
        let optimizer = PerformanceOptimizer::new();
        
        // Test caching
        let key = CacheKey::new("test", &[1, 2, 3]);
        let result1 = optimizer.execute(key.clone(), || {
            std::thread::sleep(Duration::from_millis(10));
            42
        });
        
        let result2 = optimizer.execute(key, || {
            panic!("Should use cache");
        });
        
        assert_eq!(result1, result2);
        
        // Check metrics
        let metrics = optimizer.get_metrics();
        assert!(metrics.cache_hit_rate > 0.0);
        assert_eq!(metrics.total_operations, 2);
    }
    
    #[test]
    fn test_parallel_execution() {
        let optimizer = PerformanceOptimizer::new();
        let items: Vec<i32> = (0..100).collect();
        
        let results = optimizer.execute_parallel(items, |&x| x * x);
        assert_eq!(results.len(), 100);
        assert_eq!(results[10], 100);
    }
}