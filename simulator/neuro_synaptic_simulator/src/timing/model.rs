use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// Represents the number of cycles consumed by an operation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CycleCount(pub u64);

impl CycleCount {
    pub fn new(cycles: u64) -> Self {
        Self(cycles)
    }

    pub fn as_nanoseconds(&self, clock_freq_ghz: f64) -> f64 {
        (self.0 as f64) / clock_freq_ghz
    }

    pub fn as_microseconds(&self, clock_freq_ghz: f64) -> f64 {
        self.as_nanoseconds(clock_freq_ghz) / 1000.0
    }
}

/// Represents timing information for a specific instruction type
#[derive(Debug, Clone)]
pub struct InstructionTiming {
    pub base_cycles: u64,
    pub memory_penalty_cycles: u64,
    pub pipeline_stages: u8,
}

/// WASM instruction categories for cycle counting
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum WasmOpCategory {
    // Control flow
    Nop,
    Unreachable,
    Block,
    Loop,
    If,
    Branch,
    Call,
    Return,
    
    // Parametric
    Drop,
    Select,
    
    // Variable access
    LocalGet,
    LocalSet,
    LocalTee,
    GlobalGet,
    GlobalSet,
    
    // Memory operations
    I32Load,
    I64Load,
    F32Load,
    F64Load,
    I32Store,
    I64Store,
    F32Store,
    F64Store,
    MemoryGrow,
    MemorySize,
    
    // Constants
    I32Const,
    I64Const,
    F32Const,
    F64Const,
    
    // Integer arithmetic
    I32Add,
    I32Sub,
    I32Mul,
    I32DivS,
    I32DivU,
    I64Add,
    I64Sub,
    I64Mul,
    I64DivS,
    I64DivU,
    
    // Float arithmetic
    F32Add,
    F32Sub,
    F32Mul,
    F32Div,
    F32Sqrt,
    F64Add,
    F64Sub,
    F64Mul,
    F64Div,
    F64Sqrt,
    
    // Comparisons
    I32Eq,
    I32Ne,
    I32LtS,
    I32GtS,
    F32Eq,
    F32Ne,
    F32Lt,
    F32Gt,
    
    // SIMD operations (for neural network acceleration)
    V128Load,
    V128Store,
    V128Const,
    I32x4Add,
    I32x4Mul,
    F32x4Add,
    F32x4Mul,
    F32x4Fma,  // Fused multiply-add for neural ops
    
    // Neural-specific (custom extensions)
    MatrixMultiply,
    Convolution,
    Activation,
}

/// Timing model for the neuro-synaptic chip simulator
pub struct TimingModel {
    /// Clock frequency in GHz
    clock_freq_ghz: f64,
    
    /// Instruction timing map
    instruction_timings: HashMap<WasmOpCategory, InstructionTiming>,
    
    /// Performance counters per core
    core_counters: Vec<Arc<CorePerformanceCounters>>,
    
    /// Number of cores
    num_cores: usize,
}

/// Performance counters for a single core
pub struct CorePerformanceCounters {
    pub core_id: usize,
    pub cycles_executed: AtomicU64,
    pub instructions_executed: AtomicU64,
    pub memory_accesses: AtomicU64,
    pub cache_hits: AtomicU64,
    pub cache_misses: AtomicU64,
    pub stalls: AtomicU64,
    pub branch_mispredictions: AtomicU64,
}

