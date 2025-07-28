/// Timing model for cycle-accurate simulation
/// 
/// Provides timing models for various operations including
/// memory access latencies, instruction execution times, and
/// inter-core communication delays.

pub mod power;
pub mod model;

use serde::{Serialize, Deserialize};
use std::collections::HashMap;

// Re-export power module types
pub use power::{PowerModel, PowerState, EnergyConsumption};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimingModel {
    /// Clock frequency in MHz
    pub clock_frequency_mhz: u32,
    
    /// Memory access latency in cycles
    pub memory_latency_cycles: u32,
    
    /// Cache hit latency in cycles
    pub cache_hit_cycles: u32,
    
    /// Inter-core communication latency in cycles
    pub inter_core_latency_cycles: u32,
}

impl Default for TimingModel {
    fn default() -> Self {
        TimingModel {
            clock_frequency_mhz: 1000, // 1GHz
            memory_latency_cycles: 100,
            cache_hit_cycles: 3,
            inter_core_latency_cycles: 10,
        }
    }
}

/// Cycle count for operations
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct CycleCount {
    pub compute: u64,
    pub memory: u64,
    pub communication: u64,
    pub total: u64,
}

/// Instruction timing information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstructionTiming {
    pub operation: String,
    pub cycles: u32,
    pub category: WasmOpCategory,
}

/// WASM operation categories for timing
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WasmOpCategory {
    Memory,
    Arithmetic,
    Control,
    Float,
    Integer,
    Conversion,
    Reference,
    Table,
    Global,
    Local,
}

// TODO: Implement cycle tracking and timing simulation