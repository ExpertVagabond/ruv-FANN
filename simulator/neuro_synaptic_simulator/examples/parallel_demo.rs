//! Demonstration of parallel execution model for 256-core simulation

use neuro_synaptic_simulator::{
    core::{CoreScheduler, SchedulerConfig, DistributionStrategy, SyncMode},
    memory::{SharedMemory, LayerMemory, PartitionStrategy},
};
use std::time::Instant;

fn main() {
    println!("=== 256-Core Parallel Execution Demo ===\n");
    
    // Configuration for 256 cores
    let config = SchedulerConfig {
        num_cores: 256,
        distribution: DistributionStrategy::SingleModel,
        sync_mode: SyncMode::Barrier,
        stack_size: 8,
        work_stealing: true,
    };
    
    println!("Creating scheduler with {} cores...", config.num_cores);
    let scheduler = CoreScheduler::new(config).expect("Failed to create scheduler");
    
    // Create memory for a neural network layer
    // Example: 1024 inputs, 512 outputs
    let layer = LayerMemory::new(
        1024,  // input size
        512,   // output size
        256,   // num cores
        PartitionStrategy::Equal,
    );
    
    println!("Created layer memory:");
    println!("  Input size: 1024");
    println!("  Output size: 512");
    println!("  Cores: 256");
    println!("  Partition strategy: Equal\n");
    
    // Demonstrate different distribution strategies
    demonstrate_single_model(&scheduler, &layer);
    demonstrate_batch_processing(&scheduler, &layer);
    demonstrate_dynamic_distribution(&scheduler, &layer);
    
    println!("\nFinal statistics:");
    let stats = scheduler.stats();
    println!("  Total work completed: {}", stats.completed_work);
    println!("  Active work: {}", stats.active_work);
    
    println!("\n=== Demo Complete ===");
}

fn demonstrate_single_model(scheduler: &CoreScheduler, layer: &LayerMemory) {
    println!("--- Single Model Distribution ---");
    println!("Processing one large model across all 256 cores\n");
    
    let start = Instant::now();
    
    // Simulate forward pass computation
    scheduler.execute_layer(layer, 0, |work_unit, layer_mem| {
        // Each core processes its partition
        if let Some(mut output_guard) = layer_mem.outputs.write_partition(work_unit.core_id) {
            let output_slice = output_guard.as_mut_slice();
            
            // Simulate computation (simplified)
            for i in 0..output_slice.len() {
                output_slice[i] = (work_unit.core_id as f32) * 0.1 + (i as f32) * 0.01;
            }
        }
        
        // Simulate some work
        std::thread::sleep(std::time::Duration::from_micros(10));
    });
    
    let elapsed = start.elapsed();
    println!("  Execution time: {:?}", elapsed);
    println!("  Work units per core: 1");
    println!("  Synchronization: Barrier after layer\n");
}

fn demonstrate_batch_processing(scheduler: &CoreScheduler, layer: &LayerMemory) {
    println!("--- Batch Processing Distribution ---");
    println!("Processing 32 models, each using 8 cores\n");
    
    // Create scheduler with batch configuration
    let batch_config = SchedulerConfig {
        num_cores: 256,
        distribution: DistributionStrategy::Batch { batch_size: 32 },
        sync_mode: SyncMode::Barrier,
        stack_size: 8,
        work_stealing: true,
    };
    
    let batch_scheduler = CoreScheduler::new(batch_config)
        .expect("Failed to create batch scheduler");
    
    let start = Instant::now();
    
    batch_scheduler.execute_layer(layer, 0, |work_unit, layer_mem| {
        // Each core group processes one model
        if let Some(mut output_guard) = layer_mem.outputs.write_partition(work_unit.core_id) {
            let output_slice = output_guard.as_mut_slice();
            
            // Model index = core_id / 8
            let model_idx = work_unit.core_id / 8;
            
            for i in 0..output_slice.len() {
                output_slice[i] = (model_idx as f32) + (i as f32) * 0.001;
            }
        }
        
        std::thread::sleep(std::time::Duration::from_micros(5));
    });
    
    let elapsed = start.elapsed();
    println!("  Execution time: {:?}", elapsed);
    println!("  Models in batch: 32");
    println!("  Cores per model: 8");
    println!("  Total throughput: 32 models per batch\n");
}

fn demonstrate_dynamic_distribution(scheduler: &CoreScheduler, layer: &LayerMemory) {
    println!("--- Dynamic Work Stealing ---");
    println!("Using Rayon's work-stealing for load balancing\n");
    
    // Create scheduler with dynamic configuration
    let dynamic_config = SchedulerConfig {
        num_cores: 256,
        distribution: DistributionStrategy::Dynamic,
        sync_mode: SyncMode::Async,
        stack_size: 8,
        work_stealing: true,
    };
    
    let dynamic_scheduler = CoreScheduler::new(dynamic_config)
        .expect("Failed to create dynamic scheduler");
    
    let start = Instant::now();
    
    dynamic_scheduler.execute_layer(layer, 0, |work_unit, layer_mem| {
        // Simulate variable workload
        let work_time = if work_unit.id % 10 == 0 {
            20  // Some units take longer
        } else {
            5
        };
        
        if let Some(mut output_guard) = layer_mem.outputs.write_partition(work_unit.core_id) {
            let output_slice = output_guard.as_mut_slice();
            
            for i in 0..output_slice.len() {
                output_slice[i] = (work_unit.id as f32) * 0.1;
            }
        }
        
        std::thread::sleep(std::time::Duration::from_micros(work_time));
    });
    
    let elapsed = start.elapsed();
    println!("  Execution time: {:?}", elapsed);
    println!("  Work distribution: Dynamic with stealing");
    println!("  Load balancing: Automatic\n");
}