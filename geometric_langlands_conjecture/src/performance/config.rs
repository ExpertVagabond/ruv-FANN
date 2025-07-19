//! Performance configuration system with persistence
//!
//! This module provides a flexible configuration system for performance
//! tuning with automatic persistence and runtime updates.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

/// Performance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// Cache configuration
    pub cache: CacheConfig,
    /// Parallel execution configuration
    pub parallel: ParallelConfig,
    /// Memory configuration
    pub memory: MemoryConfig,
    /// Algorithm-specific settings
    pub algorithms: AlgorithmConfig,
    /// GPU configuration
    pub gpu: GpuConfig,
    /// Custom settings
    pub custom: HashMap<String, serde_json::Value>,
}

/// Cache configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// Maximum cache size in MB
    pub max_size_mb: usize,
    /// Cache strategy
    pub strategy: String,
    /// TTL in seconds (0 = no TTL)
    pub ttl_seconds: u64,
    /// Enable compression
    pub compression: bool,
    /// Prefetch depth
    pub prefetch_depth: usize,
}

/// Parallel execution configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParallelConfig {
    /// Number of threads (0 = auto)
    pub num_threads: usize,
    /// Execution strategy
    pub strategy: String,
    /// Minimum chunk size
    pub min_chunk_size: usize,
    /// Enable work stealing
    pub work_stealing: bool,
    /// Thread pinning
    pub thread_pinning: bool,
}

/// Memory configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryConfig {
    /// Memory pool size in MB
    pub pool_size_mb: usize,
    /// Enable memory mapping
    pub memory_mapping: bool,
    /// Preallocate buffers
    pub preallocate: bool,
    /// Buffer alignment
    pub alignment: usize,
    /// Enable huge pages
    pub huge_pages: bool,
}

/// Algorithm-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlgorithmConfig {
    /// Matrix multiplication block size
    pub matmul_block_size: usize,
    /// FFT threshold for parallel execution
    pub fft_parallel_threshold: usize,
    /// Eigenvalue iteration limit
    pub eigenvalue_max_iterations: usize,
    /// Convergence tolerance
    pub convergence_tolerance: f64,
    /// Enable fast approximations
    pub fast_approximations: bool,
}

/// GPU configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuConfig {
    /// Enable GPU acceleration
    pub enabled: bool,
    /// Preferred device ID
    pub device_id: i32,
    /// Memory limit in MB (0 = no limit)
    pub memory_limit_mb: usize,
    /// Enable unified memory
    pub unified_memory: bool,
    /// Kernel launch configuration
    pub kernel_config: KernelConfig,
}

/// Kernel launch configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KernelConfig {
    /// Block dimensions
    pub block_size: (u32, u32, u32),
    /// Grid dimensions (0 = auto)
    pub grid_size: (u32, u32, u32),
    /// Shared memory size
    pub shared_memory: usize,
    /// Register limit
    pub max_registers: u32,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            cache: CacheConfig {
                max_size_mb: 1024, // 1GB
                strategy: "LRU".to_string(),
                ttl_seconds: 3600, // 1 hour
                compression: false,
                prefetch_depth: 2,
            },
            parallel: ParallelConfig {
                num_threads: 0, // Auto-detect
                strategy: "WorkStealing".to_string(),
                min_chunk_size: 1000,
                work_stealing: true,
                thread_pinning: false,
            },
            memory: MemoryConfig {
                pool_size_mb: 512,
                memory_mapping: true,
                preallocate: true,
                alignment: 64, // Cache line size
                huge_pages: false,
            },
            algorithms: AlgorithmConfig {
                matmul_block_size: 64,
                fft_parallel_threshold: 1024,
                eigenvalue_max_iterations: 1000,
                convergence_tolerance: 1e-10,
                fast_approximations: false,
            },
            gpu: GpuConfig {
                enabled: cfg!(feature = "cuda"),
                device_id: 0,
                memory_limit_mb: 0,
                unified_memory: true,
                kernel_config: KernelConfig {
                    block_size: (256, 1, 1),
                    grid_size: (0, 0, 0), // Auto
                    shared_memory: 49152,
                    max_registers: 64,
                },
            },
            custom: HashMap::new(),
        }
    }
}

/// Configuration manager with persistence
pub struct ConfigManager {
    /// Current configuration
    config: Arc<RwLock<PerformanceConfig>>,
    /// Configuration file path
    config_path: PathBuf,
    /// Auto-save on changes
    auto_save: bool,
}

impl ConfigManager {
    /// Create new configuration manager
    pub fn new(config_path: PathBuf) -> Self {
        let config = if config_path.exists() {
            Self::load_from_file(&config_path).unwrap_or_default()
        } else {
            PerformanceConfig::default()
        };
        
        Self {
            config: Arc::new(RwLock::new(config)),
            config_path,
            auto_save: true,
        }
    }
    
    /// Load or create default configuration
    pub fn load_or_default() -> Self {
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("geometric-langlands");
        
        fs::create_dir_all(&config_dir).ok();
        
        let config_path = config_dir.join("performance.toml");
        Self::new(config_path)
    }
    
