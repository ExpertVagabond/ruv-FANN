use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant};
use anyhow::{Result, Context};
use crossbeam::channel::{bounded, Sender, Receiver};
use parking_lot::Mutex as ParkingMutex;

use super::engine::{WasmEngine, WasmInstance};
use super::memory::SharedMemoryManager;
use wasmtime::{Module, Memory};

/// Instance state
#[derive(Debug, Clone, PartialEq)]
pub enum InstanceState {
    Idle,
    Running,
    Paused,
    Terminated,
}

/// Instance metadata
#[derive(Debug, Clone)]
pub struct InstanceMetadata {
    pub id: u32,
    pub core_id: u32,
    pub state: InstanceState,
    pub created_at: Instant,
    pub last_active: Instant,
    pub fuel_consumed: u64,
    pub executions: u64,
}

/// Instance pool for managing WASM instances
pub struct InstancePool {
    engine: Arc<WasmEngine>,
    memory_manager: Arc<SharedMemoryManager>,
    instances: Arc<RwLock<HashMap<u32, Arc<ParkingMutex<ManagedInstance>>>>>,
    metadata: Arc<RwLock<HashMap<u32, InstanceMetadata>>>,
    available_ids: Arc<Mutex<Vec<u32>>>,
    max_instances: u32,
}

/// Managed WASM instance
struct ManagedInstance {
    instance: Option<WasmInstance>,
    module: Module,
    shared_memory: Arc<Memory>,
}

impl InstancePool {
    /// Create a new instance pool
    pub fn new(
        engine: Arc<WasmEngine>,
        memory_manager: Arc<SharedMemoryManager>,
        max_instances: u32,
    ) -> Self {
        // Initialize available IDs
        let available_ids: Vec<u32> = (0..max_instances).collect();
        
        InstancePool {
            engine,
            memory_manager,
            instances: Arc::new(RwLock::new(HashMap::new())),
            metadata: Arc::new(RwLock::new(HashMap::new())),
            available_ids: Arc::new(Mutex::new(available_ids)),
            max_instances,
        }
    }
    
    /// Acquire an instance for a core
    pub fn acquire_instance(
        &self,
        core_id: u32,
        module: &Module,
        fuel_limit: u64,
    ) -> Result<u32> {
        // Get available ID
        let instance_id = {
            let mut ids = self.available_ids.lock().unwrap();
            ids.pop().context("No available instances")?
        };
        
        // Get shared memory
        let shared_memory = self.memory_manager.get_shared_memory()?;
        
        // Create WASM instance
        let wasm_instance = self.engine.create_instance(
            module,
            shared_memory.clone(),
            fuel_limit,
        )?;
        
        // Create managed instance
        let managed = ManagedInstance {
            instance: Some(wasm_instance),
            module: module.clone(),
            shared_memory,
        };
        
        // Store instance
        {
            let mut instances = self.instances.write().unwrap();
            instances.insert(instance_id, Arc::new(ParkingMutex::new(managed)));
        }
        
        // Create metadata
        let metadata = InstanceMetadata {
            id: instance_id,
            core_id,
            state: InstanceState::Idle,
            created_at: Instant::now(),
            last_active: Instant::now(),
            fuel_consumed: 0,
            executions: 0,
        };
        
        // Store metadata
        {
            let mut meta = self.metadata.write().unwrap();
            meta.insert(instance_id, metadata);
        }
        
        log::debug!("Acquired instance {} for core {}", instance_id, core_id);
        
        Ok(instance_id)
    }
    
    /// Release an instance
    pub fn release_instance(&self, instance_id: u32) -> Result<()> {
        // Remove instance
        {
            let mut instances = self.instances.write().unwrap();
            instances.remove(&instance_id)
                .context("Instance not found")?;
        }
        
        // Remove metadata
        {
            let mut meta = self.metadata.write().unwrap();
            meta.remove(&instance_id);
        }
        
        // Return ID to pool
        {
            let mut ids = self.available_ids.lock().unwrap();
            ids.push(instance_id);
        }
        
        log::debug!("Released instance {}", instance_id);
        
        Ok(())
    }
    
    /// Execute function on instance
    pub fn execute_on_instance(
        &self,
        instance_id: u32,
        func_name: &str,
        args: &[wasmtime::Val],
    ) -> Result<Vec<wasmtime::Val>> {
        // Get instance
        let instance_arc = {
            let instances = self.instances.read().unwrap();
            instances.get(&instance_id)
                .context("Instance not found")?
                .clone()
        };
        
        // Execute function
        let mut managed = instance_arc.lock();
        let wasm_instance = managed.instance.as_mut()
            .context("Instance is not active")?;
        
        // Update state
        {
            let mut meta = self.metadata.write().unwrap();
            if let Some(metadata) = meta.get_mut(&instance_id) {
                metadata.state = InstanceState::Running;
                metadata.last_active = Instant::now();
            }
        }
        
        // Execute
        let start = Instant::now();
        let results = wasm_instance.execute_function(func_name, args)?;
        let execution_time = start.elapsed();
        
        // Update metadata
        {
            let mut meta = self.metadata.write().unwrap();
            if let Some(metadata) = meta.get_mut(&instance_id) {
                metadata.state = InstanceState::Idle;
                metadata.fuel_consumed = wasm_instance.fuel_consumed();
                metadata.executions += 1;
            }
        }
        
        log::debug!(
            "Instance {} executed '{}' in {:?}",
            instance_id,
            func_name,
            execution_time
        );
        
        Ok(results)
    }
    
