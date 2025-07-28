//! Core scheduler for managing 256-core parallel execution
//! 
//! This module provides the core scheduling infrastructure for distributing
//! neural network computations across multiple cores using Rayon thread pools.

use rayon::prelude::*;
use rayon::{ThreadPool, ThreadPoolBuilder};
use std::sync::{Arc, Mutex, atomic::{AtomicUsize, Ordering}};
use crossbeam::channel::{bounded, Sender, Receiver};
use serde::{Serialize, Deserialize};
use crate::memory::shared::{SharedMemory, LayerMemory, PartitionStrategy};

/// Work distribution strategy for the scheduler
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DistributionStrategy {
    /// Process single model across all cores
    SingleModel,
    /// Process batch of models, one per core group
    Batch { batch_size: usize },
    /// Dynamic load balancing
    Dynamic,
    /// Round-robin distribution
    RoundRobin,
}

/// Synchronization mode for layer coordination
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum SyncMode {
    /// Barrier synchronization after each layer
    Barrier,
    /// Asynchronous with dependency tracking
    Async,
    /// Pipeline parallel with overlapping layers
    Pipeline,
}

/// Core scheduler configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulerConfig {
    /// Number of physical cores to use
    pub num_cores: usize,
    /// Work distribution strategy
    pub distribution: DistributionStrategy,
    /// Synchronization mode
    pub sync_mode: SyncMode,
    /// Thread pool stack size (in MB)
    pub stack_size: usize,
    /// Enable work stealing
    pub work_stealing: bool,
}

impl Default for SchedulerConfig {
    fn default() -> Self {
        Self {
            num_cores: 256,
            distribution: DistributionStrategy::SingleModel,
            sync_mode: SyncMode::Barrier,
            stack_size: 8,
            work_stealing: true,
        }
    }
}

/// Work unit for core execution
#[derive(Debug)]
pub struct WorkUnit {
    /// Unique work ID
    pub id: usize,
    /// Core ID assigned to this work
    pub core_id: usize,
    /// Layer index
    pub layer_idx: usize,
    /// Start index in the data
    pub start: usize,
    /// End index in the data
    pub end: usize,
}

/// Core scheduler for managing parallel execution
pub struct CoreScheduler {
    /// Rayon thread pool
    thread_pool: ThreadPool,
    /// Configuration
    config: SchedulerConfig,
    /// Work queue sender
    work_sender: Sender<WorkUnit>,
    /// Work queue receiver
    work_receiver: Receiver<WorkUnit>,
    /// Active work counter
    active_work: Arc<AtomicUsize>,
    /// Total work completed
    completed_work: Arc<AtomicUsize>,
}

impl CoreScheduler {
    /// Create new scheduler with configuration
    pub fn new(config: SchedulerConfig) -> Result<Self, rayon::ThreadPoolBuildError> {
        let thread_pool = ThreadPoolBuilder::new()
            .num_threads(config.num_cores)
            .stack_size(config.stack_size * 1024 * 1024)
            .thread_name(|idx| format!("core-{:03}", idx))
            .build()?;
        
        // Create work queue with capacity for 2x cores
        let (work_sender, work_receiver) = bounded(config.num_cores * 2);
        
        Ok(Self {
            thread_pool,
            config,
            work_sender,
            work_receiver,
            active_work: Arc::new(AtomicUsize::new(0)),
            completed_work: Arc::new(AtomicUsize::new(0)),
        })
    }
    
    /// Execute work across cores for a single layer
    pub fn execute_layer<F>(
        &self,
        layer_memory: &LayerMemory,
        layer_idx: usize,
        compute_fn: F,
    ) where
        F: Fn(&WorkUnit, &LayerMemory) + Send + Sync,
    {
        let compute_fn = Arc::new(compute_fn);
        let work_units = self.create_work_units(layer_idx, layer_memory.inputs.num_cores());
        
        match self.config.sync_mode {
            SyncMode::Barrier => {
                self.execute_with_barrier(work_units, layer_memory, compute_fn);
            },
            SyncMode::Async => {
                self.execute_async(work_units, layer_memory, compute_fn);
            },
            SyncMode::Pipeline => {
                self.execute_pipeline(work_units, layer_memory, compute_fn);
            },
        }
    }
    
