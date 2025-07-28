use wasmtime::{Engine, Memory, MemoryType, Store, SharedMemory, AsContextMut};
use anyhow::{Result, Context};
use std::sync::Arc;

/// Shared memory configuration for the Neuro-Synaptic chip
pub struct SharedMemoryConfig {
    /// Total memory size in bytes (28MB)
    pub total_size: usize,
    /// Memory pages (64KB each, 448 pages for 28MB)
    pub pages: u32,
    /// Enable shared memory between instances
    pub shared: bool,
}

impl Default for SharedMemoryConfig {
    fn default() -> Self {
        SharedMemoryConfig {
            total_size: 28 * 1024 * 1024, // 28MB
            pages: 448, // 28MB / 64KB = 448 pages
            shared: true,
        }
    }
}

/// Shared memory manager for WASM instances
pub struct SharedMemoryManager {
    engine: Arc<Engine>,
    config: SharedMemoryConfig,
    shared_memory: Option<Arc<SharedMemory>>,
}

impl SharedMemoryManager {
    /// Create a new shared memory manager
    pub fn new(engine: Arc<Engine>, config: SharedMemoryConfig) -> Self {
        SharedMemoryManager {
            engine,
            config,
            shared_memory: None,
        }
    }
    
    /// Initialize shared memory
    pub fn initialize(&mut self) -> Result<()> {
        if self.shared_memory.is_some() {
            return Ok(());
        }
        
        // Create memory type with shared flag
        let memory_type = MemoryType::new(self.config.pages, Some(self.config.pages));
        let memory_type = if self.config.shared {
            memory_type.shared(true)
        } else {
            memory_type
        };
        
        // Create shared memory
        let shared_mem = SharedMemory::new(&self.engine, memory_type)
            .context("Failed to create shared memory")?;
        
        self.shared_memory = Some(Arc::new(shared_mem));
        
        log::info!(
            "Initialized shared memory: {} pages ({} MB)",
            self.config.pages,
            self.config.total_size / (1024 * 1024)
        );
        
        Ok(())
    }
    
    /// Get shared memory instance
    pub fn get_shared_memory(&self) -> Result<Arc<SharedMemory>> {
        self.shared_memory
            .clone()
            .context("Shared memory not initialized")
    }
    
    /// Create a memory instance from shared memory
    pub fn create_memory_instance<T>(&self, store: &mut Store<T>) -> Result<Memory> {
        let shared_mem = self.get_shared_memory()?;
        Memory::new(store, shared_mem.ty())
            .context("Failed to create memory instance")
    }
    
    /// Get memory statistics
    pub fn get_stats(&self) -> MemoryStats {
        MemoryStats {
            total_pages: self.config.pages,
            total_bytes: self.config.total_size,
            page_size: 65536, // 64KB
            is_shared: self.config.shared,
        }
    }
}

/// Memory statistics
#[derive(Debug, Clone)]
pub struct MemoryStats {
    pub total_pages: u32,
    pub total_bytes: usize,
    pub page_size: usize,
    pub is_shared: bool,
}

/// Memory region for core allocation
#[derive(Debug, Clone)]
pub struct MemoryRegion {
    pub core_id: u32,
    pub start_offset: usize,
    pub size: usize,
}

impl MemoryRegion {
    /// Create memory regions for 256 cores
    pub fn create_core_regions(total_memory: usize, num_cores: u32) -> Vec<MemoryRegion> {
        let memory_per_core = total_memory / num_cores as usize;
        
        (0..num_cores)
            .map(|core_id| MemoryRegion {
                core_id,
                start_offset: (core_id as usize) * memory_per_core,
                size: memory_per_core,
            })
            .collect()
    }
    
    /// Get the memory region for a specific core
    pub fn for_core(core_id: u32, regions: &[MemoryRegion]) -> Option<&MemoryRegion> {
        regions.iter().find(|r| r.core_id == core_id)
    }
}

/// Memory access utilities
pub mod access {
    use wasmtime::{Memory, AsContextMut};
    use anyhow::{Result, Context};
    
    /// Read data from memory
    pub fn read_u32(memory: &Memory, mut store: impl AsContextMut, offset: usize) -> Result<u32> {
        let mut buffer = [0u8; 4];
        memory.read(&mut store, offset, &mut buffer)
            .context("Failed to read from memory")?;
        Ok(u32::from_le_bytes(buffer))
    }
    
    /// Write data to memory
    pub fn write_u32(memory: &Memory, mut store: impl AsContextMut, offset: usize, value: u32) -> Result<()> {
        let bytes = value.to_le_bytes();
        memory.write(&mut store, offset, &bytes)
            .context("Failed to write to memory")?;
        Ok(())
    }
    
    /// Read array from memory
    pub fn read_array(
        memory: &Memory,
        mut store: impl AsContextMut,
        offset: usize,
        length: usize,
    ) -> Result<Vec<u8>> {
        let mut buffer = vec![0u8; length];
        memory.read(&mut store, offset, &mut buffer)
            .context("Failed to read array from memory")?;
        Ok(buffer)
    }
    
    /// Write array to memory
    pub fn write_array(
        memory: &Memory,
        mut store: impl AsContextMut,
        offset: usize,
        data: &[u8],
    ) -> Result<()> {
        memory.write(&mut store, offset, data)
            .context("Failed to write array to memory")?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasmtime::Config;
    
    #[test]
    fn test_memory_config_default() {
        let config = SharedMemoryConfig::default();
        assert_eq!(config.total_size, 28 * 1024 * 1024);
        assert_eq!(config.pages, 448);
        assert!(config.shared);
    }
    
    #[test]
    fn test_memory_regions() {
        let total_memory = 28 * 1024 * 1024; // 28MB
        let regions = MemoryRegion::create_core_regions(total_memory, 256);
        
        assert_eq!(regions.len(), 256);
        
        // Check first and last regions
        assert_eq!(regions[0].core_id, 0);
        assert_eq!(regions[0].start_offset, 0);
        assert_eq!(regions[255].core_id, 255);
        
        // Check memory per core (should be ~112KB)
        let expected_size = total_memory / 256;
        assert_eq!(regions[0].size, expected_size);
    }
    
    #[test]
    fn test_shared_memory_manager() {
        let config = Config::new();
        let engine = Arc::new(Engine::new(&config).unwrap());
        let mem_config = SharedMemoryConfig::default();
        
        let mut manager = SharedMemoryManager::new(engine, mem_config);
        assert!(manager.initialize().is_ok());
        
        let stats = manager.get_stats();
        assert_eq!(stats.total_pages, 448);
        assert_eq!(stats.page_size, 65536);
        assert!(stats.is_shared);
    }
}