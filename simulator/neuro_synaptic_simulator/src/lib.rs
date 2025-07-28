//! Neuro-Synaptic Simulator Library
//! 
//! High-performance simulation of a 256-core neuro-synaptic chip with
//! parallel execution capabilities and WASM support.

// pub mod core;
pub mod memory;
// pub mod wasm;
pub mod timing;
pub mod logging;
// pub mod performance;
// pub mod visualization;

// Re-export commonly used types
// pub use crate::core::{
//     CoreScheduler,
//     SchedulerConfig,
//     DistributionStrategy,
//     SyncMode,
// };

pub use crate::memory::{
    SharedMemory,
    // LayerMemory,
    PartitionStrategy,
};

pub use crate::timing::{
    TimingModel,
    PowerModel,
    CycleCount,
    InstructionTiming,
    PowerState,
    EnergyConsumption,
    WasmOpCategory,
};

// pub use crate::performance::{
//     PerformanceMonitor,
//     PerformanceMetrics,
// };