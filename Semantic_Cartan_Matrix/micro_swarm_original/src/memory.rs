//! Memory pooling system for efficient inter-agent communication

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use serde::{Serialize, Deserialize};
use tokio::sync::{RwLock, Semaphore};
use bytes::{Bytes, BytesMut};
use uuid::Uuid;
use tracing::{info, warn, debug};

use crate::{Result, SwarmError, AgentId};

/// Memory pool configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolConfig {
    /// Total memory pool size in bytes
    pub total_size: usize,
    /// Size of each memory region
    pub region_size: usize,
    /// Maximum regions per agent
    pub max_regions_per_agent: usize,
    /// Enable memory compression
    pub compression_enabled: bool,
    /// Memory eviction policy
    pub eviction_policy: EvictionPolicy,
    /// Region lifetime before eviction
    pub region_ttl: Duration,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            total_size: 28 * 1024 * 1024, // 28MB to match chip spec
            region_size: 64 * 1024, // 64KB regions
            max_regions_per_agent: 16,
            compression_enabled: true,
            eviction_policy: EvictionPolicy::LRU,
            region_ttl: Duration::from_secs(300),
        }
    }
}

/// Memory eviction policies
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum EvictionPolicy {
    /// Least Recently Used
    LRU,
    /// Least Frequently Used
    LFU,
    /// First In First Out
    FIFO,
    /// Time-based eviction
    TTL,
}

/// Memory region metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegionMetadata {
    pub id: Uuid,
    pub owner: AgentId,
    pub size: usize,
    #[serde(skip, default = "Instant::now")]
    pub created_at: Instant,
    #[serde(skip, default = "Instant::now")]
    pub last_accessed: Instant,
    pub access_count: u64,
    pub locked: bool,
    pub compressed: bool,
}

/// Shared memory region
pub struct MemoryRegion {
    metadata: RegionMetadata,
    data: Arc<RwLock<Bytes>>,
    lock: Arc<Semaphore>,
}

impl MemoryRegion {
    /// Create a new memory region
    fn new(owner: AgentId, size: usize) -> Self {
        Self {
            metadata: RegionMetadata {
                id: Uuid::new_v4(),
                owner,
                size,
                created_at: Instant::now(),
                last_accessed: Instant::now(),
                access_count: 0,
                locked: false,
                compressed: false,
            },
            data: Arc::new(RwLock::new(Bytes::new())),
            lock: Arc::new(Semaphore::new(1)),
        }
    }
    
    /// Read data from region
    pub async fn read(&mut self) -> Result<Bytes> {
        self.metadata.last_accessed = Instant::now();
        self.metadata.access_count += 1;
        
        let data = self.data.read().await;
        Ok(data.clone())
    }
    
    /// Write data to region
    pub async fn write(&mut self, data: Bytes) -> Result<()> {
        if data.len() > self.metadata.size {
            return Err(SwarmError::Memory("Data exceeds region size".into()));
        }
        
        let _permit = self.lock.acquire().await
            .map_err(|_| SwarmError::Memory("Failed to acquire lock".into()))?;
        
        self.metadata.last_accessed = Instant::now();
        self.metadata.access_count += 1;
        
        let mut region_data = self.data.write().await;
        *region_data = data;
        
        Ok(())
    }
    
    /// Lock region for exclusive access
    pub async fn lock(&mut self) -> Result<()> {
        self.metadata.locked = true;
        Ok(())
    }
    
    /// Unlock region
    pub async fn unlock(&mut self) -> Result<()> {
        self.metadata.locked = false;
        Ok(())
    }
}

/// Memory pool statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolStats {
    pub total_regions: usize,
    pub allocated_regions: usize,
    pub free_regions: usize,
    pub total_bytes_allocated: usize,
    pub compression_ratio: f64,
    pub evictions: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
}

/// Memory pool for inter-agent communication
pub struct MemoryPool {
    config: PoolConfig,
    regions: Arc<RwLock<HashMap<Uuid, MemoryRegion>>>,
    agent_allocations: Arc<RwLock<HashMap<AgentId, Vec<Uuid>>>>,
    free_list: Arc<RwLock<Vec<Uuid>>>,
    stats: Arc<RwLock<PoolStats>>,
    allocation_semaphore: Arc<Semaphore>,
}