    /// Create work units based on distribution strategy
    fn create_work_units(&self, layer_idx: usize, num_partitions: usize) -> Vec<WorkUnit> {
        match self.config.distribution {
            DistributionStrategy::SingleModel => {
                // Distribute single model across all cores
                (0..self.config.num_cores)
                    .map(|core_id| {
                        let partition_size = num_partitions / self.config.num_cores;
                        let start = core_id * partition_size;
                        let end = if core_id == self.config.num_cores - 1 {
                            num_partitions
                        } else {
                            (core_id + 1) * partition_size
                        };
                        
                        WorkUnit {
                            id: core_id + layer_idx * self.config.num_cores,
                            core_id,
                            layer_idx,
                            start,
                            end,
                        }
                    })
                    .collect()
            },
            DistributionStrategy::Batch { batch_size } => {
                // Each core group processes one model from batch
                let cores_per_model = self.config.num_cores / batch_size;
                (0..self.config.num_cores)
                    .map(|core_id| {
                        let model_idx = core_id / cores_per_model;
                        let local_core_id = core_id % cores_per_model;
                        let partition_size = num_partitions / cores_per_model;
                        let start = local_core_id * partition_size;
                        let end = if local_core_id == cores_per_model - 1 {
                            num_partitions
                        } else {
                            (local_core_id + 1) * partition_size
                        };
                        
                        WorkUnit {
                            id: core_id + layer_idx * self.config.num_cores,
                            core_id,
                            layer_idx,
                            start: start + model_idx * num_partitions,
                            end: end + model_idx * num_partitions,
                        }
                    })
                    .collect()
            },
            DistributionStrategy::Dynamic => {
                // Dynamic work stealing handled by Rayon
                self.create_dynamic_units(layer_idx, num_partitions)
            },
            DistributionStrategy::RoundRobin => {
                // Round-robin distribution
                (0..num_partitions)
                    .map(|i| WorkUnit {
                        id: i,
                        core_id: i % self.config.num_cores,
                        layer_idx,
                        start: i,
                        end: i + 1,
                    })
                    .collect()
            },
        }
    }
    
    /// Create dynamic work units for work stealing
    fn create_dynamic_units(&self, layer_idx: usize, num_partitions: usize) -> Vec<WorkUnit> {
        // Create smaller chunks for better load balancing
        let chunk_size = (num_partitions / (self.config.num_cores * 4)).max(1);
        let mut units = Vec::new();
        let mut start = 0;
        let mut id = 0;
        
        while start < num_partitions {
            let end = (start + chunk_size).min(num_partitions);
            units.push(WorkUnit {
                id,
                core_id: id % self.config.num_cores,
                layer_idx,
                start,
                end,
            });
            start = end;
            id += 1;
        }
        
        units
    }
    
    /// Execute with barrier synchronization
    fn execute_with_barrier<F>(
        &self,
        work_units: Vec<WorkUnit>,
        layer_memory: &LayerMemory,
        compute_fn: Arc<F>,
    ) where
        F: Fn(&WorkUnit, &LayerMemory) + Send + Sync,
    {
        let barrier = layer_memory.inputs.barrier();
        
        self.thread_pool.install(|| {
            work_units.par_iter().for_each(|unit| {
                // Update active work counter
                self.active_work.fetch_add(1, Ordering::Relaxed);
                
                // Execute computation
                compute_fn(unit, layer_memory);
                
                // Update counters
                self.active_work.fetch_sub(1, Ordering::Relaxed);
                self.completed_work.fetch_add(1, Ordering::Relaxed);
                
                // Synchronize at barrier
                barrier.wait();
            });
        });
    }
    
