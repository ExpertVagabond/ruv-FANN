//! Simple neural network simulation example

use neuro_synaptic_simulator::{
    core::{CoreScheduler, SchedulerConfig, DistributionStrategy, SyncMode},
    memory::{SharedMemory, PartitionStrategy},
    performance::PerformanceMonitor,
    wasm::{WasmEngine, InstanceManager},
};
use std::sync::Arc;
use parking_lot::RwLock;
use anyhow::Result;

fn main() -> Result<()> {
    // Configuration
    let num_cores = 16;
    let memory_mb = 4;
    let timesteps = 1000;

    println!("Simple Neural Network Simulation");
    println!("================================");
    println!("Cores: {}", num_cores);
    println!("Memory: {}MB", memory_mb);
    println!("Timesteps: {}", timesteps);
    println!();

    // Initialize shared memory
    let memory_size = memory_mb * 1024 * 1024;
    let shared_memory = Arc::new(RwLock::new(SharedMemory::new(
        memory_size,
        num_cores,
        PartitionStrategy::Dynamic,
    )));

    // Initialize scheduler
    let scheduler_config = SchedulerConfig {
        num_cores,
        distribution_strategy: DistributionStrategy::RoundRobin,
        sync_mode: SyncMode::Barrier,
        max_batch_size: 32,
    };
    let scheduler = CoreScheduler::new(scheduler_config);

    // Initialize performance monitor
    let performance_monitor = Arc::new(PerformanceMonitor::new());

    // Initialize WASM engine
    let wasm_engine = WasmEngine::new()?;
    let instance_manager = Arc::new(RwLock::new(InstanceManager::new(num_cores)));

    // Load simple neural network WASM module
    let wasm_path = "tests/fixtures/wasm_modules/neural_net.wasm";
    if std::path::Path::new(wasm_path).exists() {
        println!("Loading WASM module: {}", wasm_path);
        let wasm_bytes = std::fs::read(wasm_path)?;

        // Create instances for each core
        for core_id in 0..num_cores {
            let instance = wasm_engine.instantiate(&wasm_bytes, shared_memory.clone())?;
            instance_manager.write().register_instance(core_id, instance);
        }

        // Run simulation
        println!("\nStarting simulation...");
        let start = std::time::Instant::now();

        for timestep in 0..timesteps {
            if timestep % 100 == 0 {
                println!("Timestep {}/{}", timestep, timesteps);
            }

            // Execute parallel computation
            scheduler.execute_timestep(timestep, &instance_manager, &performance_monitor)?;
        }

        let elapsed = start.elapsed();
        println!("\nSimulation completed in {:?}", elapsed);

        // Print performance metrics
        let metrics = performance_monitor.get_summary();
        println!("\nPerformance Metrics:");
        println!("  Total cycles: {}", metrics.total_cycles);
        println!("  Memory operations: {}", metrics.memory_operations);
        println!("  Cache hits: {}", metrics.cache_hits);
        println!("  Cache misses: {}", metrics.cache_misses);
        println!("  Average IPC: {:.2}", metrics.instructions_per_cycle);
        println!("  Throughput: {:.2} ops/sec", 
            (timesteps as f64 * num_cores as f64) / elapsed.as_secs_f64());
    } else {
        println!("WASM module not found. Please run:");
        println!("  cd tests/fixtures && ./build_wasm.sh");
    }

    Ok(())
}