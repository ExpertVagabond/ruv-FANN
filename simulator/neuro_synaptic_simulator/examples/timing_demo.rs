use neuro_synaptic_simulator::{
    TimingModel, PowerModel, PerformanceMonitor,
    WasmOpCategory, PowerState
};
use std::sync::Arc;

fn main() {
    println!("=== Neuro-Synaptic Chip Timing & Power Demo ===\n");
    
    // Initialize models for 256-core chip at 1.5 GHz with 2W power budget
    let timing_model = Arc::new(TimingModel::new(256, 1.5));
    let power_model = Arc::new(PowerModel::new(256, 2.0));
    let monitor = PerformanceMonitor::new(timing_model.clone(), power_model.clone());
    
    println!("Chip Configuration:");
    println!("- 256 cores @ 1.5 GHz");
    println!("- 2W max power budget");
    println!("- 28MB shared memory\n");
    
    // Simulate a neural network inference workload
    println!("Simulating neural network inference...\n");
    
    // Activate 64 cores for the workload
    let active_cores = 64;
    for core_id in 0..active_cores {
        power_model.set_core_state(core_id, PowerState::ActiveHigh, 0);
    }
    
    // Simulate executing WASM instructions on active cores
    let mut total_cycles = 0u64;
    let mut total_instructions = 0u64;
    
    // Each core executes a mix of operations
    for core_id in 0..active_cores {
        let mut core_cycles = 0u64;
        let mut core_instructions = 0u64;
        
        // Simulate matrix multiplication (common in neural networks)
        for _ in 0..100 {
            let cycles = timing_model.get_instruction_cycles(WasmOpCategory::MatrixMultiply, true);
            core_cycles += cycles.0;
            core_instructions += 1;
        }
        
        // Simulate SIMD operations
        for _ in 0..500 {
            let cycles = timing_model.get_instruction_cycles(WasmOpCategory::F32x4Fma, true);
            core_cycles += cycles.0;
            core_instructions += 1;
        }
        
        // Simulate memory operations
        for _ in 0..200 {
            let cycles = timing_model.get_instruction_cycles(WasmOpCategory::V128Load, false);
            core_cycles += cycles.0;
            core_instructions += 1;
            
            monitor.record_memory_access(128); // 128 bytes per SIMD load
        }
        
        // Update performance counters
        timing_model.update_core_counters(
            core_id,
            core_cycles,
            core_instructions,
            200,  // memory accesses
            180,  // cache hits
            20,   // cache misses
            0     // stalls
        );
        
        total_cycles += core_cycles;
        total_instructions += core_instructions;
    }
    
    // Calculate and display timing results
    let execution_time_us = timing_model.get_total_execution_time_us();
    println!("Timing Results:");
    println!("- Total instructions executed: {}", total_instructions);
    println!("- Total cycles: {}", total_cycles);
    println!("- Execution time: {:.2} μs", execution_time_us);
    println!("- Average IPC: {:.2}", timing_model.get_average_ipc());
    
    // Display power results
    let current_power = power_model.get_current_power();
    println!("\nPower Results:");
    println!("- Current power consumption: {:.3} W", current_power);
    println!("- Power budget utilization: {:.1}%", (current_power / 2.0) * 100.0);
    
    // Check if we can activate more cores
    let additional_cores = 192; // Try to activate remaining cores
    if power_model.can_activate_cores(additional_cores, PowerState::ActiveHigh) {
        println!("- Can activate {} more cores within power budget", additional_cores);
    } else {
        println!("- Cannot activate {} more cores (would exceed power budget)", additional_cores);
    }
    
    // Apply DVFS based on utilization
    power_model.apply_dvfs(0.25, (execution_time_us * 1000.0) as u64);
    
    // Get energy statistics
    let energy_stats = power_model.get_energy_stats((execution_time_us * 1000.0) as u64);
    println!("\nEnergy Statistics:");
    println!("- Total energy consumed: {:.6} J", energy_stats.total_joules);
    println!("- Average power: {:.3} W", energy_stats.average_watts);
    
    // Generate performance report
    println!("\n{}", monitor.generate_report());
    
    // Demonstrate instruction timing details
    println!("\n=== Instruction Timing Details ===");
    println!("Operation            | Cache Hit | Cache Miss");
    println!("---------------------|-----------|------------");
    
    let ops = vec![
        ("I32 Add", WasmOpCategory::I32Add),
        ("F32 Mul", WasmOpCategory::F32Mul),
        ("F32x4 FMA", WasmOpCategory::F32x4Fma),
        ("Matrix Multiply", WasmOpCategory::MatrixMultiply),
        ("Memory Load", WasmOpCategory::I32Load),
        ("SIMD Load", WasmOpCategory::V128Load),
    ];
    
    for (name, op) in ops {
        let hit_cycles = timing_model.get_instruction_cycles(op, true);
        let miss_cycles = timing_model.get_instruction_cycles(op, false);
        println!("{:<20} | {:>9} | {:>10}", 
            name, 
            format!("{} cyc", hit_cycles.0),
            format!("{} cyc", miss_cycles.0)
        );
    }
}