impl TimingModel {
    /// Create a new timing model for the chip
    /// Default clock frequency is 1.5 GHz (low-power 12nm ASIC)
    pub fn new(num_cores: usize, clock_freq_ghz: f64) -> Self {
        let mut instruction_timings = HashMap::new();
        
        // Initialize instruction timings based on typical ASIC characteristics
        // Control flow
        instruction_timings.insert(WasmOpCategory::Nop, InstructionTiming { base_cycles: 1, memory_penalty_cycles: 0, pipeline_stages: 1 });
        instruction_timings.insert(WasmOpCategory::Block, InstructionTiming { base_cycles: 1, memory_penalty_cycles: 0, pipeline_stages: 1 });
        instruction_timings.insert(WasmOpCategory::Loop, InstructionTiming { base_cycles: 2, memory_penalty_cycles: 0, pipeline_stages: 2 });
        instruction_timings.insert(WasmOpCategory::If, InstructionTiming { base_cycles: 2, memory_penalty_cycles: 0, pipeline_stages: 2 });
        instruction_timings.insert(WasmOpCategory::Branch, InstructionTiming { base_cycles: 3, memory_penalty_cycles: 0, pipeline_stages: 2 });
        instruction_timings.insert(WasmOpCategory::Call, InstructionTiming { base_cycles: 5, memory_penalty_cycles: 2, pipeline_stages: 3 });
        instruction_timings.insert(WasmOpCategory::Return, InstructionTiming { base_cycles: 4, memory_penalty_cycles: 1, pipeline_stages: 2 });
        
        // Variable access
        instruction_timings.insert(WasmOpCategory::LocalGet, InstructionTiming { base_cycles: 1, memory_penalty_cycles: 0, pipeline_stages: 1 });
        instruction_timings.insert(WasmOpCategory::LocalSet, InstructionTiming { base_cycles: 1, memory_penalty_cycles: 0, pipeline_stages: 1 });
        instruction_timings.insert(WasmOpCategory::GlobalGet, InstructionTiming { base_cycles: 2, memory_penalty_cycles: 1, pipeline_stages: 2 });
        instruction_timings.insert(WasmOpCategory::GlobalSet, InstructionTiming { base_cycles: 2, memory_penalty_cycles: 1, pipeline_stages: 2 });
        
        // Memory operations (higher latency)
        instruction_timings.insert(WasmOpCategory::I32Load, InstructionTiming { base_cycles: 4, memory_penalty_cycles: 3, pipeline_stages: 3 });
        instruction_timings.insert(WasmOpCategory::I64Load, InstructionTiming { base_cycles: 4, memory_penalty_cycles: 3, pipeline_stages: 3 });
        instruction_timings.insert(WasmOpCategory::F32Load, InstructionTiming { base_cycles: 4, memory_penalty_cycles: 3, pipeline_stages: 3 });
        instruction_timings.insert(WasmOpCategory::I32Store, InstructionTiming { base_cycles: 3, memory_penalty_cycles: 2, pipeline_stages: 2 });
        instruction_timings.insert(WasmOpCategory::I64Store, InstructionTiming { base_cycles: 3, memory_penalty_cycles: 2, pipeline_stages: 2 });
        instruction_timings.insert(WasmOpCategory::F32Store, InstructionTiming { base_cycles: 3, memory_penalty_cycles: 2, pipeline_stages: 2 });
        
        // Arithmetic operations
        instruction_timings.insert(WasmOpCategory::I32Add, InstructionTiming { base_cycles: 1, memory_penalty_cycles: 0, pipeline_stages: 1 });
        instruction_timings.insert(WasmOpCategory::I32Sub, InstructionTiming { base_cycles: 1, memory_penalty_cycles: 0, pipeline_stages: 1 });
        instruction_timings.insert(WasmOpCategory::I32Mul, InstructionTiming { base_cycles: 3, memory_penalty_cycles: 0, pipeline_stages: 2 });
        instruction_timings.insert(WasmOpCategory::I32DivS, InstructionTiming { base_cycles: 12, memory_penalty_cycles: 0, pipeline_stages: 4 });
        
        // Float operations (more expensive)
        instruction_timings.insert(WasmOpCategory::F32Add, InstructionTiming { base_cycles: 3, memory_penalty_cycles: 0, pipeline_stages: 2 });
        instruction_timings.insert(WasmOpCategory::F32Sub, InstructionTiming { base_cycles: 3, memory_penalty_cycles: 0, pipeline_stages: 2 });
        instruction_timings.insert(WasmOpCategory::F32Mul, InstructionTiming { base_cycles: 4, memory_penalty_cycles: 0, pipeline_stages: 3 });
        instruction_timings.insert(WasmOpCategory::F32Div, InstructionTiming { base_cycles: 14, memory_penalty_cycles: 0, pipeline_stages: 5 });
        instruction_timings.insert(WasmOpCategory::F32Sqrt, InstructionTiming { base_cycles: 20, memory_penalty_cycles: 0, pipeline_stages: 6 });
        
        // SIMD operations (optimized for neural networks)
        instruction_timings.insert(WasmOpCategory::V128Load, InstructionTiming { base_cycles: 4, memory_penalty_cycles: 3, pipeline_stages: 3 });
        instruction_timings.insert(WasmOpCategory::V128Store, InstructionTiming { base_cycles: 3, memory_penalty_cycles: 2, pipeline_stages: 2 });
        instruction_timings.insert(WasmOpCategory::F32x4Add, InstructionTiming { base_cycles: 3, memory_penalty_cycles: 0, pipeline_stages: 2 });
        instruction_timings.insert(WasmOpCategory::F32x4Mul, InstructionTiming { base_cycles: 4, memory_penalty_cycles: 0, pipeline_stages: 3 });
        instruction_timings.insert(WasmOpCategory::F32x4Fma, InstructionTiming { base_cycles: 5, memory_penalty_cycles: 0, pipeline_stages: 3 });
        
        // Neural-specific operations (highly optimized)
        instruction_timings.insert(WasmOpCategory::MatrixMultiply, InstructionTiming { base_cycles: 8, memory_penalty_cycles: 4, pipeline_stages: 4 });
        instruction_timings.insert(WasmOpCategory::Convolution, InstructionTiming { base_cycles: 12, memory_penalty_cycles: 6, pipeline_stages: 5 });
        instruction_timings.insert(WasmOpCategory::Activation, InstructionTiming { base_cycles: 4, memory_penalty_cycles: 0, pipeline_stages: 2 });
        
        // Initialize performance counters for each core
        let mut core_counters = Vec::with_capacity(num_cores);
        for core_id in 0..num_cores {
            core_counters.push(Arc::new(CorePerformanceCounters::new(core_id)));
        }
        
        Self {
            clock_freq_ghz,
            instruction_timings,
            core_counters,
            num_cores,
        }
    }
    
