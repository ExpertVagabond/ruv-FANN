use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};
use std::time::Duration;
use rand::prelude::*;

// Mock structures for benchmarking - these would be replaced with actual implementations
#[derive(Clone)]
struct RootVector([f32; 32]);

impl RootVector {
    fn new_random() -> Self {
        let mut rng = thread_rng();
        let mut vec = [0.0f32; 32];
        for i in 0..32 {
            vec[i] = rng.gen_range(-1.0..1.0);
        }
        Self(vec)
    }

    fn dot(&self, other: &RootVector) -> f32 {
        self.0.iter().zip(other.0.iter()).map(|(a, b)| a * b).sum()
    }

    fn normalize(&mut self) {
        let norm: f32 = self.0.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for x in &mut self.0 {
                *x /= norm;
            }
        }
    }
}

struct CartanProjection {
    basis_matrix: [[f32; 32]; 32], // H matrix (32xd simplified to 32x32 for benchmarking)
}

impl CartanProjection {
    fn new() -> Self {
        // Initialize with orthonormal basis
        let mut basis = [[0.0f32; 32]; 32];
        for i in 0..32 {
            basis[i][i] = (2.0f32).sqrt(); // Cartan convention: ⟨αᵢ, αᵢ⟩ = 2
        }
        Self { basis_matrix: basis }
    }

    fn project_to_root(&self, input: &[f32]) -> RootVector {
        let mut result = [0.0f32; 32];
        
        // Compute dot products with each basis vector
        for i in 0..32 {
            let mut sum = 0.0f32;
            for j in 0..input.len().min(32) {
                sum += self.basis_matrix[i][j] * input[j];
            }
            result[i] = sum;
        }
        
        RootVector(result)
    }

    #[cfg(target_arch = "wasm32")]
    fn project_to_root_simd(&self, input: &[f32]) -> RootVector {
        use core::arch::wasm32::*;
        let mut result = [0.0f32; 32];
        
        // Process 4 elements at a time using WASM SIMD
        for i in 0..32 {
            let mut sum = f32x4_splat(0.0);
            let chunks = input.chunks_exact(4);
            let remainder = chunks.remainder();
            
            for (j, chunk) in chunks.enumerate() {
                let a = v128_load(chunk.as_ptr() as *const v128);
                let b = v128_load(&self.basis_matrix[i][j * 4] as *const f32 as *const v128);
                sum = f32x4_add(sum, f32x4_mul(a, b));
            }
            
            // Sum the vector elements
            result[i] = f32x4_extract_lane::<0>(sum) + 
                       f32x4_extract_lane::<1>(sum) + 
                       f32x4_extract_lane::<2>(sum) + 
                       f32x4_extract_lane::<3>(sum);
            
            // Handle remainder
            for (k, &val) in remainder.iter().enumerate() {
                result[i] += self.basis_matrix[i][chunks.len() * 4 + k] * val;
            }
        }
        
        RootVector(result)
    }
}

struct CartanAttention {
    target_matrix: [[f32; 32]; 32], // Target Cartan matrix C_target
    lambda: f32,                      // Regularization strength
}

impl CartanAttention {
    fn new(lambda: f32) -> Self {
        // Initialize target matrix (F4-like structure)
        let mut target = [[0.0f32; 32]; 32];
        for i in 0..32 {
            target[i][i] = 2.0; // Diagonal elements
            if i > 0 {
                target[i][i-1] = -1.0; // Adjacent connections
                target[i-1][i] = -1.0;
            }
        }
        Self { target_matrix: target, lambda }
    }

    fn compute_attention(&self, queries: &[RootVector], keys: &[RootVector]) -> Vec<Vec<f32>> {
        let n = queries.len();
        let m = keys.len();
        let mut scores = vec![vec![0.0f32; m]; n];
        
        for i in 0..n {
            for j in 0..m {
                scores[i][j] = queries[i].dot(&keys[j]);
            }
        }
        
        scores
    }

    fn apply_cartan_regularization(&self, scores: &mut Vec<Vec<f32>>, indices: &[(usize, usize)]) {
        for &(i, j) in indices {
            if i < 32 && j < 32 {
                let bias = self.lambda * self.target_matrix[i][j];
                if i < scores.len() && j < scores[i].len() {
                    scores[i][j] += bias;
                }
            }
        }
    }

