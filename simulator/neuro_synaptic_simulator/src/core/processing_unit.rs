use serde::{Serialize, Deserialize};
use std::sync::{Arc, Mutex};
use anyhow::Result;

/// Represents a single processing unit (core) in the neuro-synaptic chip
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingUnit {
    /// Unique identifier for this core (0-255)
    pub id: u8,
    
    /// Current state of the processing unit
    pub state: ProcessingState,
    
    /// Local memory allocation for this core (in bytes)
    pub local_memory_size: usize,
    
    /// Performance counters
    pub counters: PerformanceCounters,
    
    /// Current workload assigned to this core
    pub workload: Option<Workload>,
}

/// Processing unit state
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ProcessingState {
    /// Core is idle and ready for work
    Idle,
    
    /// Core is actively processing
    Processing,
    
    /// Core is waiting for memory access
    WaitingForMemory,
    
    /// Core is in error state
    Error(String),
    
    /// Core is powered down
    PoweredDown,
}

/// Performance counters for monitoring
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PerformanceCounters {
    /// Total cycles executed
    pub cycles: u64,
    
    /// Number of instructions executed
    pub instructions: u64,
    
    /// Memory reads performed
    pub memory_reads: u64,
    
    /// Memory writes performed
    pub memory_writes: u64,
    
    /// Cache hits
    pub cache_hits: u64,
    
    /// Cache misses
    pub cache_misses: u64,
    
    /// Time spent idle (in nanoseconds)
    pub idle_time_ns: u64,
    
    /// Time spent processing (in nanoseconds)
    pub processing_time_ns: u64,
}

/// Workload assigned to a processing unit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workload {
    /// Unique workload identifier
    pub id: String,
    
    /// Type of workload
    pub workload_type: WorkloadType,
    
    /// WASM module bytecode (if applicable)
    pub wasm_bytecode: Option<Vec<u8>>,
    
    /// Input data for processing
    pub input_data: Vec<u8>,
    
    /// Expected completion time (in timesteps)
    pub expected_duration: u32,
}

/// Types of workloads that can be executed
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum WorkloadType {
    /// WASM module execution
    WasmExecution,
    
    /// Neural network inference
    NeuralInference,
    
    /// Matrix multiplication
    MatrixMultiply,
    
    /// Memory copy operation
    MemoryCopy,
    
    /// Custom computation
    Custom(String),
}

impl ProcessingUnit {
    /// Create a new processing unit with given ID
    pub fn new(id: u8, local_memory_size: usize) -> Self {
        ProcessingUnit {
            id,
            state: ProcessingState::Idle,
            local_memory_size,
            counters: PerformanceCounters::default(),
            workload: None,
        }
    }
    
    /// Assign a workload to this processing unit
    pub fn assign_workload(&mut self, workload: Workload) -> Result<()> {
        match self.state {
            ProcessingState::Idle => {
                self.workload = Some(workload);
                self.state = ProcessingState::Processing;
                Ok(())
            }
            _ => Err(anyhow::anyhow!(
                "Cannot assign workload to core {} in state {:?}",
                self.id,
                self.state
            )),
        }
    }
    
    /// Execute one cycle of processing
    pub fn tick(&mut self) -> Result<()> {
        match self.state {
            ProcessingState::Processing => {
                // Increment cycle counter
                self.counters.cycles += 1;
                
                // TODO: Actual processing logic
                // - Execute WASM instructions
                // - Update performance counters
                // - Check for completion
                
                Ok(())
            }
            ProcessingState::Idle => {
                self.counters.idle_time_ns += 1_000; // Assuming 1us per tick
                Ok(())
            }
            _ => Ok(()),
        }
    }
    
    /// Reset performance counters
    pub fn reset_counters(&mut self) {
        self.counters = PerformanceCounters::default();
    }
    
    /// Get current utilization percentage (0-100)
    pub fn get_utilization(&self) -> f32 {
        let total_time = self.counters.processing_time_ns + self.counters.idle_time_ns;
        if total_time == 0 {
            0.0
        } else {
            (self.counters.processing_time_ns as f32 / total_time as f32) * 100.0
        }
    }
    
    /// Check if the core is available for work
    pub fn is_available(&self) -> bool {
        matches!(self.state, ProcessingState::Idle)
    }
    
    /// Power down the core
    pub fn power_down(&mut self) {
        self.state = ProcessingState::PoweredDown;
        self.workload = None;
    }
    
    /// Power up the core
    pub fn power_up(&mut self) {
        if matches!(self.state, ProcessingState::PoweredDown) {
            self.state = ProcessingState::Idle;
        }
    }
}

/// Thread-safe wrapper for ProcessingUnit
pub type SharedProcessingUnit = Arc<Mutex<ProcessingUnit>>;

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_processing_unit_creation() {
        let unit = ProcessingUnit::new(0, 1024 * 1024); // 1MB local memory
        assert_eq!(unit.id, 0);
        assert_eq!(unit.state, ProcessingState::Idle);
        assert_eq!(unit.local_memory_size, 1024 * 1024);
        assert!(unit.is_available());
    }
    
    #[test]
    fn test_workload_assignment() {
        let mut unit = ProcessingUnit::new(1, 1024 * 1024);
        let workload = Workload {
            id: "test-workload".to_string(),
            workload_type: WorkloadType::WasmExecution,
            wasm_bytecode: Some(vec![0x00, 0x61, 0x73, 0x6d]), // WASM magic number
            input_data: vec![1, 2, 3, 4],
            expected_duration: 100,
        };
        
        assert!(unit.assign_workload(workload).is_ok());
        assert_eq!(unit.state, ProcessingState::Processing);
        assert!(!unit.is_available());
    }
    
    #[test]
    fn test_power_management() {
        let mut unit = ProcessingUnit::new(2, 1024 * 1024);
        
        unit.power_down();
        assert_eq!(unit.state, ProcessingState::PoweredDown);
        
        unit.power_up();
        assert_eq!(unit.state, ProcessingState::Idle);
    }
}