    /// Get the cycle count for a specific WASM operation
    pub fn get_instruction_cycles(&self, op: WasmOpCategory, cache_hit: bool) -> CycleCount {
        if let Some(timing) = self.instruction_timings.get(&op) {
            let cycles = if cache_hit {
                timing.base_cycles
            } else {
                timing.base_cycles + timing.memory_penalty_cycles
            };
            CycleCount::new(cycles)
        } else {
            // Default timing for unknown operations
            CycleCount::new(1)
        }
    }
    
    /// Update performance counters for a core
    pub fn update_core_counters(
        &self,
        core_id: usize,
        cycles: u64,
        instructions: u64,
        memory_accesses: u64,
        cache_hits: u64,
        cache_misses: u64,
        stalls: u64,
    ) {
        if let Some(counter) = self.core_counters.get(core_id) {
            counter.cycles_executed.fetch_add(cycles, Ordering::Relaxed);
            counter.instructions_executed.fetch_add(instructions, Ordering::Relaxed);
            counter.memory_accesses.fetch_add(memory_accesses, Ordering::Relaxed);
            counter.cache_hits.fetch_add(cache_hits, Ordering::Relaxed);
            counter.cache_misses.fetch_add(cache_misses, Ordering::Relaxed);
            counter.stalls.fetch_add(stalls, Ordering::Relaxed);
        }
    }
    
    /// Get performance counters for a specific core
    pub fn get_core_counters(&self, core_id: usize) -> Option<Arc<CorePerformanceCounters>> {
        self.core_counters.get(core_id).cloned()
    }
    
    /// Calculate the total cycles executed across all cores
    pub fn get_total_cycles(&self) -> u64 {
        self.core_counters
            .iter()
            .map(|counter| counter.cycles_executed.load(Ordering::Relaxed))
            .sum()
    }
    