impl MemoryPool {
    /// Create a new memory pool
    pub fn new(config: PoolConfig) -> Self {
        let total_regions = config.total_size / config.region_size;
        let allocation_semaphore = Arc::new(Semaphore::new(total_regions));
        
        Self {
            config,
            regions: Arc::new(RwLock::new(HashMap::new())),
            agent_allocations: Arc::new(RwLock::new(HashMap::new())),
            free_list: Arc::new(RwLock::new(Vec::with_capacity(total_regions))),
            stats: Arc::new(RwLock::new(PoolStats {
                total_regions,
                allocated_regions: 0,
                free_regions: total_regions,
                total_bytes_allocated: 0,
                compression_ratio: 1.0,
                evictions: 0,
                cache_hits: 0,
                cache_misses: 0,
            })),
            allocation_semaphore,
        }
    }
    
    /// Initialize the memory pool
    pub async fn initialize(&self) -> Result<()> {
        info!("Initializing memory pool with {} regions", 
              self.config.total_size / self.config.region_size);
        
        // Pre-allocate regions for free list
        let mut free_list = self.free_list.write().await;
        for _ in 0..self.config.total_size / self.config.region_size {
            free_list.push(Uuid::new_v4());
        }
        
        Ok(())
    }
    
    /// Allocate a memory region for an agent
    pub async fn allocate(&self, agent_id: AgentId, size: usize) -> Result<Uuid> {
        // Check size
        if size > self.config.region_size {
            return Err(SwarmError::Memory("Requested size exceeds region size".into()));
        }
        
        // Check agent allocation limit
        let allocations = self.agent_allocations.read().await;
        if let Some(agent_regions) = allocations.get(&agent_id) {
            if agent_regions.len() >= self.config.max_regions_per_agent {
                return Err(SwarmError::Memory("Agent allocation limit reached".into()));
            }
        }
        drop(allocations);
        
        // Acquire allocation permit
        let _permit = self.allocation_semaphore.acquire().await
            .map_err(|_| SwarmError::Memory("No free regions available".into()))?;
        
        // Get region from free list
        let mut free_list = self.free_list.write().await;
        let region_id = free_list.pop()
            .ok_or_else(|| SwarmError::Memory("No free regions available".into()))?;
        drop(free_list);
        
        // Create region
        let region = MemoryRegion::new(agent_id, self.config.region_size);
        
        // Add to regions map
        let mut regions = self.regions.write().await;
        regions.insert(region_id, region);
        
        // Update agent allocations
        let mut allocations = self.agent_allocations.write().await;
        allocations.entry(agent_id).or_insert_with(Vec::new).push(region_id);
        
        // Update stats
        let mut stats = self.stats.write().await;
        stats.allocated_regions += 1;
        stats.free_regions -= 1;
        stats.total_bytes_allocated += size;
        
        debug!("Allocated region {} for agent {:?}", region_id, agent_id);
        Ok(region_id)
    }
    
    /// Deallocate a memory region
    pub async fn deallocate(&self, region_id: Uuid) -> Result<()> {
        // Remove from regions map
        let mut regions = self.regions.write().await;
        let region = regions.remove(&region_id)
            .ok_or_else(|| SwarmError::Memory("Region not found".into()))?;
        
        // Remove from agent allocations
        let mut allocations = self.agent_allocations.write().await;
        if let Some(agent_regions) = allocations.get_mut(&region.metadata.owner) {
            agent_regions.retain(|&id| id != region_id);
        }
        
        // Return to free list
        let mut free_list = self.free_list.write().await;
        free_list.push(region_id);
        
        // Update stats
        let mut stats = self.stats.write().await;
        stats.allocated_regions -= 1;
        stats.free_regions += 1;
        stats.total_bytes_allocated -= region.metadata.size;
        
        debug!("Deallocated region {}", region_id);
        Ok(())
    }
    
    /// Read data from a region
    pub async fn read(&self, region_id: Uuid) -> Result<Bytes> {
        let mut regions = self.regions.write().await;
        let region = regions.get_mut(&region_id)
            .ok_or_else(|| SwarmError::Memory("Region not found".into()))?;
        
        if region.metadata.locked {
            return Err(SwarmError::Memory("Region is locked".into()));
        }
        
        region.read().await
    }
    
    /// Write data to a region
    pub async fn write(&self, region_id: Uuid, data: Bytes) -> Result<()> {
        let mut regions = self.regions.write().await;
        let region = regions.get_mut(&region_id)
            .ok_or_else(|| SwarmError::Memory("Region not found".into()))?;
        
        if region.metadata.locked {
            return Err(SwarmError::Memory("Region is locked".into()));
        }
        
        region.write(data).await
    }
    
    /// Copy data between regions
    pub async fn copy(&self, src_id: Uuid, dst_id: Uuid) -> Result<()> {
        // Read from source
        let data = self.read(src_id).await?;
        
        // Write to destination
        self.write(dst_id, data).await?;
        
        Ok(())
    }
    
