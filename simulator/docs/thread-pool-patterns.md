# Thread Pool Patterns for 256-Core Simulation

## Overview

This document describes efficient thread pool patterns for simulating 256 processing cores on host machines with varying core counts. The patterns balance simulation accuracy with host performance.

## Core Simulation Strategies

### 1. Adaptive Thread Pool Architecture

```rust
use std::sync::Arc;
use crossbeam::channel::{bounded, Sender, Receiver};
use rayon::prelude::*;

pub struct AdaptiveThreadPool {
    // Logical cores (always 256)
    logical_cores: Vec<LogicalCore>,
    
    // Physical thread pool (adapts to host)
    physical_pool: rayon::ThreadPool,
    
    // Work distribution
    scheduler: WorkScheduler,
    
    // Performance metrics
    metrics: Arc<PerformanceMetrics>,
}

impl AdaptiveThreadPool {
    pub fn new() -> Result<Self> {
        // Detect optimal thread count
        let physical_cores = num_cpus::get_physical();
        let logical_cores_available = num_cpus::get();
        
        // Heuristic: Use 2x physical cores up to 64 threads
        let optimal_threads = (physical_cores * 2).min(64).max(8);
        
        println!("Host: {} physical cores, {} logical cores", 
                 physical_cores, logical_cores_available);
        println!("Simulator: Using {} threads for 256-core simulation", 
                 optimal_threads);
        
        let physical_pool = rayon::ThreadPoolBuilder::new()
            .num_threads(optimal_threads)
            .thread_name(|index| format!("sim-worker-{}", index))
            .stack_size(2 * 1024 * 1024) // 2MB stack per thread
            .build()?;
        
        Ok(Self {
            logical_cores: (0..256).map(LogicalCore::new).collect(),
            physical_pool,
            scheduler: WorkScheduler::new(optimal_threads),
            metrics: Arc::new(PerformanceMetrics::new()),
        })
    }
}
```

### 2. Work-Stealing Queue Pattern

```rust
use crossbeam::deque::{Injector, Stealer, Worker};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

pub struct WorkStealingScheduler {
    // Global queue for new tasks
    injector: Injector<SimulationTask>,
    
    // Per-thread work queues
    workers: Vec<Worker<SimulationTask>>,
    stealers: Vec<Stealer<SimulationTask>>,
    
    // Core assignment tracking
    core_assignments: [AtomicUsize; 256],
    
    // Termination flag
    shutdown: AtomicBool,
}

impl WorkStealingScheduler {
    pub fn new(num_threads: usize) -> Self {
        let mut workers = Vec::with_capacity(num_threads);
        let mut stealers = Vec::with_capacity(num_threads);
        
        for _ in 0..num_threads {
            let worker = Worker::new_fifo();
            stealers.push(worker.stealer());
            workers.push(worker);
        }
        
        Self {
            injector: Injector::new(),
            workers,
            stealers,
            core_assignments: array_init::array_init(|_| AtomicUsize::new(usize::MAX)),
            shutdown: AtomicBool::new(false),
        }
    }
    
    pub fn schedule_core_work(&self, core_id: usize, task: SimulationTask) {
        self.injector.push(task);
        self.core_assignments[core_id].store(
            thread_id::get() % self.workers.len(), 
            Ordering::Relaxed
        );
    }
    
    pub fn worker_loop(&self, worker_id: usize) {
        let worker = &self.workers[worker_id];
        
        loop {
            // Check local queue first
            let task = worker.pop().or_else(|| {
                // Try global queue
                std::iter::repeat_with(|| {
                    self.injector.steal_batch_and_pop(worker)
                })
                .find(|s| !s.is_retry())
                .and_then(|s| s.success())
            }).or_else(|| {
                // Steal from other workers
                self.stealers.iter()
                    .enumerate()
                    .filter(|(i, _)| *i != worker_id)
                    .map(|(_, s)| s.steal())
                    .find(|s| !s.is_retry())
                    .and_then(|s| s.success())
            });
            
            match task {
                Some(task) => self.execute_task(task),
                None => {
                    if self.shutdown.load(Ordering::Relaxed) {
                        break;
                    }
                    std::thread::yield_now();
                }
            }
        }
    }
}
```

### 3. Time-Sliced Core Simulation

For accurate timing simulation when physical threads < 256:

```rust
use std::time::{Duration, Instant};
use parking_lot::Mutex;

pub struct TimeSlicedSimulator {
    // Virtual time tracking
    virtual_clock: Arc<AtomicU64>,
    
    // Core execution states
    core_states: Arc<[Mutex<CoreExecutionState>; 256]>,
    
    // Time slice duration (microseconds)
    time_slice_us: u64,
    
    // Quantum size (instructions per slice)
    quantum_size: u64,
}

#[derive(Debug)]
struct CoreExecutionState {
    core_id: usize,
    instructions_executed: u64,
    virtual_cycles: u64,
    state: CoreState,
    pending_task: Option<SimulationTask>,
}

#[derive(Debug, Clone, Copy)]
enum CoreState {
    Idle,
    Running,
    Blocked,
    Complete,
}

impl TimeSlicedSimulator {
    pub fn new() -> Self {
        Self {
            virtual_clock: Arc::new(AtomicU64::new(0)),
            core_states: Arc::new(array_init::array_init(|i| {
                Mutex::new(CoreExecutionState {
                    core_id: i,
                    instructions_executed: 0,
                    virtual_cycles: 0,
                    state: CoreState::Idle,
                    pending_task: None,
                })
            })),
            time_slice_us: 100, // 100 microsecond slices
            quantum_size: 100_000, // 100k instructions per quantum
        }
    }
    
    pub fn run_time_slice(&self, thread_pool: &ThreadPool) {
        let current_virtual_time = self.virtual_clock.load(Ordering::Relaxed);
        
        // Schedule runnable cores
        let runnable_cores: Vec<usize> = (0..256)
            .filter(|&i| {
                let state = self.core_states[i].lock();
                matches!(state.state, CoreState::Running)
            })
            .collect();
        
        // Execute in parallel batches
        thread_pool.install(|| {
            runnable_cores.par_iter().for_each(|&core_id| {
                self.execute_core_quantum(core_id);
            });
        });
        
        // Advance virtual clock
        self.virtual_clock.fetch_add(self.time_slice_us, Ordering::Relaxed);
    }
    
    fn execute_core_quantum(&self, core_id: usize) {
        let mut state = self.core_states[core_id].lock();
        
        if let Some(task) = &mut state.pending_task {
            // Simulate quantum execution
            let instructions = self.quantum_size.min(task.remaining_instructions);
            task.execute_instructions(instructions);
            state.instructions_executed += instructions;
            state.virtual_cycles += instructions; // Simplified: 1 cycle per instruction
            
            if task.is_complete() {
                state.state = CoreState::Complete;
                state.pending_task = None;
            }
        }
    }
}
```

### 4. NUMA-Aware Thread Pooling

For large systems with NUMA architecture:

```rust
#[cfg(target_os = "linux")]
mod numa {
    use hwloc::{Topology, ObjectType, CpuSet};
    use std::sync::Arc;
    
    pub struct NumaAwareThreadPool {
        topology: Arc<Topology>,
        numa_nodes: Vec<NumaNode>,
        thread_pools: Vec<rayon::ThreadPool>,
    }
    
    struct NumaNode {
        id: usize,
        cpus: CpuSet,
        memory_size: usize,
        assigned_cores: Vec<usize>, // Logical core IDs
    }
    
    impl NumaAwareThreadPool {
        pub fn new() -> Result<Self> {
            let topology = Topology::new()?;
            let numa_nodes = Self::discover_numa_nodes(&topology)?;
            
            // Create thread pool per NUMA node
            let thread_pools = numa_nodes.iter()
                .map(|node| {
                    Self::create_numa_local_pool(node, &topology)
                })
                .collect::<Result<Vec<_>>>()?;
            
            // Distribute 256 logical cores across NUMA nodes
            let mut numa_aware_pool = Self {
                topology: Arc::new(topology),
                numa_nodes,
                thread_pools,
            };
            
            numa_aware_pool.distribute_logical_cores();
            
            Ok(numa_aware_pool)
        }
        
        fn distribute_logical_cores(&mut self) {
            let cores_per_node = 256 / self.numa_nodes.len();
            let mut core_id = 0;
            
            for node in &mut self.numa_nodes {
                for _ in 0..cores_per_node {
                    node.assigned_cores.push(core_id);
                    core_id += 1;
                }
            }
            
            // Assign remaining cores to last node
            while core_id < 256 {
                self.numa_nodes.last_mut().unwrap()
                    .assigned_cores.push(core_id);
                core_id += 1;
            }
        }
        
        pub fn execute_on_numa_node(&self, logical_core_id: usize, task: SimulationTask) {
            // Find which NUMA node owns this logical core
            let node_idx = self.numa_nodes.iter()
                .position(|node| node.assigned_cores.contains(&logical_core_id))
                .unwrap_or(0);
            
            // Execute on appropriate thread pool
            self.thread_pools[node_idx].install(|| {
                task.execute();
            });
        }
    }
}
```