    /// Execute asynchronously with dependency tracking
    fn execute_async<F>(
        &self,
        work_units: Vec<WorkUnit>,
        layer_memory: &LayerMemory,
        compute_fn: Arc<F>,
    ) where
        F: Fn(&WorkUnit, &LayerMemory) + Send + Sync,
    {
        // Queue all work units
        for unit in work_units {
            self.work_sender.send(unit).unwrap();
        }
        
        // Process asynchronously
        self.thread_pool.install(|| {
            rayon::scope(|s| {
                for _ in 0..self.config.num_cores {
                    let receiver = self.work_receiver.clone();
                    let compute_fn = Arc::clone(&compute_fn);
                    let active = Arc::clone(&self.active_work);
                    let completed = Arc::clone(&self.completed_work);
                    
                    s.spawn(move |_| {
                        while let Ok(unit) = receiver.recv() {
                            active.fetch_add(1, Ordering::Relaxed);
                            compute_fn(&unit, layer_memory);
                            active.fetch_sub(1, Ordering::Relaxed);
                            completed.fetch_add(1, Ordering::Relaxed);
                        }
                    });
                }
            });
        });
    }
    
    /// Execute in pipeline mode with overlapping layers
    fn execute_pipeline<F>(
        &self,
        work_units: Vec<WorkUnit>,
        layer_memory: &LayerMemory,
        compute_fn: Arc<F>,
    ) where
        F: Fn(&WorkUnit, &LayerMemory) + Send + Sync,
    {
        // Pipeline execution allows overlapping of consecutive layers
        // This is more complex and would require additional state management
        // For now, fall back to barrier mode
        self.execute_with_barrier(work_units, layer_memory, compute_fn);
    }
    
    /// Get current statistics
    pub fn stats(&self) -> SchedulerStats {
        SchedulerStats {
            active_work: self.active_work.load(Ordering::Relaxed),
            completed_work: self.completed_work.load(Ordering::Relaxed),
            num_cores: self.config.num_cores,
            distribution: self.config.distribution,
        }
    }
    
    /// Shutdown the scheduler
    pub fn shutdown(self) {
        // Drop work sender to signal shutdown
        drop(self.work_sender);
        // Thread pool will be dropped automatically
    }
}

/// Scheduler statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulerStats {
    pub active_work: usize,
    pub completed_work: usize,
    pub num_cores: usize,
    pub distribution: DistributionStrategy,
}

/// Helper functions for common computation patterns
pub mod patterns {
    use super::*;
    
    /// Matrix multiplication pattern
    pub fn matrix_multiply(
        scheduler: &CoreScheduler,
        a: &LayerMemory,
        b: &LayerMemory,
        c: &mut LayerMemory,
    ) {
        scheduler.execute_layer(a, 0, |unit, _| {
            // Partition-specific matrix multiplication
            // This would contain the actual computation logic
        });
    }
    
    /// Convolution pattern
    pub fn convolution(
        scheduler: &CoreScheduler,
        input: &LayerMemory,
        kernel: &LayerMemory,
        output: &mut LayerMemory,
    ) {
        scheduler.execute_layer(input, 0, |unit, _| {
            // Partition-specific convolution
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_scheduler_creation() {
        let config = SchedulerConfig {
            num_cores: 4,
            ..Default::default()
        };
        
        let scheduler = CoreScheduler::new(config).unwrap();
        let stats = scheduler.stats();
        
        assert_eq!(stats.num_cores, 4);
        assert_eq!(stats.active_work, 0);
        assert_eq!(stats.completed_work, 0);
    }
    
    #[test]
    fn test_work_unit_creation() {
        let config = SchedulerConfig {
            num_cores: 4,
            distribution: DistributionStrategy::SingleModel,
            ..Default::default()
        };
        
        let scheduler = CoreScheduler::new(config).unwrap();
        let units = scheduler.create_work_units(0, 100);
        
        assert_eq!(units.len(), 4);
        assert_eq!(units[0].start, 0);
        assert_eq!(units[0].end, 25);
        assert_eq!(units[3].start, 75);
        assert_eq!(units[3].end, 100);
    }
    
    #[test]
    fn test_batch_distribution() {
        let config = SchedulerConfig {
            num_cores: 8,
            distribution: DistributionStrategy::Batch { batch_size: 4 },
            ..Default::default()
        };
        
        let scheduler = CoreScheduler::new(config).unwrap();
        let units = scheduler.create_work_units(0, 100);
        
        assert_eq!(units.len(), 8);
        // Each model gets 2 cores
        assert_eq!(units[0].core_id, 0);
        assert_eq!(units[1].core_id, 1);
    }
}