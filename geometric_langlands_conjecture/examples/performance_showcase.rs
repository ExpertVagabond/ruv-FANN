//! Performance optimization showcase
//!
//! This example demonstrates the performance improvements achieved
//! through the optimization system including caching, parallelization,
//! and algorithm selection.

use geometric_langlands::prelude::*;
use geometric_langlands::performance::{
    PerformanceOptimizer, 
    ConfigManager, 
    WorkloadType,
    kernels::*
};
use nalgebra::{DMatrix, DVector};
use num_complex::Complex64;
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Geometric Langlands Performance Showcase");
    println!("============================================\n");
    
    // Initialize performance optimizer
    let optimizer = PerformanceOptimizer::global();
    
    // Load and display configuration
    let config_manager = ConfigManager::load_or_default();
    println!("📊 Performance Configuration:");
    println!("   Cache Size: {} MB", config_manager.get_cache_size() / (1024 * 1024));
    println!("   Thread Count: {}", config_manager.get_thread_count());
    println!("   Block Size: {}\n", config_manager.get_block_size());
    
    // 1. Matrix Operations Benchmark
    println!("🔢 Matrix Operations Performance");
    println!("--------------------------------");
    
    demonstrate_matrix_performance()?;
    
    // 2. L-Function Computation Benchmark
    println!("\n📈 L-Function Computation Performance");
    println!("------------------------------------");
    
    demonstrate_l_function_performance()?;
    
    // 3. Caching Effectiveness
    println!("\n💾 Caching Effectiveness");
    println!("-----------------------");
    
    demonstrate_caching_effectiveness()?;
    
    // 4. Parallel Scaling
    println!("\n⚡ Parallel Execution Scaling");
    println!("-----------------------------");
    
    demonstrate_parallel_scaling()?;
    
    // 5. Memory Optimization
    println!("\n🧠 Memory Optimization");
    println!("---------------------");
    
    demonstrate_memory_optimization()?;
    
    // 6. Configuration Optimization
    println!("\n⚙️  Configuration Optimization");
    println!("------------------------------");
    
    demonstrate_configuration_optimization()?;
    
    // 7. Performance Report
    println!("\n📊 Performance Report");
    println!("---------------------");
    
    let report = optimizer.get_report();
    let metrics = optimizer.get_metrics();
    
    println!("Cache Hit Rate: {:.2}%", metrics.cache_hit_rate * 100.0);
    println!("Average Computation Time: {:?}", metrics.avg_computation_time);
    println!("Memory Usage: {:.2} MB", metrics.memory_usage as f64 / (1024.0 * 1024.0));
    println!("Total Operations: {}", metrics.total_operations);
    
    Ok(())
}

fn demonstrate_matrix_performance() -> Result<(), Box<dyn std::error::Error>> {
    let size = 256;
    let matrix_a = DMatrix::<Complex64>::identity(size, size);
    let matrix_b = DMatrix::<Complex64>::identity(size, size);
    
    // Standard multiplication
    let start = Instant::now();
    let _result1 = &matrix_a * &matrix_b;
    let standard_time = start.elapsed();
    
    // Optimized multiplication
    let start = Instant::now();
    let _result2 = OptimizedMatrixOps::matmul_blocked(&matrix_a, &matrix_b);
    let optimized_time = start.elapsed();
    
    let speedup = standard_time.as_secs_f64() / optimized_time.as_secs_f64();
    
    println!("   Matrix Size: {}x{}", size, size);
    println!("   Standard Time: {:?}", standard_time);
    println!("   Optimized Time: {:?}", optimized_time);
    println!("   Speedup: {:.2}x", speedup);
    
    Ok(())
}

fn demonstrate_l_function_performance() -> Result<(), Box<dyn std::error::Error>> {
    let group = ReductiveGroup::gl_n(3);
    let mut correspondence = LanglandsCorrespondence::new(group.clone());
    
    // Add automorphic form
    let form = AutomorphicForm::eisenstein_series(&group, 2);
    correspondence.add_automorphic_form(form)?;
    
    // First computation (cold cache)
    let start = Instant::now();
    let l_func1 = correspondence.compute_l_function()?;
    let cold_time = start.elapsed();
    
    // Second computation (warm cache)
    let start = Instant::now();
    let _l_func2 = correspondence.compute_l_function()?;
    let warm_time = start.elapsed();
    
    let cache_speedup = cold_time.as_secs_f64() / warm_time.as_secs_f64();
    
    println!("   Cold Cache Time: {:?}", cold_time);
    println!("   Warm Cache Time: {:?}", warm_time);
    println!("   Cache Speedup: {:.2}x", cache_speedup);
    
    // Evaluate L-function at multiple points
    let eval_points: Vec<Complex64> = (0..100)
        .map(|i| Complex64::new(2.0 + i as f64 * 0.01, 0.5))
        .collect();
    
    let start = Instant::now();
    for &s in &eval_points {
        let _value = l_func1.evaluate(s);
    }
    let eval_time = start.elapsed();
    
    println!("   100 Evaluations Time: {:?}", eval_time);
    
    Ok(())
}

