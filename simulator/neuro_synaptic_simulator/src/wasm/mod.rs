pub mod engine;
pub mod memory;
pub mod instance_manager;
pub mod loader;

pub use engine::{WasmEngine, WasmInstance};
pub use memory::{SharedMemoryConfig, SharedMemoryManager, MemoryRegion, MemoryStats};
pub use instance_manager::{InstancePool, InstanceScheduler, InstanceState, InstanceMetadata};
pub use loader::{ModuleLoader, ModuleInfo, ModuleValidator, CacheStats};

/// WASM module exports required for neuro-synaptic cores
pub const REQUIRED_EXPORTS: &[&str] = &[
    "process_spike",
    "update_weights", 
    "get_output",
    "reset_state",
];

/// Default fuel limit per execution (instruction count)
pub const DEFAULT_FUEL_LIMIT: u64 = 1_000_000;

/// Maximum concurrent instances (one per core)
pub const MAX_INSTANCES: u32 = 256;

/// Shared memory size in MB
pub const SHARED_MEMORY_MB: usize = 28;

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_constants() {
        assert_eq!(MAX_INSTANCES, 256);
        assert_eq!(SHARED_MEMORY_MB, 28);
        assert_eq!(DEFAULT_FUEL_LIMIT, 1_000_000);
        assert_eq!(REQUIRED_EXPORTS.len(), 4);
    }
}