use neuro_synaptic_simulator::memory::{SharedMemory, MEMORY_SIZE};
use std::sync::{Arc, Barrier};
use std::thread;
use std::sync::atomic::{AtomicBool, Ordering};

#[test]
fn test_shared_memory_creation() {
    let memory = SharedMemory::new();
    assert_eq!(memory.capacity(), MEMORY_SIZE);
    assert_eq!(MEMORY_SIZE, 28 * 1024 * 1024); // 28 MB
}

#[test]
fn test_shared_memory_read_write() {
    let mut memory = SharedMemory::new();
    
    // Write data
    let data = vec![0x42; 1024];
    memory.write(0x1000, &data).unwrap();
    
    // Read data back
    let mut buffer = vec![0; 1024];
    memory.read(0x1000, &mut buffer).unwrap();
    
    assert_eq!(buffer, data);
}

#[test]
fn test_shared_memory_bounds_checking() {
    let mut memory = SharedMemory::new();
    
    // Valid write at boundary
    let data = vec![0xFF; 1024];
    let result = memory.write(MEMORY_SIZE - 1024, &data);
    assert!(result.is_ok());
    
    // Invalid write beyond boundary
    let result = memory.write(MEMORY_SIZE - 512, &data);
    assert!(result.is_err());
    assert!(matches!(result, Err(MemoryError::OutOfBounds { .. })));
    
    // Invalid read beyond boundary
    let mut buffer = vec![0; 1024];
    let result = memory.read(MEMORY_SIZE - 512, &mut buffer);
    assert!(result.is_err());
}

#[test]
fn test_shared_memory_zero_initialization() {
    let memory = SharedMemory::new();
    let mut buffer = vec![0xFF; 1024];
    
    // Read from uninitialized memory should return zeros
    memory.read(0, &mut buffer).unwrap();
    assert!(buffer.iter().all(|&b| b == 0));
}

#[test]
fn test_shared_memory_thread_safety() {
    let memory = Arc::new(SharedMemory::new());
    let barrier = Arc::new(Barrier::new(4));
    let mut handles = vec![];
    
    // Spawn multiple threads writing to different regions
    for i in 0..4 {
        let mem_clone = memory.clone();
        let barrier_clone = barrier.clone();
        
        let handle = thread::spawn(move || {
            barrier_clone.wait(); // Synchronize start
            
            let offset = i * 0x100000; // Each thread gets 1MB region
            let data = vec![i as u8; 0x100000];
            
            for _ in 0..100 {
                mem_clone.write(offset, &data).unwrap();
            }
        });
        
        handles.push(handle);
    }
    
    // Wait for all threads
    for handle in handles {
        handle.join().unwrap();
    }
    
    // Verify each region has correct data
    for i in 0..4 {
        let offset = i * 0x100000;
        let mut buffer = vec![0; 0x100000];
        memory.read(offset, &mut buffer).unwrap();
        assert!(buffer.iter().all(|&b| b == i as u8));
    }
}

#[test]
fn test_shared_memory_concurrent_read_write() {
    let memory = Arc::new(SharedMemory::new());
    let running = Arc::new(AtomicBool::new(true));
    let mut handles = vec![];
    
    // Writer thread
    let mem_writer = memory.clone();
    let running_writer = running.clone();
    let writer = thread::spawn(move || {
        let mut counter = 0u32;
        while running_writer.load(Ordering::Relaxed) {
            let data = counter.to_le_bytes();
            mem_writer.write(0, &data).unwrap();
            counter = counter.wrapping_add(1);
        }
    });
    handles.push(writer);
    
    // Multiple reader threads
    for _ in 0..3 {
        let mem_reader = memory.clone();
        let running_reader = running.clone();
        let reader = thread::spawn(move || {
            let mut buffer = [0u8; 4];
            let mut last_value = 0u32;
            
            while running_reader.load(Ordering::Relaxed) {
                mem_reader.read(0, &mut buffer).unwrap();
                let value = u32::from_le_bytes(buffer);
                
                // Value should only increase or wrap
                assert!(value >= last_value || value < 100);
                last_value = value;
            }
        });
        handles.push(reader);
    }
    
    // Let threads run for a bit
    thread::sleep(Duration::from_millis(100));
    running.store(false, Ordering::Relaxed);
    
    // Wait for all threads
    for handle in handles {
        handle.join().unwrap();
    }
}

#[test]
fn test_shared_memory_partitioning() {
    let mut memory = SharedMemory::new();
    
    // Define memory regions
    const WEIGHTS_OFFSET: usize = 0;
    const WEIGHTS_SIZE: usize = 16 * 1024 * 1024; // 16MB
    
    const ACTIVATIONS_OFFSET: usize = WEIGHTS_SIZE;
    const ACTIVATIONS_SIZE: usize = 8 * 1024 * 1024; // 8MB
    
    const IO_OFFSET: usize = WEIGHTS_SIZE + ACTIVATIONS_SIZE;
    const IO_SIZE: usize = 4 * 1024 * 1024; // 4MB
    
    // Write to each region
    let weights = vec![0xAA; WEIGHTS_SIZE];
    let activations = vec![0xBB; ACTIVATIONS_SIZE];
    let io_data = vec![0xCC; IO_SIZE];
    
    memory.write(WEIGHTS_OFFSET, &weights).unwrap();
    memory.write(ACTIVATIONS_OFFSET, &activations).unwrap();
    memory.write(IO_OFFSET, &io_data).unwrap();
    
    // Verify regions don't overlap
    let mut check = vec![0; 1024];
    
    memory.read(WEIGHTS_OFFSET, &mut check).unwrap();
    assert!(check.iter().all(|&b| b == 0xAA));
    
    memory.read(ACTIVATIONS_OFFSET, &mut check).unwrap();
    assert!(check.iter().all(|&b| b == 0xBB));
    
    memory.read(IO_OFFSET, &mut check).unwrap();
    assert!(check.iter().all(|&b| b == 0xCC));
}

