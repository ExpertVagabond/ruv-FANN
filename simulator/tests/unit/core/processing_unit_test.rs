use neuro_synaptic_simulator::core::ProcessingUnit;
use std::sync::{Arc, Barrier};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;
use std::time::Duration;

#[test]
fn test_processing_unit_creation() {
    let unit = ProcessingUnit::new(42);
    assert_eq!(unit.core_id(), 42);
    assert_eq!(unit.state(), ProcessingUnitState::Idle);
    assert_eq!(unit.cycles_executed(), 0);
}

#[test]
fn test_processing_unit_range_validation() {
    // Valid core IDs (0-255)
    for id in 0..=255 {
        let unit = ProcessingUnit::new(id);
        assert_eq!(unit.core_id(), id);
    }
    
    // Invalid core IDs should panic
    let result = std::panic::catch_unwind(|| {
        ProcessingUnit::new(256);
    });
    assert!(result.is_err());
}

#[test]
fn test_processing_unit_state_transitions() {
    let mut unit = ProcessingUnit::new(0);
    
    // Initial state
    assert_eq!(unit.state(), ProcessingUnitState::Idle);
    
    // Transition to Active
    unit.start_task("test_task");
    assert_eq!(unit.state(), ProcessingUnitState::Active);
    assert_eq!(unit.current_task(), Some("test_task"));
    
    // Cannot start another task while active
    let result = unit.try_start_task("another_task");
    assert!(result.is_err());
    
    // Complete task
    unit.complete_task();
    assert_eq!(unit.state(), ProcessingUnitState::Idle);
    assert_eq!(unit.current_task(), None);
}

#[test]
fn test_processing_unit_cycle_counting() {
    let mut unit = ProcessingUnit::new(0);
    
    unit.start_task("compute");
    unit.execute_cycles(1000);
    assert_eq!(unit.cycles_executed(), 1000);
    
    unit.execute_cycles(500);
    assert_eq!(unit.cycles_executed(), 1500);
    
    unit.complete_task();
    assert_eq!(unit.task_cycles(), 1500);
    
    // Cycles reset for new task
    unit.start_task("compute2");
    assert_eq!(unit.task_cycles(), 0);
}

#[test]
fn test_processing_unit_memory_access() {
    let mut unit = ProcessingUnit::new(0);
    let memory = Arc::new(SharedMemory::new());
    
    unit.attach_memory(memory.clone());
    
    // Write data
    let data = vec![1, 2, 3, 4];
    unit.write_memory(0x1000, &data).unwrap();
    
    // Read data back
    let mut buffer = vec![0; 4];
    unit.read_memory(0x1000, &mut buffer).unwrap();
    assert_eq!(buffer, data);
}

#[test]
fn test_processing_unit_concurrent_execution() {
    const NUM_CORES: usize = 8;
    let barrier = Arc::new(Barrier::new(NUM_CORES));
    let counter = Arc::new(AtomicUsize::new(0));
    let mut handles = vec![];
    
    for i in 0..NUM_CORES {
        let barrier_clone = barrier.clone();
        let counter_clone = counter.clone();
        
        let handle = thread::spawn(move || {
            let mut unit = ProcessingUnit::new(i as u8);
            
            // Synchronize start
            barrier_clone.wait();
            
            unit.start_task(&format!("task_{}", i));
            unit.execute_cycles(1000);
            unit.complete_task();
            
            counter_clone.fetch_add(1, Ordering::SeqCst);
        });
        
        handles.push(handle);
    }
    
    for handle in handles {
        handle.join().unwrap();
    }
    
    assert_eq!(counter.load(Ordering::SeqCst), NUM_CORES);
}

#[test]
fn test_processing_unit_power_states() {
    let mut unit = ProcessingUnit::new(0);
    
    // Idle power
    assert_eq!(unit.power_state(), PowerState::Idle);
    assert!(unit.power_consumption_mw() < 10.0); // Low idle power
    
    // Active power
    unit.start_task("compute");
    assert_eq!(unit.power_state(), PowerState::Active);
    assert!(unit.power_consumption_mw() > 5.0); // Higher active power
    assert!(unit.power_consumption_mw() < 10.0); // But still within 2W/256 cores
    
    // Return to idle
    unit.complete_task();
    assert_eq!(unit.power_state(), PowerState::Idle);
}