### 5. Hybrid Synchronous/Asynchronous Pattern

Combining tokio for I/O with rayon for compute:

```rust
use tokio::runtime::Runtime;
use tokio::sync::mpsc;
use std::sync::Arc;

pub struct HybridSimulator {
    // Async runtime for I/O and coordination
    async_runtime: Arc<Runtime>,
    
    // Compute thread pool
    compute_pool: Arc<rayon::ThreadPool>,
    
    // Communication channels
    task_sender: mpsc::Sender<SimulationTask>,
    result_receiver: mpsc::Receiver<SimulationResult>,
}

impl HybridSimulator {
    pub fn new() -> Result<Self> {
        // Async runtime with limited threads for I/O
        let async_runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(4)
            .thread_name("sim-async")
            .enable_all()
            .build()?;
        
        // Compute pool sized for CPU work
        let compute_pool = rayon::ThreadPoolBuilder::new()
            .num_threads(num_cpus::get_physical())
            .thread_name(|i| format!("sim-compute-{}", i))
            .build()?;
        
        let (tx, rx) = mpsc::channel(1024);
        
        Ok(Self {
            async_runtime: Arc::new(async_runtime),
            compute_pool: Arc::new(compute_pool),
            task_sender: tx,
            result_receiver: rx,
        })
    }
    
    pub async fn run_simulation(&self, tasks: Vec<SimulationTask>) {
        use futures::stream::{FuturesUnordered, StreamExt};
        
        let mut futures = FuturesUnordered::new();
        
        for (core_id, task) in tasks.into_iter().enumerate() {
            let compute_pool = Arc::clone(&self.compute_pool);
            
            let future = tokio::task::spawn_blocking(move || {
                compute_pool.install(|| {
                    // Heavy computation in thread pool
                    let result = task.execute();
                    (core_id, result)
                })
            });
            
            futures.push(future);
        }
        
        // Collect results as they complete
        while let Some(result) = futures.next().await {
            match result {
                Ok((core_id, sim_result)) => {
                    println!("Core {} completed", core_id);
                    // Process result
                }
                Err(e) => eprintln!("Task failed: {}", e),
            }
        }
    }
}
```

## Performance Optimization Patterns

### 1. Batch Processing for Cache Efficiency

```rust
pub struct BatchProcessor {
    batch_size: usize,
    prefetch_distance: usize,
}

impl BatchProcessor {
    pub fn process_cores_in_batches(&self, cores: &[LogicalCore]) {
        // Process cores in cache-friendly batches
        cores.par_chunks(self.batch_size)
            .for_each(|batch| {
                // Prefetch next batch data
                if let Some(next_batch) = cores.get(
                    batch[0].id + self.batch_size..
                    batch[0].id + self.batch_size * 2
                ) {
                    self.prefetch_batch_data(next_batch);
                }
                
                // Process current batch
                batch.iter().for_each(|core| {
                    core.execute_cycle();
                });
            });
    }
    
    fn prefetch_batch_data(&self, batch: &[LogicalCore]) {
        use core::arch::x86_64::_mm_prefetch;
        
        unsafe {
            for core in batch {
                // Prefetch core data structures
                _mm_prefetch(
                    core as *const _ as *const i8, 
                    core::arch::x86_64::_MM_HINT_T0
                );
            }
        }
    }
}
```

### 2. Lock-Free Progress Tracking

```rust
use std::sync::atomic::{AtomicU64, AtomicU32, Ordering};

pub struct LockFreeProgress {
    // Per-core progress counters
    instructions: [AtomicU64; 256],
    cycles: [AtomicU64; 256],
    state: [AtomicU32; 256], // CoreState as u32
    
    // Global progress
    total_instructions: AtomicU64,
    active_cores: AtomicU32,
}

impl LockFreeProgress {
    pub fn update_core_progress(&self, core_id: usize, instructions: u64, cycles: u64) {
        self.instructions[core_id].fetch_add(instructions, Ordering::Relaxed);
        self.cycles[core_id].fetch_add(cycles, Ordering::Relaxed);
        self.total_instructions.fetch_add(instructions, Ordering::Relaxed);
    }
    
    pub fn get_snapshot(&self) -> ProgressSnapshot {
        // Lock-free snapshot of all counters
        let mut snapshot = ProgressSnapshot {
            per_core_instructions: [0; 256],
            per_core_cycles: [0; 256],
            total_instructions: 0,
            active_cores: 0,
        };
        
        for i in 0..256 {
            snapshot.per_core_instructions[i] = 
                self.instructions[i].load(Ordering::Relaxed);
            snapshot.per_core_cycles[i] = 
                self.cycles[i].load(Ordering::Relaxed);
        }
        
        snapshot.total_instructions = 
            self.total_instructions.load(Ordering::Relaxed);
        snapshot.active_cores = 
            self.active_cores.load(Ordering::Relaxed);
        
        snapshot
    }
}
```