fn demonstrate_caching_effectiveness() -> Result<(), Box<dyn std::error::Error>> {
    let optimizer = PerformanceOptimizer::global();
    let matrix = DMatrix::<Complex64>::identity(100, 100);
    
    // Clear cache
    optimizer.clear_caches();
    
    // First eigenvalue computation (cache miss)
    let start = Instant::now();
    let _eigenvals1 = OptimizedEigenvalues::compute_eigenvalues(&matrix);
    let miss_time = start.elapsed();
    
    // Second computation (cache hit)
    let start = Instant::now();
    let _eigenvals2 = OptimizedEigenvalues::compute_eigenvalues(&matrix);
    let hit_time = start.elapsed();
    
    let cache_speedup = miss_time.as_secs_f64() / hit_time.as_secs_f64();
    
    println!("   Cache Miss Time: {:?}", miss_time);
    println!("   Cache Hit Time: {:?}", hit_time);
    println!("   Cache Speedup: {:.2}x", cache_speedup);
    
    Ok(())
}

fn demonstrate_parallel_scaling() -> Result<(), Box<dyn std::error::Error>> {
    let optimizer = PerformanceOptimizer::global();
    let data: Vec<i32> = (0..100_000).collect();
    
    // Sequential computation
    let start = Instant::now();
    let _results1: Vec<i32> = data.iter().map(|&x| x * x + 2 * x + 1).collect();
    let sequential_time = start.elapsed();
    
    // Parallel computation
    let start = Instant::now();
    let _results2 = optimizer.execute_parallel(data, |&x| x * x + 2 * x + 1);
    let parallel_time = start.elapsed();
    
    let parallel_speedup = sequential_time.as_secs_f64() / parallel_time.as_secs_f64();
    
    println!("   Data Size: 100,000 elements");
    println!("   Sequential Time: {:?}", sequential_time);
    println!("   Parallel Time: {:?}", parallel_time);
    println!("   Parallel Speedup: {:.2}x", parallel_speedup);
    
    Ok(())
}

fn demonstrate_memory_optimization() -> Result<(), Box<dyn std::error::Error>> {
    use geometric_langlands::performance::memory::{MemoryOptimizer, PooledBox};
    use std::sync::Arc;
    
    let optimizer = Arc::new(MemoryOptimizer::new());
    let n_allocations = 10_000;
    
    // Standard allocations
    let start = Instant::now();
    let mut standard_data = Vec::new();
    for i in 0..n_allocations {
        let data = Box::new(Complex64::new(i as f64, 0.0));
        standard_data.push(data);
    }
    drop(standard_data);
    let standard_time = start.elapsed();
    
    // Pooled allocations
    let start = Instant::now();
    let mut pooled_data = Vec::new();
    for i in 0..n_allocations {
        if let Some(data) = PooledBox::new(Complex64::new(i as f64, 0.0), optimizer.clone()) {
            pooled_data.push(data);
        }
    }
    drop(pooled_data);
    let pooled_time = start.elapsed();
    
    let memory_speedup = standard_time.as_secs_f64() / pooled_time.as_secs_f64();
    
    println!("   Allocations: {}", n_allocations);
    println!("   Standard Time: {:?}", standard_time);
    println!("   Pooled Time: {:?}", pooled_time);
    println!("   Memory Speedup: {:.2}x", memory_speedup);
    
    Ok(())
}

fn demonstrate_configuration_optimization() -> Result<(), Box<dyn std::error::Error>> {
    let config_manager = ConfigManager::load_or_default();
    
    // Test different workload optimizations
    let workloads = [
        ("Large Matrix", WorkloadType::LargeMatrix),
        ("Many Small", WorkloadType::ManySmallComputations),
        ("Real Time", WorkloadType::RealTime),
        ("Memory Constrained", WorkloadType::MemoryConstrained),
    ];
    
    for (name, workload) in workloads.iter() {
        println!("   Optimizing for {}: ", name);
        
        let start = Instant::now();
        config_manager.optimize_for_workload(*workload)?;
        let optimization_time = start.elapsed();
        
        println!("Done in {:?}", optimization_time);
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_performance_showcase() {
        // Run a simplified version for testing
        let optimizer = PerformanceOptimizer::global();
        let matrix = DMatrix::<Complex64>::identity(10, 10);
        
        let result = OptimizedMatrixOps::matmul_blocked(&matrix, &matrix);
        assert_eq!(result, matrix);
        
        let metrics = optimizer.get_metrics();
        assert!(metrics.total_operations > 0);
    }
}