    fn orthogonalize_vectors(&self, vectors: &mut [RootVector]) {
        // Gram-Schmidt orthogonalization
        for i in 1..vectors.len() {
            let mut v = vectors[i].clone();
            for j in 0..i {
                let proj = vectors[i].dot(&vectors[j]);
                for k in 0..32 {
                    v.0[k] -= proj * vectors[j].0[k];
                }
            }
            v.normalize();
            vectors[i] = v;
        }
    }
}

// Benchmark: Projection to root space
fn bench_projection(c: &mut Criterion) {
    let mut group = c.benchmark_group("cartan_projection");
    let projection = CartanProjection::new();
    
    for dim in [768, 1024, 2048, 4096].iter() {
        let input: Vec<f32> = (0..*dim).map(|i| (i as f32 * 0.001).sin()).collect();
        
        group.throughput(Throughput::Elements(*dim as u64));
        group.bench_with_input(
            BenchmarkId::new("project_to_root", dim),
            dim,
            |b, _| {
                b.iter(|| {
                    projection.project_to_root(black_box(&input))
                });
            },
        );
    }
    
    #[cfg(target_arch = "wasm32")]
    {
        for dim in [768, 1024, 2048, 4096].iter() {
            let input: Vec<f32> = (0..*dim).map(|i| (i as f32 * 0.001).sin()).collect();
            
            group.throughput(Throughput::Elements(*dim as u64));
            group.bench_with_input(
                BenchmarkId::new("project_to_root_simd", dim),
                dim,
                |b, _| {
                    b.iter(|| {
                        projection.project_to_root_simd(black_box(&input))
                    });
                },
            );
        }
    }
    
    group.finish();
}

// Benchmark: Attention computation with Cartan regularization
fn bench_attention(c: &mut Criterion) {
    let mut group = c.benchmark_group("cartan_attention");
    let attention = CartanAttention::new(0.1);
    
    for seq_len in [32, 64, 128, 256, 512].iter() {
        let queries: Vec<_> = (0..*seq_len).map(|_| RootVector::new_random()).collect();
        let keys: Vec<_> = (0..*seq_len).map(|_| RootVector::new_random()).collect();
        let indices: Vec<_> = (0..*seq_len).flat_map(|i| {
            (0..*seq_len).map(move |j| (i, j))
        }).collect();
        
        group.throughput(Throughput::Elements((*seq_len * *seq_len) as u64));
        group.bench_with_input(
            BenchmarkId::new("compute_attention", seq_len),
            seq_len,
            |b, _| {
                b.iter(|| {
                    attention.compute_attention(black_box(&queries), black_box(&keys))
                });
            },
        );
        
        group.bench_with_input(
            BenchmarkId::new("apply_regularization", seq_len),
            seq_len,
            |b, _| {
                b.iter(|| {
                    let mut scores = attention.compute_attention(&queries, &keys);
                    attention.apply_cartan_regularization(black_box(&mut scores), black_box(&indices));
                    scores
                });
            },
        );
    }
    
    group.finish();
}

// Benchmark: Orthogonalization
fn bench_orthogonalization(c: &mut Criterion) {
    let mut group = c.benchmark_group("orthogonalization");
    let attention = CartanAttention::new(0.1);
    
    for n_vectors in [8, 16, 32, 64].iter() {
        let mut vectors: Vec<_> = (0..*n_vectors).map(|_| RootVector::new_random()).collect();
        
        group.throughput(Throughput::Elements(*n_vectors as u64));
        group.bench_with_input(
            BenchmarkId::new("gram_schmidt", n_vectors),
            n_vectors,
            |b, _| {
                b.iter(|| {
                    let mut v = vectors.clone();
                    attention.orthogonalize_vectors(black_box(&mut v));
                    v
                });
            },
        );
    }
    
    group.finish();
}

