//! Performance optimization benchmarks
//!
//! This benchmark suite measures the effectiveness of the performance
//! optimization system and tracks regression over time.

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};
use geometric_langlands::prelude::*;
use geometric_langlands::performance::{PerformanceOptimizer, kernels::*};
use nalgebra::{DMatrix, DVector};
use num_complex::Complex64;
use std::time::Duration;

/// Benchmark matrix operations
fn benchmark_matrix_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("matrix_operations");
    group.measurement_time(Duration::from_secs(10));
    
    for size in [32, 64, 128, 256, 512].iter() {
        group.throughput(Throughput::Elements(*size * *size));
        
        let matrix_a = DMatrix::<Complex64>::identity(*size, *size);
        let matrix_b = DMatrix::<Complex64>::identity(*size, *size);
        
        // Standard multiplication
        group.bench_with_input(
            BenchmarkId::new("standard_matmul", size),
            size,
            |b, _| {
                b.iter(|| {
                    black_box(&matrix_a * &matrix_b);
                });
            },
        );
        
        // Optimized multiplication
        group.bench_with_input(
            BenchmarkId::new("optimized_matmul", size),
            size,
            |b, _| {
                b.iter(|| {
                    black_box(OptimizedMatrixOps::matmul_blocked(&matrix_a, &matrix_b));
                });
            },
        );
        
        // Matrix-vector multiplication
        let vector = DVector::<Complex64>::from_element(*size, Complex64::new(1.0, 0.0));
        
        group.bench_with_input(
            BenchmarkId::new("standard_matvec", size),
            size,
            |b, _| {
                b.iter(|| {
                    black_box(&matrix_a * &vector);
                });
            },
        );
        
        group.bench_with_input(
            BenchmarkId::new("optimized_matvec", size),
            size,
            |b, _| {
                b.iter(|| {
                    black_box(OptimizedMatrixOps::matvec_optimized(&matrix_a, &vector));
                });
            },
        );
    }
    
    group.finish();
}

/// Benchmark FFT operations
fn benchmark_fft_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("fft_operations");
    group.measurement_time(Duration::from_secs(10));
    
    for size in [64, 128, 256, 512, 1024, 2048].iter() {
        group.throughput(Throughput::Elements(*size));
        
        let data = DVector::from_fn(*size, |i, _| {
            Complex64::new((i as f64).sin(), (i as f64).cos())
        });
        
        group.bench_with_input(
            BenchmarkId::new("optimized_fft", size),
            size,
            |b, _| {
                b.iter(|| {
                    black_box(OptimizedFFT::fft_auto(&data));
                });
            },
        );
    }
    
    group.finish();
}

/// Benchmark eigenvalue computations
fn benchmark_eigenvalue_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("eigenvalue_operations");
    group.measurement_time(Duration::from_secs(15));
    
    for size in [16, 32, 64, 128].iter() {
        let matrix = DMatrix::<Complex64>::identity(*size, *size);
        
        group.bench_with_input(
            BenchmarkId::new("optimized_eigenvalues", size),
            size,
            |b, _| {
                b.iter(|| {
                    black_box(OptimizedEigenvalues::compute_eigenvalues(&matrix));
                });
            },
        );
    }
    
    group.finish();
}

/// Benchmark caching effectiveness
fn benchmark_caching_effectiveness(c: &mut Criterion) {
    let mut group = c.benchmark_group("caching_effectiveness");
    
    let optimizer = PerformanceOptimizer::global();
    let matrix_a = DMatrix::<Complex64>::identity(100, 100);
    let matrix_b = DMatrix::<Complex64>::identity(100, 100);
    
    // Cold cache (first run)
    group.bench_function("cold_cache_matmul", |b| {
        b.iter(|| {
            optimizer.clear_caches();
            black_box(optimizer.optimized_matmul(&matrix_a, &matrix_b));
        });
    });
    
    // Warm cache (subsequent runs)
    group.bench_function("warm_cache_matmul", |b| {
        // Prime the cache
        optimizer.optimized_matmul(&matrix_a, &matrix_b);
        
        b.iter(|| {
            black_box(optimizer.optimized_matmul(&matrix_a, &matrix_b));
        });
    });
    
    group.finish();
}