### 3. CPU Affinity for Consistent Performance

```rust
#[cfg(target_os = "linux")]
pub fn set_thread_affinity(thread_id: usize, cpu_set: &[usize]) -> Result<()> {
    use nix::sched::{CpuSet, sched_setaffinity};
    use nix::unistd::Pid;
    
    let mut cpu_set_obj = CpuSet::new();
    for &cpu in cpu_set {
        cpu_set_obj.set(cpu)?;
    }
    
    sched_setaffinity(Pid::from_raw(0), &cpu_set_obj)?;
    
    Ok(())
}

pub struct AffinityOptimizedPool {
    thread_pool: ThreadPool,
    cpu_assignments: Vec<Vec<usize>>,
}

impl AffinityOptimizedPool {
    pub fn new() -> Result<Self> {
        let num_cpus = num_cpus::get();
        let threads_per_cpu = 256 / num_cpus;
        
        // Assign logical cores to physical CPUs
        let mut cpu_assignments = vec![vec![]; num_cpus];
        for core_id in 0..256 {
            let cpu_id = core_id % num_cpus;
            cpu_assignments[cpu_id].push(core_id);
        }
        
        // Create thread pool with affinity
        let thread_pool = ThreadPoolBuilder::new()
            .num_threads(num_cpus)
            .spawn_handler(|thread| {
                let cpu_id = thread.index();
                std::thread::spawn(move || {
                    #[cfg(target_os = "linux")]
                    set_thread_affinity(thread.index(), &[cpu_id]).ok();
                    thread.run();
                });
                Ok(())
            })
            .build()?;
        
        Ok(Self {
            thread_pool,
            cpu_assignments,
        })
    }
}
```

## Benchmarking and Tuning

### Performance Metrics Collection

```rust
use std::time::Instant;

pub struct SimulatorBenchmark {
    start_time: Instant,
    core_metrics: [CoreMetrics; 256],
    global_metrics: GlobalMetrics,
}

#[derive(Default)]
struct CoreMetrics {
    execution_time: Duration,
    instructions: u64,
    cache_misses: u64,
    memory_accesses: u64,
}

#[derive(Default)]
struct GlobalMetrics {
    total_time: Duration,
    throughput_ips: f64, // Instructions per second
    efficiency: f64,     // Actual vs theoretical performance
    thread_utilization: f64,
}

impl SimulatorBenchmark {
    pub fn measure_performance<F>(&mut self, f: F) -> GlobalMetrics 
    where 
        F: FnOnce() 
    {
        let start = Instant::now();
        
        f();
        
        let elapsed = start.elapsed();
        
        // Calculate metrics
        let total_instructions: u64 = self.core_metrics.iter()
            .map(|m| m.instructions)
            .sum();
        
        let throughput_ips = total_instructions as f64 / elapsed.as_secs_f64();
        
        let theoretical_ips = 256.0 * 1_000_000_000.0; // 1GHz × 256 cores
        let efficiency = throughput_ips / theoretical_ips;
        
        GlobalMetrics {
            total_time: elapsed,
            throughput_ips,
            efficiency,
            thread_utilization: self.calculate_thread_utilization(),
        }
    }
}
```

## Best Practices

1. **Dynamic Thread Count**: Adapt to host capabilities, don't force 256 OS threads
2. **Work Stealing**: Use work-stealing queues for load balancing
3. **Time Slicing**: Simulate timing accurately even with fewer physical threads
4. **NUMA Awareness**: Optimize memory locality on NUMA systems
5. **Hybrid Approach**: Use async for coordination, thread pools for compute
6. **Lock-Free Progress**: Minimize synchronization overhead
7. **CPU Affinity**: Pin threads for consistent performance
8. **Batch Processing**: Group operations for cache efficiency

## References

- [Rayon: Data Parallelism for Rust](https://github.com/rayon-rs/rayon)
- [Crossbeam: Concurrent Programming Tools](https://github.com/crossbeam-rs/crossbeam)
- [Work-Stealing Schedulers](https://en.wikipedia.org/wiki/Work_stealing)
- [NUMA Optimization Guide](https://www.kernel.org/doc/html/latest/admin-guide/mm/numa_memory_policy.html)