    /// Calculate the total execution time in microseconds
    pub fn get_total_execution_time_us(&self) -> f64 {
        let max_cycles = self.core_counters
            .iter()
            .map(|counter| counter.cycles_executed.load(Ordering::Relaxed))
            .max()
            .unwrap_or(0);
        
        CycleCount::new(max_cycles).as_microseconds(self.clock_freq_ghz)
    }
    
    /// Get average instructions per cycle (IPC) across all cores
    pub fn get_average_ipc(&self) -> f64 {
        let total_instructions: u64 = self.core_counters
            .iter()
            .map(|counter| counter.instructions_executed.load(Ordering::Relaxed))
            .sum();
        
        let total_cycles = self.get_total_cycles();
        
        if total_cycles > 0 {
            (total_instructions as f64) / (total_cycles as f64)
        } else {
            0.0
        }
    }
    
    /// Reset all performance counters
    pub fn reset_counters(&self) {
        for counter in &self.core_counters {
            counter.reset();
        }
    }
    
    /// Get clock frequency
    pub fn get_clock_freq_ghz(&self) -> f64 {
        self.clock_freq_ghz
    }
}

impl CorePerformanceCounters {
    pub fn new(core_id: usize) -> Self {
        Self {
            core_id,
            cycles_executed: AtomicU64::new(0),
            instructions_executed: AtomicU64::new(0),
            memory_accesses: AtomicU64::new(0),
            cache_hits: AtomicU64::new(0),
            cache_misses: AtomicU64::new(0),
            stalls: AtomicU64::new(0),
            branch_mispredictions: AtomicU64::new(0),
        }
    }
    
    pub fn reset(&self) {
        self.cycles_executed.store(0, Ordering::Relaxed);
        self.instructions_executed.store(0, Ordering::Relaxed);
        self.memory_accesses.store(0, Ordering::Relaxed);
        self.cache_hits.store(0, Ordering::Relaxed);
        self.cache_misses.store(0, Ordering::Relaxed);
        self.stalls.store(0, Ordering::Relaxed);
        self.branch_mispredictions.store(0, Ordering::Relaxed);
    }
    
    pub fn get_cache_hit_rate(&self) -> f64 {
        let hits = self.cache_hits.load(Ordering::Relaxed) as f64;
        let total = (self.cache_hits.load(Ordering::Relaxed) + self.cache_misses.load(Ordering::Relaxed)) as f64;
        
        if total > 0.0 {
            hits / total
        } else {
            0.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_cycle_count_conversions() {
        let cycles = CycleCount::new(1500);
        let clock_freq = 1.5; // 1.5 GHz
        
        assert_eq!(cycles.as_nanoseconds(clock_freq), 1000.0);
        assert_eq!(cycles.as_microseconds(clock_freq), 1.0);
    }
    
    #[test]
    fn test_instruction_timing() {
        let model = TimingModel::new(4, 1.5);
        
        // Test simple add operation
        let add_cycles = model.get_instruction_cycles(WasmOpCategory::I32Add, true);
        assert_eq!(add_cycles.0, 1);
        
        // Test memory load with cache miss
        let load_cycles = model.get_instruction_cycles(WasmOpCategory::I32Load, false);
        assert_eq!(load_cycles.0, 7); // 4 base + 3 penalty
        
        // Test SIMD operation
        let simd_cycles = model.get_instruction_cycles(WasmOpCategory::F32x4Fma, true);
        assert_eq!(simd_cycles.0, 5);
    }
    
    #[test]
    fn test_performance_counters() {
        let model = TimingModel::new(2, 1.5);
        
        // Update counters for core 0
        model.update_core_counters(0, 100, 50, 20, 15, 5, 10);
        
        // Check counter values
        let counter = model.get_core_counters(0).unwrap();
        assert_eq!(counter.cycles_executed.load(Ordering::Relaxed), 100);
        assert_eq!(counter.instructions_executed.load(Ordering::Relaxed), 50);
        assert_eq!(counter.get_cache_hit_rate(), 0.75); // 15 / (15 + 5)
        
        // Test IPC calculation
        assert_eq!(model.get_average_ipc(), 0.5); // 50 instructions / 100 cycles
    }
}