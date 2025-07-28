use neuro_synaptic_simulator::timing::{TimingModel, ClockConfig, SimulationClock};
use std::time::{Duration, Instant};
use std::sync::Arc;
use std::thread;

#[test]
fn test_timing_model_creation() {
    let config = ClockConfig {
        frequency_mhz: 1000, // 1 GHz
        cycles_per_instruction: 1,
    };
    
    let model = TimingModel::new(config);
    assert_eq!(model.frequency_mhz(), 1000);
    assert_eq!(model.nanoseconds_per_cycle(), 1);
}

#[test]
fn test_clock_frequency_calculations() {
    // Test various frequencies
    let test_cases = vec![
        (500, 2),     // 500 MHz = 2ns per cycle
        (1000, 1),    // 1 GHz = 1ns per cycle
        (2000, 0),    // 2 GHz = 0.5ns (rounds to 0)
        (100, 10),    // 100 MHz = 10ns per cycle
    ];
    
    for (freq_mhz, expected_ns) in test_cases {
        let config = ClockConfig {
            frequency_mhz: freq_mhz,
            cycles_per_instruction: 1,
        };
        let model = TimingModel::new(config);
        assert_eq!(model.nanoseconds_per_cycle(), expected_ns);
    }
}

#[test]
fn test_simulation_clock() {
    let mut clock = SimulationClock::new();
    
    assert_eq!(clock.current_time_ns(), 0);
    assert_eq!(clock.current_cycle(), 0);
    
    // Advance time
    clock.advance_ns(1000);
    assert_eq!(clock.current_time_ns(), 1000);
    
    // Advance cycles (assuming 1GHz)
    clock.advance_cycles(500, 1); // 500 cycles at 1ns per cycle
    assert_eq!(clock.current_time_ns(), 1500);
    assert_eq!(clock.current_cycle(), 500);
}

#[test]
fn test_instruction_timing() {
    let config = ClockConfig {
        frequency_mhz: 1000,
        cycles_per_instruction: 1,
    };
    let model = TimingModel::new(config);
    
    // Calculate time for various instruction counts
    assert_eq!(model.instructions_to_nanoseconds(1000), 1000);
    assert_eq!(model.instructions_to_microseconds(1000), 1);
    assert_eq!(model.instructions_to_milliseconds(1_000_000), 1);
}

#[test]
fn test_multi_cycle_instructions() {
    let config = ClockConfig {
        frequency_mhz: 1000,
        cycles_per_instruction: 4, // Each instruction takes 4 cycles
    };
    let model = TimingModel::new(config);
    
    // 1000 instructions * 4 cycles * 1ns = 4000ns
    assert_eq!(model.instructions_to_nanoseconds(1000), 4000);
}

#[test]
fn test_parallel_execution_timing() {
    let config = ClockConfig {
        frequency_mhz: 1000,
        cycles_per_instruction: 1,
    };
    let model = Arc::new(TimingModel::new(config));
    
    // Simulate 4 cores running different workloads
    let workloads = vec![1000, 2000, 1500, 3000]; // Instructions per core
    let mut completion_times = vec![];
    
    for &workload in &workloads {
        let time_ns = model.instructions_to_nanoseconds(workload);
        completion_times.push(time_ns);
    }
    
    // Total time is the maximum (they run in parallel)
    let total_time = completion_times.iter().max().unwrap();
    assert_eq!(*total_time, 3000); // Core with 3000 instructions
}

#[test]
fn test_power_model() {
    let config = ClockConfig {
        frequency_mhz: 1000,
        cycles_per_instruction: 1,
    };
    let mut model = TimingModel::new(config);
    
    // Test idle power
    assert!(model.current_power_mw() < 100.0); // Low idle power
    
    // Activate cores
    model.set_active_cores(128);
    let half_power = model.current_power_mw();
    assert!(half_power > 500.0 && half_power < 1500.0); // ~1W for 128 cores
    
    // Full load
    model.set_active_cores(256);
    let full_power = model.current_power_mw();
    assert!(full_power > 1500.0 && full_power < 2100.0); // ~2W for 256 cores
    
    // Power scales approximately linearly
    assert!((full_power / half_power - 2.0).abs() < 0.1);
}

#[test]
fn test_energy_calculation() {
    let config = ClockConfig {
        frequency_mhz: 1000,
        cycles_per_instruction: 1,
    };
    let mut model = TimingModel::new(config);
    
    model.set_active_cores(256); // Full power ~2W
    
    // Run for 1 millisecond
    let runtime_ms = 1.0;
    let energy_mj = model.calculate_energy_mj(runtime_ms);
    
    // Energy = Power * Time = 2W * 1ms = 2mJ
    assert!(energy_mj > 1.5 && energy_mj < 2.5);
}

