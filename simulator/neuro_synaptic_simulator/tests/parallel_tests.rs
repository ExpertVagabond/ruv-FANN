//! Integration tests for parallel execution model

use neuro_synaptic_simulator::{
    core::{CoreScheduler, SchedulerConfig, DistributionStrategy, SyncMode},
    memory::{SharedMemory, LayerMemory, PartitionStrategy},
};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

#[test]
fn test_shared_memory_partitioning() {
    let mem = SharedMemory::new(1024, 4, PartitionStrategy::Equal);
    
    // Verify partitions
    assert_eq!(mem.partition_info(0), Some((0, 256)));
    assert_eq!(mem.partition_info(1), Some((256, 512)));
    assert_eq!(mem.partition_info(2), Some((512, 768)));
    assert_eq!(mem.partition_info(3), Some((768, 1024)));
}

#[test]
fn test_concurrent_partition_access() {
    let mem = Arc::new(SharedMemory::new(1024, 4, PartitionStrategy::Equal));
    let counter = Arc::new(AtomicUsize::new(0));
    
    // Spawn threads to access different partitions
    let handles: Vec<_> = (0..4)
        .map(|core_id| {
            let mem_clone = Arc::clone(&mem);
            let counter_clone = Arc::clone(&counter);
            
            std::thread::spawn(move || {
                // Write to partition
                if let Some(mut guard) = mem_clone.write_partition(core_id) {
                    let slice = guard.as_mut_slice();
                    for (i, val) in slice.iter_mut().enumerate() {
                        *val = (core_id * 1000 + i) as f32;
                    }
                    counter_clone.fetch_add(1, Ordering::Relaxed);
                }
            })
        })
        .collect();
    
    // Wait for all threads
    for handle in handles {
        handle.join().unwrap();
    }
    
    // Verify all partitions were written
    assert_eq!(counter.load(Ordering::Relaxed), 4);
    
    // Verify data
    for core_id in 0..4 {
        if let Some(guard) = mem.read_partition(core_id) {
            let slice = guard.as_slice();
            assert_eq!(slice[0], (core_id * 1000) as f32);
        }
    }
}

#[test]
fn test_scheduler_creation() {
    let config = SchedulerConfig {
        num_cores: 8,
        distribution: DistributionStrategy::SingleModel,
        sync_mode: SyncMode::Barrier,
        stack_size: 4,
        work_stealing: true,
    };
    
    let scheduler = CoreScheduler::new(config).unwrap();
    let stats = scheduler.stats();
    
    assert_eq!(stats.num_cores, 8);
    assert_eq!(stats.active_work, 0);
}

#[test]
fn test_layer_execution() {
    let config = SchedulerConfig {
        num_cores: 4,
        distribution: DistributionStrategy::SingleModel,
        sync_mode: SyncMode::Barrier,
        stack_size: 4,
        work_stealing: false,
    };
    
    let scheduler = CoreScheduler::new(config).unwrap();
    let layer = LayerMemory::new(128, 64, 4, PartitionStrategy::Equal);
    
    let counter = Arc::new(AtomicUsize::new(0));
    let counter_clone = Arc::clone(&counter);
    
    // Execute layer computation
    scheduler.execute_layer(&layer, 0, move |work_unit, layer_mem| {
        if let Some(mut guard) = layer_mem.outputs.write_partition(work_unit.core_id) {
            let slice = guard.as_mut_slice();
            // Simple computation
            for val in slice.iter_mut() {
                *val = work_unit.core_id as f32;
            }
            counter_clone.fetch_add(1, Ordering::Relaxed);
        }
    });
    
    // Verify all cores executed
    assert_eq!(counter.load(Ordering::Relaxed), 4);
}

#[test]
fn test_batch_distribution() {
    let config = SchedulerConfig {
        num_cores: 8,
        distribution: DistributionStrategy::Batch { batch_size: 4 },
        sync_mode: SyncMode::Barrier,
        stack_size: 4,
        work_stealing: false,
    };
    
    let scheduler = CoreScheduler::new(config).unwrap();
    let layer = LayerMemory::new(256, 128, 8, PartitionStrategy::Equal);
    
    let executed_cores = Arc::new(parking_lot::Mutex::new(Vec::new()));
    let executed_clone = Arc::clone(&executed_cores);
    
    scheduler.execute_layer(&layer, 0, move |work_unit, _| {
        executed_clone.lock().push(work_unit.core_id);
    });
    
    let cores = executed_cores.lock();
    assert_eq!(cores.len(), 8);
    
    // Verify batch distribution
    // With 8 cores and batch_size 4, we have 2 cores per model
    for i in 0..4 {
        assert!(cores.contains(&(i * 2)));
        assert!(cores.contains(&(i * 2 + 1)));
    }
}

#[test]
fn test_barrier_synchronization() {
    let mem = SharedMemory::new(256, 4, PartitionStrategy::Equal);
    let barrier = mem.barrier();
    let counter = Arc::new(AtomicUsize::new(0));
    
    let handles: Vec<_> = (0..4)
        .map(|i| {
            let barrier_clone = Arc::clone(&barrier);
            let counter_clone = Arc::clone(&counter);
            
            std::thread::spawn(move || {
                // First phase
                counter_clone.fetch_add(1, Ordering::Relaxed);
                
                // Wait at barrier
                barrier_clone.wait();
                
                // Second phase - should only execute after all threads reach barrier
                let count = counter_clone.load(Ordering::Relaxed);
                assert_eq!(count, 4);
            })
        })
        .collect();
    
    for handle in handles {
        handle.join().unwrap();
    }
}