#[test]
fn test_shared_memory_allocation_tracking() {
    let mut memory = SharedMemory::new();
    
    // Allocate regions
    let region1 = memory.allocate(1024 * 1024).unwrap(); // 1MB
    let region2 = memory.allocate(2 * 1024 * 1024).unwrap(); // 2MB
    
    // Regions should not overlap
    assert!(region1.end <= region2.start || region2.end <= region1.start);
    
    // Try to allocate more than available
    let result = memory.allocate(30 * 1024 * 1024); // 30MB > 28MB
    assert!(result.is_err());
    
    // Free a region
    memory.free(region1);
    
    // Should be able to allocate in freed space
    let region3 = memory.allocate(512 * 1024).unwrap();
    assert!(region3.start >= region1.start && region3.end <= region1.end);
}

#[test]
fn test_shared_memory_copy_within() {
    let mut memory = SharedMemory::new();
    
    // Write source data
    let source_data = (0..1024).map(|i| (i % 256) as u8).collect::<Vec<_>>();
    memory.write(0x1000, &source_data).unwrap();
    
    // Copy within memory
    memory.copy_within(0x1000, 0x2000, 1024).unwrap();
    
    // Verify copy
    let mut buffer = vec![0; 1024];
    memory.read(0x2000, &mut buffer).unwrap();
    assert_eq!(buffer, source_data);
}

#[test]
fn test_shared_memory_compare_and_swap() {
    let memory = Arc::new(SharedMemory::new());
    let mut handles = vec![];
    
    // Initialize value
    memory.write(0, &[0u8; 8]).unwrap();
    
    // Multiple threads trying to increment
    for _ in 0..10 {
        let mem_clone = memory.clone();
        let handle = thread::spawn(move || {
            for _ in 0..1000 {
                loop {
                    let mut current = [0u8; 8];
                    mem_clone.read(0, &mut current).unwrap();
                    let value = u64::from_le_bytes(current);
                    
                    let new_value = value + 1;
                    let new_bytes = new_value.to_le_bytes();
                    
                    if mem_clone.compare_and_swap(0, &current, &new_bytes).is_ok() {
                        break;
                    }
                }
            }
        });
        handles.push(handle);
    }
    
    for handle in handles {
        handle.join().unwrap();
    }
    
    // Verify final value
    let mut final_value = [0u8; 8];
    memory.read(0, &mut final_value).unwrap();
    assert_eq!(u64::from_le_bytes(final_value), 10000);
}

#[test]
fn test_shared_memory_fill() {
    let mut memory = SharedMemory::new();
    
    // Fill a region
    memory.fill(0x1000, 0x2000, 0x42).unwrap();
    
    // Verify filled region
    let mut buffer = vec![0; 0x2000];
    memory.read(0x1000, &mut buffer).unwrap();
    assert!(buffer.iter().all(|&b| b == 0x42));
    
    // Verify adjacent regions untouched
    let mut before = vec![0; 16];
    let mut after = vec![0; 16];
    memory.read(0x1000 - 16, &mut before).unwrap();
    memory.read(0x3000, &mut after).unwrap();
    assert!(before.iter().all(|&b| b == 0));
    assert!(after.iter().all(|&b| b == 0));
}

// Property-based testing
#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;
    
    proptest! {
        #[test]
        fn test_memory_read_write_consistency(
            offset in 0usize..MEMORY_SIZE,
            data in prop::collection::vec(any::<u8>(), 1..4096)
        ) {
            let mut memory = SharedMemory::new();
            
            if offset + data.len() <= MEMORY_SIZE {
                // Write should succeed
                prop_assert!(memory.write(offset, &data).is_ok());
                
                // Read back should match
                let mut buffer = vec![0; data.len()];
                prop_assert!(memory.read(offset, &mut buffer).is_ok());
                prop_assert_eq!(buffer, data);
            } else {
                // Write should fail with bounds error
                prop_assert!(memory.write(offset, &data).is_err());
            }
        }
        
        #[test]
        fn test_memory_isolation(
            writes in prop::collection::vec(
                (0usize..1000, prop::collection::vec(any::<u8>(), 1..100)),
                1..10
            )
        ) {
            let mut memory = SharedMemory::new();
            
            // Perform all writes
            for (offset, data) in &writes {
                let scaled_offset = offset * 1024; // Scale to avoid overlaps
                if scaled_offset + data.len() <= MEMORY_SIZE {
                    memory.write(scaled_offset, data).unwrap();
                }
            }
            
            // Verify all data intact
            for (offset, expected) in &writes {
                let scaled_offset = offset * 1024;
                if scaled_offset + expected.len() <= MEMORY_SIZE {
                    let mut buffer = vec![0; expected.len()];
                    memory.read(scaled_offset, &mut buffer).unwrap();
                    prop_assert_eq!(&buffer, expected);
                }
            }
        }
    }
}