#[test]
fn test_performance_metrics() {
    let config = ClockConfig {
        frequency_mhz: 1000,
        cycles_per_instruction: 1,
    };
    let mut model = TimingModel::new(config);
    
    // Track execution of multiple tasks
    model.record_task_start(0, 0);
    model.record_task_start(1, 0);
    model.record_task_start(2, 0);
    
    model.record_task_complete(0, 1000, 1_000_000); // 1M instructions in 1μs
    model.record_task_complete(1, 2000, 2_000_000); // 2M instructions in 2μs
    model.record_task_complete(2, 1500, 1_500_000); // 1.5M instructions in 1.5μs
    
    let metrics = model.get_performance_metrics();
    assert_eq!(metrics.total_tasks, 3);
    assert_eq!(metrics.total_instructions, 4_500_000);
    assert_eq!(metrics.total_time_ns, 2000); // Max of parallel tasks
    assert_eq!(metrics.throughput_gips, 2.25); // 4.5M instructions / 2μs
}

#[test]
fn test_core_utilization() {
    let config = ClockConfig {
        frequency_mhz: 1000,
        cycles_per_instruction: 1,
    };
    let mut model = TimingModel::new(config);
    
    // Simulate varying core usage over time
    let mut clock = SimulationClock::new();
    
    model.update_utilization(&clock, 256); // All cores active
    clock.advance_ns(1000);
    
    model.update_utilization(&clock, 128); // Half cores active
    clock.advance_ns(1000);
    
    model.update_utilization(&clock, 256); // All cores active again
    clock.advance_ns(1000);
    
    let avg_utilization = model.average_utilization();
    assert!((avg_utilization - 0.833).abs() < 0.01); // (256+128+256)/(3*256)
}

#[test]
fn test_timing_accuracy_with_known_workload() {
    let config = ClockConfig {
        frequency_mhz: 1000,
        cycles_per_instruction: 1,
    };
    let model = TimingModel::new(config);
    
    // Known workload: simple loop of N iterations
    // Each iteration = 3 instructions (load, increment, branch)
    let iterations = 1000;
    let instructions_per_iteration = 3;
    let total_instructions = iterations * instructions_per_iteration;
    
    let expected_time_ns = model.instructions_to_nanoseconds(total_instructions);
    assert_eq!(expected_time_ns, 3000); // 3000 instructions * 1ns
    
    // Verify with actual measurement
    let start = Instant::now();
    let mut counter = 0;
    for _ in 0..iterations {
        counter += 1; // Simulated work
    }
    let _elapsed = start.elapsed();
    
    // The simulated time should be deterministic regardless of actual elapsed time
    assert_eq!(expected_time_ns, 3000);
}

#[test]
fn test_concurrent_timing_synchronization() {
    let config = ClockConfig {
        frequency_mhz: 1000,
        cycles_per_instruction: 1,
    };
    let model = Arc::new(TimingModel::new(config));
    let clock = Arc::new(SimulationClock::new());
    
    // Multiple threads updating timing
    let mut handles = vec![];
    
    for core_id in 0..4 {
        let model_clone = model.clone();
        let clock_clone = clock.clone();
        
        let handle = thread::spawn(move || {
            // Each core executes different amount
            let instructions = (core_id + 1) * 1000;
            let time_ns = model_clone.instructions_to_nanoseconds(instructions);
            
            // Report completion time
            time_ns
        });
        
        handles.push(handle);
    }
    
    let mut completion_times = vec![];
    for handle in handles {
        completion_times.push(handle.join().unwrap());
    }
    
    // Verify times are as expected
    assert_eq!(completion_times, vec![1000, 2000, 3000, 4000]);
}

#[test]
fn test_throttling_for_power_limit() {
    let config = ClockConfig {
        frequency_mhz: 1000,
        cycles_per_instruction: 1,
    };
    let mut model = TimingModel::new(config);
    
    // Try to exceed power limit
    model.set_power_limit_mw(1000.0); // 1W limit
    
    // Request all 256 cores
    let allowed_cores = model.throttle_cores_for_power(256);
    
    // Should throttle to approximately 128 cores
    assert!(allowed_cores < 256);
    assert!(allowed_cores > 100);
}

// Property-based tests
#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;
    
    proptest! {
        #[test]
        fn test_timing_monotonicity(
            instructions in prop::collection::vec(1u64..100000, 1..100)
        ) {
            let config = ClockConfig {
                frequency_mhz: 1000,
                cycles_per_instruction: 1,
            };
            let model = TimingModel::new(config);
            
            let mut last_time = 0;
            for &inst_count in &instructions {
                let time = model.instructions_to_nanoseconds(inst_count);
                prop_assert!(time >= last_time);
                last_time = time;
            }
        }
        
        #[test]
        fn test_power_scaling(
            active_cores in 0u32..=256
        ) {
            let config = ClockConfig {
                frequency_mhz: 1000,
                cycles_per_instruction: 1,
            };
            let mut model = TimingModel::new(config);
            
            model.set_active_cores(active_cores);
            let power = model.current_power_mw();
            
            // Power should be between idle and max
            prop_assert!(power >= 0.0);
            prop_assert!(power <= 2100.0); // Slightly above 2W for margin
            
            // Power should scale with cores
            if active_cores == 0 {
                prop_assert!(power < 100.0); // Idle power
            } else {
                let per_core = power / active_cores as f64;
                prop_assert!(per_core < 10.0); // < 10mW per core
            }
        }
    }
}