#[test]
fn test_processing_unit_wasm_execution() {
    let mut unit = ProcessingUnit::new(0);
    
    // Simple WASM module that adds two numbers
    let wasm_module = include_bytes!("../../fixtures/add.wasm");
    
    unit.load_wasm_module(wasm_module).unwrap();
    
    // Execute WASM function
    unit.start_task("wasm_add");
    let result = unit.execute_wasm_function("add", &[5i32.into(), 3i32.into()]).unwrap();
    unit.complete_task();
    
    assert_eq!(result[0].i32(), Some(8));
    assert!(unit.cycles_executed() > 0); // Should have consumed cycles
}

#[test]
fn test_processing_unit_isolation() {
    let memory = Arc::new(SharedMemory::new());
    let mut unit1 = ProcessingUnit::new(0);
    let mut unit2 = ProcessingUnit::new(1);
    
    unit1.attach_memory(memory.clone());
    unit2.attach_memory(memory.clone());
    
    // Each unit writes to its own memory region
    let data1 = vec![0xAA; 1024];
    let data2 = vec![0xBB; 1024];
    
    unit1.write_memory(0x0000, &data1).unwrap();
    unit2.write_memory(0x1000, &data2).unwrap();
    
    // Verify no overlap
    let mut check1 = vec![0; 1024];
    let mut check2 = vec![0; 1024];
    
    unit1.read_memory(0x0000, &mut check1).unwrap();
    unit2.read_memory(0x1000, &mut check2).unwrap();
    
    assert_eq!(check1, data1);
    assert_eq!(check2, data2);
}

#[test]
fn test_processing_unit_error_handling() {
    let mut unit = ProcessingUnit::new(0);
    
    // Cannot execute cycles when idle
    let result = std::panic::catch_unwind(|| {
        let mut unit = ProcessingUnit::new(0);
        unit.execute_cycles(100);
    });
    assert!(result.is_err());
    
    // Cannot complete task when idle
    let result = std::panic::catch_unwind(|| {
        let mut unit = ProcessingUnit::new(0);
        unit.complete_task();
    });
    assert!(result.is_err());
}

#[test]
fn test_processing_unit_performance_counters() {
    let mut unit = ProcessingUnit::new(0);
    
    unit.start_task("perf_test");
    unit.execute_cycles(1000);
    unit.increment_memory_accesses(50);
    unit.increment_cache_hits(45);
    unit.complete_task();
    
    let stats = unit.performance_stats();
    assert_eq!(stats.total_cycles, 1000);
    assert_eq!(stats.memory_accesses, 50);
    assert_eq!(stats.cache_hits, 45);
    assert_eq!(stats.cache_hit_rate, 0.9); // 45/50
}

// Property-based testing with proptest
#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;
    
    proptest! {
        #[test]
        fn test_processing_unit_cycles_accumulate(
            cycles in prop::collection::vec(1u64..10000, 1..100)
        ) {
            let mut unit = ProcessingUnit::new(0);
            unit.start_task("prop_test");
            
            let total: u64 = cycles.iter().sum();
            for &c in &cycles {
                unit.execute_cycles(c);
            }
            
            prop_assert_eq!(unit.cycles_executed(), total);
        }
        
        #[test]
        fn test_processing_unit_memory_bounds(
            offset in 0usize..MEMORY_SIZE,
            size in 1usize..1024
        ) {
            let mut unit = ProcessingUnit::new(0);
            let memory = Arc::new(SharedMemory::new());
            unit.attach_memory(memory);
            
            let data = vec![0x42; size];
            
            if offset + size <= MEMORY_SIZE {
                // Should succeed
                prop_assert!(unit.write_memory(offset, &data).is_ok());
            } else {
                // Should fail with bounds error
                prop_assert!(unit.write_memory(offset, &data).is_err());
            }
        }
    }
}