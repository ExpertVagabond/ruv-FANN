// Neuro-Synaptic Chip Simulator Library

pub mod core;
pub mod memory;
pub mod wasm;
pub mod timing;
pub mod logging;

// Re-export commonly used types
pub use core::{ProcessingUnit, ProcessingUnitState, PowerState};
pub use memory::{SharedMemory, MemoryError, MEMORY_SIZE};
pub use wasm::{WasmExecutor, WasmConfig};
pub use timing::{TimingModel, ClockConfig, SimulationClock};
pub use logging::{Logger, LogEvent, SimulationResult, LogConfig};