/// Benchmark parallel execution scaling
fn benchmark_parallel_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("parallel_scaling");
    
    let optimizer = PerformanceOptimizer::global();
    let items: Vec<i32> = (0..10000).collect();
    
    // Sequential execution
    group.bench_function("sequential_computation", |b| {
        b.iter(|| {
            let results: Vec<_> = items.iter().map(|&x| x * x + 1).collect();
            black_box(results);
        });
    });
    
    // Parallel execution
    group.bench_function("parallel_computation", |b| {
        b.iter(|| {
            let results = optimizer.execute_parallel(items.clone(), |&x| x * x + 1);
            black_box(results);
        });
    });
    
    group.finish();
}

/// Benchmark L-function computations with optimization
fn benchmark_l_function_optimization(c: &mut Criterion) {
    let mut group = c.benchmark_group("l_function_optimization");
    group.measurement_time(Duration::from_secs(20));
    
    let group_g = ReductiveGroup::gl_n(3);
    let mut correspondence = LanglandsCorrespondence::new(group_g.clone());
    
    // Add an automorphic form
    let form = AutomorphicForm::eisenstein_series(&group_g, 2);
    correspondence.add_automorphic_form(form).unwrap();
    
    // Benchmark L-function computation
    group.bench_function("optimized_l_function", |b| {
        b.iter(|| {
            let mut corr = correspondence.clone();
            black_box(corr.compute_l_function().unwrap());
        });
    });
    
    // Benchmark L-function evaluation
    let l_func = correspondence.compute_l_function().unwrap();
    let s_values: Vec<Complex64> = (0..100)
        .map(|i| Complex64::new(2.0 + i as f64 * 0.1, 0.5))
        .collect();
    
    group.bench_function("l_function_evaluation", |b| {
        b.iter(|| {
            for &s in &s_values {
                black_box(l_func.evaluate(s));
            }
        });
    });
    
    group.finish();
}

/// Benchmark memory optimization
fn benchmark_memory_optimization(c: &mut Criterion) {
    use geometric_langlands::performance::memory::{MemoryOptimizer, PooledBox};
    use std::sync::Arc;
    
    let mut group = c.benchmark_group("memory_optimization");
    
    let optimizer = Arc::new(MemoryOptimizer::new());
    
    // Standard allocation
    group.bench_function("standard_allocation", |b| {
        b.iter(|| {
            let data: Vec<Complex64> = (0..1000)
                .map(|i| Complex64::new(i as f64, 0.0))
                .collect();
            black_box(data);
        });
    });
    
    // Pooled allocation
    group.bench_function("pooled_allocation", |b| {
        b.iter(|| {
            let mut data = Vec::new();
            for i in 0..1000 {
                let boxed = PooledBox::new(Complex64::new(i as f64, 0.0), optimizer.clone());
                data.push(boxed);
            }
            black_box(data);
        });
    });
    
    group.finish();
}

/// Benchmark configuration system
fn benchmark_configuration_system(c: &mut Criterion) {
    use geometric_langlands::performance::config::{ConfigManager, WorkloadType};
    use tempfile::tempdir;
    
    let mut group = c.benchmark_group("configuration_system");
    
    let dir = tempdir().unwrap();
    let config_path = dir.path().join("bench_config.toml");
    let manager = ConfigManager::new(config_path);
    
    group.bench_function("config_update", |b| {
        b.iter(|| {
            manager.update(|config| {
                config.cache.max_size_mb += 1;
            }).unwrap();
        });
    });
    
    group.bench_function("config_optimize_workload", |b| {
        b.iter(|| {
            manager.optimize_for_workload(WorkloadType::LargeMatrix).unwrap();
        });
    });
    
    group.finish();
}

/// Benchmark profiler overhead
fn benchmark_profiler_overhead(c: &mut Criterion) {
    use geometric_langlands::performance::profiler::Profiler;
    
    let mut group = c.benchmark_group("profiler_overhead");
    
    let mut profiler = Profiler::new();
    
    // Function without profiling
    group.bench_function("no_profiling", |b| {
        b.iter(|| {
            let data: Vec<i32> = (0..1000).map(|x| x * x).collect();
            black_box(data);
        });
    });
    
    // Function with profiling
    group.bench_function("with_profiling", |b| {
        b.iter(|| {
            profiler.profile("benchmark_function", || {
                let data: Vec<i32> = (0..1000).map(|x| x * x).collect();
                black_box(data);
            });
        });
    });
    
    group.finish();
}

criterion_group!(
    performance_benches,
    benchmark_matrix_operations,
    benchmark_fft_operations,
    benchmark_eigenvalue_operations,
    benchmark_caching_effectiveness,
    benchmark_parallel_scaling,
    benchmark_l_function_optimization,
    benchmark_memory_optimization,
    benchmark_configuration_system,
    benchmark_profiler_overhead
);

criterion_main!(performance_benches);