    /// Load configuration from file
    fn load_from_file(path: &Path) -> Result<PerformanceConfig, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let config = toml::from_str(&content)?;
        Ok(config)
    }
    
    /// Save configuration to file
    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let config = self.config.read().unwrap();
        let content = toml::to_string_pretty(&*config)?;
        fs::write(&self.config_path, content)?;
        Ok(())
    }
    
    /// Get cache size
    pub fn get_cache_size(&self) -> usize {
        self.config.read().unwrap().cache.max_size_mb * 1024 * 1024
    }
    
    /// Get thread count
    pub fn get_thread_count(&self) -> usize {
        let threads = self.config.read().unwrap().parallel.num_threads;
        if threads == 0 {
            num_cpus::get()
        } else {
            threads
        }
    }
    
    /// Get block size for matrix operations
    pub fn get_block_size(&self) -> usize {
        self.config.read().unwrap().algorithms.matmul_block_size
    }
    
    /// Update configuration
    pub fn update<F>(&self, updater: F) -> Result<(), Box<dyn std::error::Error>>
    where
        F: FnOnce(&mut PerformanceConfig),
    {
        {
            let mut config = self.config.write().unwrap();
            updater(&mut config);
        }
        
        if self.auto_save {
            self.save()?;
        }
        
        Ok(())
    }
    
    /// Get custom setting
    pub fn get_custom<T: serde::de::DeserializeOwned>(&self, key: &str) -> Option<T> {
        let config = self.config.read().unwrap();
        config.custom.get(key)
            .and_then(|v| serde_json::from_value(v.clone()).ok())
    }
    
    /// Set custom setting
    pub fn set_custom<T: Serialize>(&self, key: &str, value: T) -> Result<(), Box<dyn std::error::Error>> {
        self.update(|config| {
            config.custom.insert(
                key.to_string(),
                serde_json::to_value(value).unwrap(),
            );
        })
    }
    
    /// Create optimized configuration for specific workload
    pub fn optimize_for_workload(&self, workload: WorkloadType) -> Result<(), Box<dyn std::error::Error>> {
        self.update(|config| {
            match workload {
                WorkloadType::LargeMatrix => {
                    config.cache.max_size_mb = 2048;
                    config.algorithms.matmul_block_size = 128;
                    config.parallel.min_chunk_size = 10000;
                    config.memory.pool_size_mb = 1024;
                }
                WorkloadType::ManySmallComputations => {
                    config.cache.max_size_mb = 512;
                    config.algorithms.matmul_block_size = 32;
                    config.parallel.min_chunk_size = 100;
                    config.parallel.work_stealing = true;
                }
                WorkloadType::RealTime => {
                    config.cache.ttl_seconds = 60;
                    config.algorithms.fast_approximations = true;
                    config.parallel.thread_pinning = true;
                }
                WorkloadType::MemoryConstrained => {
                    config.cache.max_size_mb = 256;
                    config.cache.compression = true;
                    config.memory.pool_size_mb = 256;
                    config.memory.preallocate = false;
                }
            }
        })
    }
}

/// Workload types for optimization
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkloadType {
    /// Large matrix operations
    LargeMatrix,
    /// Many small computations
    ManySmallComputations,
    /// Real-time processing
    RealTime,
    /// Memory-constrained environment
    MemoryConstrained,
}

/// Configuration presets
pub mod presets {
    use super::*;
    
    /// High-performance preset
    pub fn high_performance() -> PerformanceConfig {
        let mut config = PerformanceConfig::default();
        config.cache.max_size_mb = 4096;
        config.parallel.work_stealing = true;
        config.memory.huge_pages = true;
        config.gpu.enabled = true;
        config
    }
    
    /// Balanced preset
    pub fn balanced() -> PerformanceConfig {
        PerformanceConfig::default()
    }
    
    /// Memory-efficient preset
    pub fn memory_efficient() -> PerformanceConfig {
        let mut config = PerformanceConfig::default();
        config.cache.max_size_mb = 256;
        config.cache.compression = true;
        config.memory.pool_size_mb = 256;
        config.memory.preallocate = false;
        config
    }
    
    /// CPU-only preset
    pub fn cpu_only() -> PerformanceConfig {
        let mut config = PerformanceConfig::default();
        config.gpu.enabled = false;
        config.parallel.num_threads = num_cpus::get();
        config
    }
}

/// Environment variable overrides
impl ConfigManager {
    /// Apply environment variable overrides
    pub fn apply_env_overrides(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.update(|config| {
            // Cache size
            if let Ok(size) = std::env::var("LANGLANDS_CACHE_SIZE_MB") {
                if let Ok(size) = size.parse() {
                    config.cache.max_size_mb = size;
                }
            }
            
            // Thread count
            if let Ok(threads) = std::env::var("LANGLANDS_NUM_THREADS") {
                if let Ok(threads) = threads.parse() {
                    config.parallel.num_threads = threads;
                }
            }
            
            // GPU enable
            if let Ok(gpu) = std::env::var("LANGLANDS_ENABLE_GPU") {
                config.gpu.enabled = gpu.to_lowercase() == "true" || gpu == "1";
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    
    #[test]
    fn test_config_default() {
        let config = PerformanceConfig::default();
        assert_eq!(config.cache.max_size_mb, 1024);
        assert_eq!(config.parallel.num_threads, 0);
    }
    
    #[test]
    fn test_config_save_load() {
        let dir = tempdir().unwrap();
        let config_path = dir.path().join("test_config.toml");
        
        let manager = ConfigManager::new(config_path.clone());
        manager.update(|config| {
            config.cache.max_size_mb = 2048;
        }).unwrap();
        
        // Load again
        let manager2 = ConfigManager::new(config_path);
        assert_eq!(manager2.get_cache_size(), 2048 * 1024 * 1024);
    }
    
    #[test]
    fn test_custom_settings() {
        let dir = tempdir().unwrap();
        let manager = ConfigManager::new(dir.path().join("test.toml"));
        
        manager.set_custom("test_value", 42).unwrap();
        let value: i32 = manager.get_custom("test_value").unwrap();
        assert_eq!(value, 42);
    }
}