//! Shared memory implementation for 256-core parallel simulation
//! 
//! This module provides thread-safe shared memory structures optimized for
//! parallel access patterns in neural network simulation.

use std::sync::{Arc, Barrier};
use parking_lot::{RwLock, Mutex, RwLockReadGuard, RwLockWriteGuard};
use crossbeam::utils::CachePadded;
use serde::{Serialize, Deserialize};

/// Memory partitioning strategy for core isolation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PartitionStrategy {
    /// Each core gets equal memory partition
    Equal,
    /// Memory distributed based on workload
    Dynamic,
    /// Custom partitioning with specified boundaries
    Custom(Vec<(usize, usize)>),
}

/// Shared memory structure with configurable partitioning
#[derive(Debug)]
pub struct SharedMemory {
    /// Total memory buffer
    data: Arc<RwLock<Vec<f32>>>,
    /// Partition boundaries for each core
    partitions: Arc<Vec<CachePadded<(usize, usize)>>>,
    /// Number of cores
    num_cores: usize,
    /// Memory size per partition
    partition_size: usize,
    /// Global synchronization barrier
    barrier: Arc<Barrier>,
}

impl SharedMemory {
    /// Create new shared memory with specified size and core count
    pub fn new(total_size: usize, num_cores: usize, strategy: PartitionStrategy) -> Self {
        let data = vec![0.0f32; total_size];
        let partitions = match strategy {
            PartitionStrategy::Equal => {
                let partition_size = total_size / num_cores;
                (0..num_cores)
                    .map(|i| {
                        let start = i * partition_size;
                        let end = if i == num_cores - 1 {
                            total_size
                        } else {
                            (i + 1) * partition_size
                        };
                        CachePadded::new((start, end))
                    })
                    .collect()
            },
            PartitionStrategy::Dynamic => {
                // Initially equal, can be adjusted during runtime
                let partition_size = total_size / num_cores;
                (0..num_cores)
                    .map(|i| {
                        let start = i * partition_size;
                        let end = if i == num_cores - 1 {
                            total_size
                        } else {
                            (i + 1) * partition_size
                        };
                        CachePadded::new((start, end))
                    })
                    .collect()
            },
            PartitionStrategy::Custom(boundaries) => {
                boundaries.into_iter()
                    .map(|b| CachePadded::new(b))
                    .collect()
            },
        };
        
        let partition_size = total_size / num_cores;
        let barrier = Arc::new(Barrier::new(num_cores));
        
        Self {
            data: Arc::new(RwLock::new(data)),
            partitions: Arc::new(partitions),
            num_cores,
            partition_size,
            barrier,
        }
    }
    
    /// Get read access to a specific core's partition
    pub fn read_partition(&self, core_id: usize) -> Option<PartitionReadGuard> {
        if core_id >= self.num_cores {
            return None;
        }
        
        let (start, end) = **self.partitions.get(core_id)?;
        let data = self.data.read();
        
        Some(PartitionReadGuard {
            data,
            start,
            end,
        })
    }
    
    /// Get write access to a specific core's partition
    pub fn write_partition(&self, core_id: usize) -> Option<PartitionWriteGuard> {
        if core_id >= self.num_cores {
            return None;
        }
        
        let (start, end) = **self.partitions.get(core_id)?;
        let data = self.data.write();
        
        Some(PartitionWriteGuard {
            data,
            start,
            end,
        })
    }
    
    /// Get read access to the entire memory (for global operations)
    pub fn read_all(&self) -> RwLockReadGuard<Vec<f32>> {
        self.data.read()
    }
    
    /// Get write access to the entire memory (for initialization)
    pub fn write_all(&self) -> RwLockWriteGuard<Vec<f32>> {
        self.data.write()
    }
    
    /// Synchronization barrier for all cores
    pub fn barrier(&self) -> Arc<Barrier> {
        Arc::clone(&self.barrier)
    }
    
    /// Get the number of cores
    pub fn num_cores(&self) -> usize {
        self.num_cores
    }
    
    /// Get partition info for a core
    pub fn partition_info(&self, core_id: usize) -> Option<(usize, usize)> {
        self.partitions.get(core_id).map(|p| **p)
    }
}

/// Read guard for a memory partition
pub struct PartitionReadGuard<'a> {
    data: RwLockReadGuard<'a, Vec<f32>>,
    start: usize,
    end: usize,
}

impl<'a> PartitionReadGuard<'a> {
    /// Get slice of the partition
    pub fn as_slice(&self) -> &[f32] {
        &self.data[self.start..self.end]
    }
    
    /// Get partition size
    pub fn len(&self) -> usize {
        self.end - self.start
    }
    
    /// Check if partition is empty
    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }
}

/// Write guard for a memory partition
pub struct PartitionWriteGuard<'a> {
    data: RwLockWriteGuard<'a, Vec<f32>>,
    start: usize,
    end: usize,
}

impl<'a> PartitionWriteGuard<'a> {
    /// Get mutable slice of the partition
    pub fn as_mut_slice(&mut self) -> &mut [f32] {
        &mut self.data[self.start..self.end]
    }
    
    /// Get partition size
    pub fn len(&self) -> usize {
        self.end - self.start
    }
    
    /// Check if partition is empty
    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }
}

/// Layer-specific shared memory for neural network layers
#[derive(Debug)]
pub struct LayerMemory {
    /// Input buffer
    pub inputs: SharedMemory,
    /// Output buffer
    pub outputs: SharedMemory,
    /// Weight buffer
    pub weights: SharedMemory,
    /// Gradient buffer for backpropagation
    pub gradients: SharedMemory,
}

impl LayerMemory {
    /// Create layer memory with specified dimensions
    pub fn new(
        input_size: usize,
        output_size: usize,
        num_cores: usize,
        strategy: PartitionStrategy,
    ) -> Self {
        let weight_size = input_size * output_size;
        
        Self {
            inputs: SharedMemory::new(input_size, num_cores, strategy.clone()),
            outputs: SharedMemory::new(output_size, num_cores, strategy.clone()),
            weights: SharedMemory::new(weight_size, num_cores, strategy.clone()),
            gradients: SharedMemory::new(weight_size, num_cores, strategy),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_shared_memory_creation() {
        let mem = SharedMemory::new(1024, 4, PartitionStrategy::Equal);
        assert_eq!(mem.num_cores(), 4);
        
        // Check partition boundaries
        assert_eq!(mem.partition_info(0), Some((0, 256)));
        assert_eq!(mem.partition_info(1), Some((256, 512)));
        assert_eq!(mem.partition_info(2), Some((512, 768)));
        assert_eq!(mem.partition_info(3), Some((768, 1024)));
    }
    
    #[test]
    fn test_partition_access() {
        let mem = SharedMemory::new(1024, 4, PartitionStrategy::Equal);
        
        // Write to partition
        {
            let mut write_guard = mem.write_partition(1).unwrap();
            let slice = write_guard.as_mut_slice();
            slice[0] = 42.0;
        }
        
        // Read from partition
        {
            let read_guard = mem.read_partition(1).unwrap();
            let slice = read_guard.as_slice();
            assert_eq!(slice[0], 42.0);
        }
    }
    
    #[test]
    fn test_layer_memory() {
        let layer = LayerMemory::new(784, 128, 8, PartitionStrategy::Equal);
        
        // Check dimensions
        assert_eq!(layer.inputs.partition_size, 784 / 8);
        assert_eq!(layer.outputs.partition_size, 128 / 8);
    }
}