// Benchmark: Memory pooling for micro-nets
fn bench_memory_pooling(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_pooling");
    
    #[derive(Clone)]
    struct MicroNet {
        weights: Vec<f32>,
        activations: Vec<RootVector>,
    }
    
    impl MicroNet {
        fn new(size: usize) -> Self {
            Self {
                weights: vec![0.1; size * 32],
                activations: vec![RootVector::new_random(); size],
            }
        }
        
        fn forward(&mut self, input: &RootVector) -> RootVector {
            // Simulate forward pass
            let mut output = input.clone();
            for (i, activation) in self.activations.iter_mut().enumerate() {
                let weight_offset = i * 32;
                for j in 0..32 {
                    output.0[j] = (output.0[j] * self.weights[weight_offset + j]).tanh();
                }
                *activation = output.clone();
            }
            output
        }
    }
    
    struct MemoryPool {
        pool: Vec<MicroNet>,
        free_indices: Vec<usize>,
    }
    
    impl MemoryPool {
        fn new(capacity: usize, net_size: usize) -> Self {
            Self {
                pool: (0..capacity).map(|_| MicroNet::new(net_size)).collect(),
                free_indices: (0..capacity).collect(),
            }
        }
        
        fn allocate(&mut self) -> Option<usize> {
            self.free_indices.pop()
        }
        
        fn deallocate(&mut self, index: usize) {
            self.free_indices.push(index);
        }
        
        fn get_mut(&mut self, index: usize) -> Option<&mut MicroNet> {
            self.pool.get_mut(index)
        }
    }
    
    for pool_size in [16, 32, 64, 128].iter() {
        let mut pool = MemoryPool::new(*pool_size, 10);
        let input = RootVector::new_random();
        
        group.bench_with_input(
            BenchmarkId::new("pool_allocation", pool_size),
            pool_size,
            |b, _| {
                b.iter(|| {
                    // Simulate allocation, use, and deallocation
                    let mut indices = Vec::new();
                    
                    // Allocate half the pool
                    for _ in 0..(*pool_size / 2) {
                        if let Some(idx) = pool.allocate() {
                            indices.push(idx);
                        }
                    }
                    
                    // Use the allocated nets
                    for &idx in &indices {
                        if let Some(net) = pool.get_mut(idx) {
                            net.forward(black_box(&input));
                        }
                    }
                    
                    // Deallocate
                    for idx in indices {
                        pool.deallocate(idx);
                    }
                });
            },
        );
    }
    
    group.finish();
}

// Benchmark: Parallel micro-net execution
fn bench_parallel_execution(c: &mut Criterion) {
    let mut group = c.benchmark_group("parallel_execution");
    group.sample_size(50);
    group.measurement_time(Duration::from_secs(10));
    
    use rayon::prelude::*;
    
    struct SwarmOrchestrator {
        micro_nets: Vec<Box<dyn Fn(&RootVector) -> RootVector + Send + Sync>>,
    }
    
    impl SwarmOrchestrator {
        fn new(n_agents: usize) -> Self {
            let micro_nets: Vec<_> = (0..n_agents)
                .map(|i| {
                    let weights = vec![0.1 + i as f32 * 0.01; 32];
                    Box::new(move |input: &RootVector| {
                        let mut output = input.clone();
                        for j in 0..32 {
                            output.0[j] = (output.0[j] * weights[j]).tanh();
                        }
                        output
                    }) as Box<dyn Fn(&RootVector) -> RootVector + Send + Sync>
                })
                .collect();
            
            Self { micro_nets }
        }
        
        fn process_parallel(&self, inputs: &[RootVector]) -> Vec<RootVector> {
            inputs.par_iter()
                .zip(self.micro_nets.par_iter().cycle())
                .map(|(input, net)| net(input))
                .collect()
        }
        
        fn process_sequential(&self, inputs: &[RootVector]) -> Vec<RootVector> {
            inputs.iter()
                .zip(self.micro_nets.iter().cycle())
                .map(|(input, net)| net(input))
                .collect()
        }
    }
    
    for n_agents in [4, 8, 16, 32].iter() {
        let orchestrator = SwarmOrchestrator::new(*n_agents);
        let batch_size = 1000;
        let inputs: Vec<_> = (0..batch_size).map(|_| RootVector::new_random()).collect();
        
        group.throughput(Throughput::Elements(batch_size as u64));
        
        group.bench_with_input(
            BenchmarkId::new("sequential", n_agents),
            n_agents,
            |b, _| {
                b.iter(|| {
                    orchestrator.process_sequential(black_box(&inputs))
                });
            },
        );
        
        group.bench_with_input(
            BenchmarkId::new("parallel", n_agents),
            n_agents,
            |b, _| {
                b.iter(|| {
                    orchestrator.process_parallel(black_box(&inputs))
                });
            },
        );
    }
    
    group.finish();
}