    /// Get instance state
    pub fn get_instance_state(&self, instance_id: u32) -> Option<InstanceState> {
        let meta = self.metadata.read().unwrap();
        meta.get(&instance_id).map(|m| m.state.clone())
    }
    
    /// Get all instance metadata
    pub fn get_all_metadata(&self) -> Vec<InstanceMetadata> {
        let meta = self.metadata.read().unwrap();
        meta.values().cloned().collect()
    }
    
    /// Get active instance count
    pub fn active_count(&self) -> usize {
        let meta = self.metadata.read().unwrap();
        meta.values()
            .filter(|m| m.state != InstanceState::Terminated)
            .count()
    }
    
    /// Pause an instance
    pub fn pause_instance(&self, instance_id: u32) -> Result<()> {
        let mut meta = self.metadata.write().unwrap();
        let metadata = meta.get_mut(&instance_id)
            .context("Instance not found")?;
        
        if metadata.state == InstanceState::Running {
            metadata.state = InstanceState::Paused;
        }
        
        Ok(())
    }
    
    /// Resume an instance
    pub fn resume_instance(&self, instance_id: u32) -> Result<()> {
        let mut meta = self.metadata.write().unwrap();
        let metadata = meta.get_mut(&instance_id)
            .context("Instance not found")?;
        
        if metadata.state == InstanceState::Paused {
            metadata.state = InstanceState::Idle;
        }
        
        Ok(())
    }
}

/// Instance scheduler for managing execution across cores
pub struct InstanceScheduler {
    pool: Arc<InstancePool>,
    work_queue: Arc<Mutex<Vec<WorkItem>>>,
    worker_handles: Vec<std::thread::JoinHandle<()>>,
    shutdown_tx: Sender<()>,
    shutdown_rx: Receiver<()>,
}

/// Work item for scheduler
#[derive(Clone)]
struct WorkItem {
    instance_id: u32,
    func_name: String,
    args: Vec<wasmtime::Val>,
    result_tx: Sender<Result<Vec<wasmtime::Val>>>,
}

impl InstanceScheduler {
    /// Create a new scheduler with worker threads
    pub fn new(pool: Arc<InstancePool>, num_workers: usize) -> Self {
        let (shutdown_tx, shutdown_rx) = bounded(1);
        let work_queue = Arc::new(Mutex::new(Vec::new()));
        let mut worker_handles = Vec::new();
        
        // Spawn worker threads
        for worker_id in 0..num_workers {
            let pool = pool.clone();
            let queue = work_queue.clone();
            let shutdown = shutdown_rx.clone();
            
            let handle = std::thread::spawn(move || {
                Self::worker_loop(worker_id, pool, queue, shutdown);
            });
            
            worker_handles.push(handle);
        }
        
        InstanceScheduler {
            pool,
            work_queue,
            worker_handles,
            shutdown_tx,
            shutdown_rx,
        }
    }
    
    /// Schedule work on an instance
    pub fn schedule(
        &self,
        instance_id: u32,
        func_name: &str,
        args: Vec<wasmtime::Val>,
    ) -> Receiver<Result<Vec<wasmtime::Val>>> {
        let (result_tx, result_rx) = bounded(1);
        
        let work_item = WorkItem {
            instance_id,
            func_name: func_name.to_string(),
            args,
            result_tx,
        };
        
        let mut queue = self.work_queue.lock().unwrap();
        queue.push(work_item);
        
        result_rx
    }
    
    /// Worker loop for processing work items
    fn worker_loop(
        worker_id: usize,
        pool: Arc<InstancePool>,
        work_queue: Arc<Mutex<Vec<WorkItem>>>,
        shutdown: Receiver<()>,
    ) {
        log::debug!("Worker {} started", worker_id);
        
        loop {
            // Check for shutdown
            if shutdown.try_recv().is_ok() {
                log::debug!("Worker {} shutting down", worker_id);
                break;
            }
            
            // Get work item
            let work_item = {
                let mut queue = work_queue.lock().unwrap();
                queue.pop()
            };
            
            if let Some(work) = work_item {
                // Execute work
                let result = pool.execute_on_instance(
                    work.instance_id,
                    &work.func_name,
                    &work.args,
                );
                
                // Send result
                let _ = work.result_tx.send(result);
            } else {
                // No work, sleep briefly
                std::thread::sleep(Duration::from_millis(1));
            }
        }
    }
    
    /// Shutdown the scheduler
    pub fn shutdown(self) {
        // Send shutdown signal
        let _ = self.shutdown_tx.send(());
        
        // Wait for workers
        for handle in self.worker_handles {
            let _ = handle.join();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasmtime::Config;
    
    #[test]
    fn test_instance_pool_creation() {
        let config = Config::new();
        let engine = Arc::new(WasmEngine::new().unwrap());
        let memory_manager = Arc::new(SharedMemoryManager::new(
            Arc::new(wasmtime::Engine::new(&config).unwrap()),
            Default::default(),
        ));
        
        let pool = InstancePool::new(engine, memory_manager, 256);
        assert_eq!(pool.active_count(), 0);
        assert_eq!(pool.max_instances, 256);
    }
    
    #[test]
    fn test_instance_metadata() {
        let metadata = InstanceMetadata {
            id: 1,
            core_id: 0,
            state: InstanceState::Idle,
            created_at: Instant::now(),
            last_active: Instant::now(),
            fuel_consumed: 0,
            executions: 0,
        };
        
        assert_eq!(metadata.id, 1);
        assert_eq!(metadata.core_id, 0);
        assert_eq!(metadata.state, InstanceState::Idle);
    }
}