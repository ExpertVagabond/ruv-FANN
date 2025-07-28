use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use ruv_fann_simulator::{SimulatorCore, ParallelEngine, NeuralNetwork, WasmModule};
use std::sync::Arc;

/// Benchmark basic simulator operations
fn bench_simulator_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("simulator_operations");
    
    // Test different input sizes
    for size in [10, 100, 1000, 10000].iter() {
        let input: Vec<f32> = (0..*size).map(|i| (i as f32) * 0.1).collect();
        let simulator = SimulatorCore::new(256);
        
        group.bench_with_input(
            BenchmarkId::new("process_input", size),
            size,
            |b, _| {
                b.iter(|| {
                    simulator.process_input(black_box(&input))
                });
            },
        );
    }
    
    group.finish();
}

/// Benchmark parallel execution with different core counts
fn bench_parallel_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("parallel_scaling");
    group.sample_size(10); // Reduce sample size for longer benchmarks
    
    let workload_size = 1_000_000;
    let workload: Vec<f32> = (0..workload_size).map(|i| (i as f32) * 0.001).collect();
    
    for cores in [1, 4, 16, 64, 128, 256].iter() {
        let engine = ParallelEngine::new(*cores);
        
        group.bench_with_input(
            BenchmarkId::new("parallel_compute", cores),
            cores,
            |b, _| {
                b.iter(|| {
                    engine.parallel_process(black_box(&workload), |&x| {
                        // Simulate neural computation
                        let mut result = x;
                        for _ in 0..5 {
                            result = (result * 2.0 - 1.0).tanh();
                        }
                        result
                    })
                });
            },
        );
    }
    
    group.finish();
}

/// Benchmark neural network operations
fn bench_neural_network(c: &mut Criterion) {
    let mut group = c.benchmark_group("neural_network");
    
    // Different network architectures
    let architectures = vec![
        vec![10, 20, 10],
        vec![50, 100, 50],
        vec![100, 200, 100, 50],
        vec![256, 512, 256, 128, 64],
    ];
    
    for arch in architectures {
        let layer_str = arch.iter()
            .map(|&s| s.to_string())
            .collect::<Vec<_>>()
            .join("x");
        
        let network = NeuralNetwork::new(&arch);
        let input: Vec<f32> = (0..arch[0]).map(|i| (i as f32) * 0.1).collect();
        
        group.bench_with_input(
            BenchmarkId::new("forward_pass", &layer_str),
            &arch,
            |b, _| {
                b.iter(|| {
                    network.forward(black_box(&input))
                });
            },
        );
    }
    
    group.finish();
}

/// Benchmark WASM module execution
fn bench_wasm_execution(c: &mut Criterion) {
    let mut group = c.benchmark_group("wasm_execution");
    
    // Load test WASM modules
    let module_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/wasm_modules/compute.wasm");
    
    if let Ok(module) = WasmModule::load(&module_path) {
        let module = Arc::new(module);
        
        for size in [10, 100, 1000].iter() {
            let input: Vec<f32> = (0..*size).map(|i| (i as f32) * 0.1).collect();
            
            group.bench_with_input(
                BenchmarkId::new("wasm_compute", size),
                size,
                |b, _| {
                    b.iter(|| {
                        module.execute("compute", black_box(&input))
                    });
                },
            );
        }
    }
    
    group.finish();
}

/// Benchmark memory allocation patterns
fn bench_memory_patterns(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_patterns");
    
    // Test different allocation patterns
    let sizes = vec![1_000, 10_000, 100_000, 1_000_000];
    
    for size in sizes {
        group.bench_with_input(
            BenchmarkId::new("sequential_alloc", size),
            &size,
            |b, &size| {
                b.iter(|| {
                    let mut vec = Vec::with_capacity(size);
                    for i in 0..size {
                        vec.push(i as f32);
                    }
                    black_box(vec)
                });
            },
        );
        
        group.bench_with_input(
            BenchmarkId::new("parallel_alloc", size),
            &size,
            |b, &size| {
                b.iter(|| {
                    use rayon::prelude::*;
                    let vec: Vec<f32> = (0..size)
                        .into_par_iter()
                        .map(|i| i as f32)
                        .collect();
                    black_box(vec)
                });
            },
        );
    }
    
    group.finish();
}

/// Benchmark visualization rendering
fn bench_visualization(c: &mut Criterion) {
    let mut group = c.benchmark_group("visualization");
    group.sample_size(10);
    
    let network_sizes = vec![10, 50, 100, 200];
    
    for size in network_sizes {
        let network = NeuralNetwork::new(&vec![size, size * 2, size]);
        let activations = vec![vec![0.5; size], vec![0.5; size * 2], vec![0.5; size]];
        
        group.bench_with_input(
            BenchmarkId::new("render_network", size),
            &size,
            |b, _| {
                b.iter(|| {
                    let viz = network.visualize(&activations);
                    black_box(viz)
                });
            },
        );
    }
    
    group.finish();
}

/// Benchmark batch processing
fn bench_batch_processing(c: &mut Criterion) {
    let mut group = c.benchmark_group("batch_processing");
    group.sample_size(10);
    
    let batch_sizes = vec![10, 50, 100, 500];
    let simulator = Arc::new(SimulatorCore::new(256));
    
    for batch_size in batch_sizes {
        let batches: Vec<Vec<f32>> = (0..batch_size)
            .map(|i| vec![i as f32; 100])
            .collect();
        
        group.bench_with_input(
            BenchmarkId::new("batch_simulate", batch_size),
            &batch_size,
            |b, _| {
                b.iter(|| {
                    use rayon::prelude::*;
                    let results: Vec<_> = batches
                        .par_iter()
                        .map(|batch| simulator.process_input(batch))
                        .collect();
                    black_box(results)
                });
            },
        );
    }
    
    group.finish();
}

/// Benchmark different activation functions
fn bench_activation_functions(c: &mut Criterion) {
    let mut group = c.benchmark_group("activation_functions");
    
    let input_sizes = vec![1000, 10000, 100000];
    
    for size in input_sizes {
        let input: Vec<f32> = (0..size).map(|i| ((i as f32) * 0.01) - 5.0).collect();
        
        group.bench_with_input(
            BenchmarkId::new("relu", size),
            &size,
            |b, _| {
                b.iter(|| {
                    input.iter()
                        .map(|&x| if x > 0.0 { x } else { 0.0 })
                        .collect::<Vec<_>>()
                });
            },
        );
        
        group.bench_with_input(
            BenchmarkId::new("sigmoid", size),
            &size,
            |b, _| {
                b.iter(|| {
                    input.iter()
                        .map(|&x| 1.0 / (1.0 + (-x).exp()))
                        .collect::<Vec<_>>()
                });
            },
        );
        
        group.bench_with_input(
            BenchmarkId::new("tanh", size),
            &size,
            |b, _| {
                b.iter(|| {
                    input.iter()
                        .map(|&x| x.tanh())
                        .collect::<Vec<_>>()
                });
            },
        );
    }
    
    group.finish();
}

criterion_group!(
    benches,
    bench_simulator_operations,
    bench_parallel_scaling,
    bench_neural_network,
    bench_wasm_execution,
    bench_memory_patterns,
    bench_visualization,
    bench_batch_processing,
    bench_activation_functions
);

criterion_main!(benches);