// Benchmark: Drift tracking
fn bench_drift_tracking(c: &mut Criterion) {
    let mut group = c.benchmark_group("drift_tracking");
    
    struct DriftTracker {
        history: Vec<RootVector>,
        window_size: usize,
    }
    
    impl DriftTracker {
        fn new(window_size: usize) -> Self {
            Self {
                history: Vec::with_capacity(window_size),
                window_size,
            }
        }
        
        fn update(&mut self, vector: RootVector) -> f32 {
            if self.history.len() >= self.window_size {
                self.history.remove(0);
            }
            
            let drift = if !self.history.is_empty() {
                let avg_dot: f32 = self.history.iter()
                    .map(|h| vector.dot(h))
                    .sum::<f32>() / self.history.len() as f32;
                1.0 - avg_dot // Higher value = more drift
            } else {
                0.0
            };
            
            self.history.push(vector);
            drift
        }
    }
    
    for window_size in [10, 50, 100, 500].iter() {
        let mut tracker = DriftTracker::new(*window_size);
        let vectors: Vec<_> = (0..1000).map(|_| RootVector::new_random()).collect();
        
        group.bench_with_input(
            BenchmarkId::new("update_drift", window_size),
            window_size,
            |b, _| {
                b.iter(|| {
                    let mut total_drift = 0.0;
                    for v in &vectors {
                        total_drift += tracker.update(v.clone());
                    }
                    black_box(total_drift)
                });
            },
        );
    }
    
    group.finish();
}

// Benchmark: WASM vs Native performance comparison
fn bench_wasm_native_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("wasm_native_comparison");
    
    // Simulate compute-intensive operation
    fn neural_compute(input: &[f32]) -> Vec<f32> {
        let mut output = input.to_vec();
        for _ in 0..5 {
            for i in 0..output.len() {
                output[i] = (output[i] * 2.0 - 1.0).tanh();
            }
        }
        output
    }
    
    for size in [100, 1000, 10000].iter() {
        let input: Vec<f32> = (0..*size).map(|i| (i as f32 * 0.001).sin()).collect();
        
        group.throughput(Throughput::Elements(*size as u64));
        
        // Native Rust benchmark
        group.bench_with_input(
            BenchmarkId::new("native", size),
            size,
            |b, _| {
                b.iter(|| {
                    neural_compute(black_box(&input))
                });
            },
        );
        
        // WASM simulation (in actual WASM, this would call into WASM module)
        #[cfg(target_arch = "wasm32")]
        group.bench_with_input(
            BenchmarkId::new("wasm", size),
            size,
            |b, _| {
                b.iter(|| {
                    // In real implementation, this would call WASM module
                    neural_compute(black_box(&input))
                });
            },
        );
    }
    
    group.finish();
}

// Benchmark: Cartan matrix computation overhead
fn bench_cartan_overhead(c: &mut Criterion) {
    let mut group = c.benchmark_group("cartan_overhead");
    
    fn compute_cartan_loss(actual: &[[f32; 32]; 32], target: &[[f32; 32]; 32]) -> f32 {
        let mut loss = 0.0;
        for i in 0..32 {
            for j in 0..32 {
                let diff = actual[i][j] - target[i][j];
                loss += diff * diff;
            }
        }
        loss
    }
    
    let actual = [[0.1; 32]; 32];
    let target = {
        let mut t = [[0.0; 32]; 32];
        for i in 0..32 {
            t[i][i] = 2.0;
            if i > 0 {
                t[i][i-1] = -1.0;
                t[i-1][i] = -1.0;
            }
        }
        t
    };
    
    group.bench_function("compute_cartan_loss", |b| {
        b.iter(|| {
            compute_cartan_loss(black_box(&actual), black_box(&target))
        });
    });
    
    // Benchmark matrix multiplication for C_actual = H W H^T
    let h_matrix = [[0.1f32; 32]; 32];
    let w_matrix = [[0.2f32; 32]; 32];
    
    group.bench_function("matrix_triple_product", |b| {
        b.iter(|| {
            let mut temp = [[0.0f32; 32]; 32];
            let mut result = [[0.0f32; 32]; 32];
            
            // W H^T
            for i in 0..32 {
                for j in 0..32 {
                    let mut sum = 0.0;
                    for k in 0..32 {
                        sum += w_matrix[i][k] * h_matrix[j][k]; // H^T means H[j][k] not H[k][j]
                    }
                    temp[i][j] = sum;
                }
            }
            
            // H (W H^T)
            for i in 0..32 {
                for j in 0..32 {
                    let mut sum = 0.0;
                    for k in 0..32 {
                        sum += h_matrix[i][k] * temp[k][j];
                    }
                    result[i][j] = sum;
                }
            }
            
            black_box(result)
        });
    });
    
    group.finish();
}

criterion_group!(
    benches,
    bench_projection,
    bench_attention,
    bench_orthogonalization,
    bench_memory_pooling,
    bench_parallel_execution,
    bench_drift_tracking,
    bench_wasm_native_comparison,
    bench_cartan_overhead
);

criterion_main!(benches);