    /// Zero-copy transfer between agents
    pub async fn transfer(&self, region_id: Uuid, new_owner: AgentId) -> Result<()> {
        let mut regions = self.regions.write().await;
        let region = regions.get_mut(&region_id)
            .ok_or_else(|| SwarmError::Memory("Region not found".into()))?;
        
        let old_owner = region.metadata.owner;
        region.metadata.owner = new_owner;
        
        // Update allocations
        let mut allocations = self.agent_allocations.write().await;
        
        // Remove from old owner
        if let Some(old_regions) = allocations.get_mut(&old_owner) {
            old_regions.retain(|&id| id != region_id);
        }
        
        // Add to new owner
        allocations.entry(new_owner).or_insert_with(Vec::new).push(region_id);
        
        debug!("Transferred region {} from {:?} to {:?}", region_id, old_owner, new_owner);
        Ok(())
    }
    
    /// Get pool statistics
    pub async fn get_stats(&self) -> PoolStats {
        self.stats.read().await.clone()
    }
    
    /// Perform garbage collection
    pub async fn gc(&self) -> Result<u64> {
        let mut evicted = 0;
        let now = Instant::now();
        
        match self.config.eviction_policy {
            EvictionPolicy::TTL => {
                let regions = self.regions.read().await;
                let expired: Vec<_> = regions.iter()
                    .filter(|(_, region)| {
                        now.duration_since(region.metadata.created_at) > self.config.region_ttl
                    })
                    .map(|(id, _)| *id)
                    .collect();
                drop(regions);
                
                for region_id in expired {
                    self.deallocate(region_id).await?;
                    evicted += 1;
                }
            }
            _ => {
                // Other eviction policies not implemented for brevity
            }
        }
        
        let mut stats = self.stats.write().await;
        stats.evictions += evicted;
        
        info!("Garbage collection evicted {} regions", evicted);
        Ok(evicted)
    }
}

/// Memory-mapped region for large data
pub struct MappedRegion {
    id: Uuid,
    path: std::path::PathBuf,
    mmap: memmap2::MmapMut,
    metadata: RegionMetadata,
}

impl MappedRegion {
    /// Create a new memory-mapped region
    pub async fn new(
        owner: AgentId,
        size: usize,
        path: std::path::PathBuf,
    ) -> Result<Self> {
        use std::fs::OpenOptions;
        
        // Create file
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&path)
            .map_err(|e| SwarmError::Memory(format!("Failed to create mmap file: {}", e)))?;
        
        file.set_len(size as u64)
            .map_err(|e| SwarmError::Memory(format!("Failed to set file size: {}", e)))?;
        
        // Create memory map
        let mmap = unsafe {
            memmap2::MmapMut::map_mut(&file)
                .map_err(|e| SwarmError::Memory(format!("Failed to mmap: {}", e)))?
        };
        
        Ok(Self {
            id: Uuid::new_v4(),
            path,
            mmap,
            metadata: RegionMetadata {
                id: Uuid::new_v4(),
                owner,
                size,
                created_at: Instant::now(),
                last_accessed: Instant::now(),
                access_count: 0,
                locked: false,
                compressed: false,
            },
        })
    }
    
    /// Get a slice of the mapped memory
    pub fn as_slice(&self) -> &[u8] {
        &self.mmap[..]
    }
    
    /// Get a mutable slice of the mapped memory
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        &mut self.mmap[..]
    }
    
    /// Flush changes to disk
    pub fn flush(&self) -> Result<()> {
        self.mmap.flush()
            .map_err(|e| SwarmError::Memory(format!("Failed to flush mmap: {}", e)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_memory_pool() {
        let pool = MemoryPool::new(PoolConfig::default());
        pool.initialize().await.unwrap();
        
        let agent_id = AgentId::new();
        
        // Allocate region
        let region_id = pool.allocate(agent_id, 1024).await.unwrap();
        
        // Write data
        let data = Bytes::from(vec![1, 2, 3, 4]);
        pool.write(region_id, data.clone()).await.unwrap();
        
        // Read data
        let read_data = pool.read(region_id).await.unwrap();
        assert_eq!(data, read_data);
        
        // Check stats
        let stats = pool.get_stats().await;
        assert_eq!(stats.allocated_regions, 1);
        
        // Deallocate
        pool.deallocate(region_id).await.unwrap();
        
        let stats = pool.get_stats().await;
        assert_eq!(stats.allocated_regions, 0);
    }
}