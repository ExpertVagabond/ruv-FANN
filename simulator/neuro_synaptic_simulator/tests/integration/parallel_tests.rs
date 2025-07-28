use ruv_fann_simulator::{SimulatorCore, ParallelEngine, WasmModule};
use std::sync::Arc;
use std::time::Instant;
use rayon::prelude::*;

/// Test that 256 cores are properly utilized
#[test]
fn test_256_core_utilization() {
    let engine = ParallelEngine::new(256);
    assert_eq!(engine.num_cores(), 256);
    
    // Create a workload that should utilize all cores
    let workload: Vec<f32> = (0..256_000).map(|i| i as f32).collect();
    let start = Instant::now();
    
    let results: Vec<f32> = engine.parallel_process(&workload, |&x| x * x + x.sqrt());
    
    let duration = start.elapsed();
    println!("Processed {} items in {:?} using 256 cores", workload.len(), duration);
    
    // Verify results
    for (i, &result) in results.iter().enumerate() {
        let expected = (i as f32) * (i as f32) + (i as f32).sqrt();
        assert!((result - expected).abs() < 1e-5);
    }
}

/// Test core scaling performance
#[test]
fn test_core_scaling() {
    let core_counts = vec![1, 4, 16, 64, 128, 256];
    let workload_size = 1_000_000;
    let workload: Vec<f32> = (0..workload_size).map(|i| (i as f32) * 0.01).collect();
    
    let mut timings = Vec::new();
    
    for &cores in &core_counts {
        let engine = ParallelEngine::new(cores);
        let start = Instant::now();
        
        let _results: Vec<f32> = engine.parallel_process(&workload, |&x| {
            // Simulate neural network computation
            let mut result = x;
            for _ in 0..10 {
                result = (result * 2.0 - 1.0).tanh();
            }
            result
        });
        
        let duration = start.elapsed();
        timings.push((cores, duration));
        println!("Cores: {}, Time: {:?}", cores, duration);
    }
    
    // Verify scaling efficiency
    let single_core_time = timings[0].1.as_secs_f64();
    for (cores, duration) in timings.iter().skip(1) {
        let speedup = single_core_time / duration.as_secs_f64();
        let efficiency = speedup / (*cores as f64);
        println!("Cores: {}, Speedup: {:.2}x, Efficiency: {:.2}%", 
                 cores, speedup, efficiency * 100.0);
        
        // Expect at least 70% efficiency for reasonable core counts
        if *cores <= 64 {
            assert!(efficiency > 0.7, "Poor scaling efficiency at {} cores", cores);
        }
    }
}

/// Test parallel WASM module execution
#[test]
fn test_parallel_wasm_execution() {
    let module_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/wasm_modules/parallel_compute.wasm");
    
    let module = Arc::new(WasmModule::load(&module_path).expect("Failed to load WASM"));
    let engine = ParallelEngine::new(256);
    
    // Create multiple input sets
    let input_sets: Vec<Vec<f32>> = (0..1000)
        .map(|i| vec![i as f32, (i * 2) as f32, (i * 3) as f32])
        .collect();
    
    let start = Instant::now();
    
    // Process all input sets in parallel
    let results: Vec<Vec<f32>> = engine.parallel_process_wasm(
        &input_sets,
        module.clone(),
        "process_inputs"
    ).expect("Failed to process WASM in parallel");
    
    let duration = start.elapsed();
    println!("Processed {} WASM executions in {:?}", input_sets.len(), duration);
    
    // Verify results
    assert_eq!(results.len(), input_sets.len());
    for (i, result) in results.iter().enumerate() {
        assert!(!result.is_empty(), "Result {} is empty", i);
    }
}

/// Test memory consistency across parallel execution
#[test]
fn test_parallel_memory_consistency() {
    let engine = ParallelEngine::new(256);
    let shared_state = Arc::new(std::sync::Mutex::new(Vec::new()));
    
    // Run parallel operations that access shared state
    let operations = 10_000;
    let shared_state_clone = shared_state.clone();
    
    engine.parallel_execute(operations, move |i| {
        let mut state = shared_state_clone.lock().unwrap();
        state.push(i);
    });
    
    // Verify all operations completed
    let final_state = shared_state.lock().unwrap();
    assert_eq!(final_state.len(), operations);
    
    // Verify no duplicates (each operation ran exactly once)
    let mut sorted = final_state.clone();
    sorted.sort_unstable();
    for i in 0..operations {
        assert_eq!(sorted[i], i);
    }
}

/// Test error handling in parallel execution
#[test]
fn test_parallel_error_handling() {
    let engine = ParallelEngine::new(256);
    
    // Create workload where some operations will fail
    let workload: Vec<f32> = (-100..100).map(|i| i as f32).collect();
    
    let results = engine.parallel_process_with_errors(&workload, |&x| {
        if x < 0.0 {
            Err(format!("Negative value: {}", x))
        } else {
            Ok(x.sqrt())
        }
    });
    
    // Count successes and failures
    let mut successes = 0;
    let mut failures = 0;
    
    for result in results {
        match result {
            Ok(_) => successes += 1,
            Err(_) => failures += 1,
        }
    }
    
    assert_eq!(successes, 100); // 0 to 99
    assert_eq!(failures, 100);  // -100 to -1
}

/// Test load balancing across cores
#[test]
fn test_load_balancing() {
    let engine = ParallelEngine::new(256);
    
    // Create uneven workload (some tasks take longer)
    let workload: Vec<(usize, u32)> = (0..1000)
        .map(|i| (i, if i % 10 == 0 { 1000 } else { 100 }))
        .collect();
    
    let start = Instant::now();
    
    let results: Vec<u64> = engine.parallel_process(&workload, |(idx, iterations)| {
        // Simulate variable computation time
        let mut sum = 0u64;
        for i in 0..*iterations {
            sum += (*idx as u64) * (i as u64);
        }
        sum
    });
    
    let duration = start.elapsed();
    println!("Load-balanced execution completed in {:?}", duration);
    
    // Verify all tasks completed
    assert_eq!(results.len(), workload.len());
}

/// Test concurrent access to simulator core
#[test]
fn test_concurrent_simulator_access() {
    let simulator = Arc::new(SimulatorCore::new(256));
    let num_threads = 16;
    let operations_per_thread = 100;
    
    let handles: Vec<_> = (0..num_threads)
        .map(|thread_id| {
            let sim = simulator.clone();
            std::thread::spawn(move || {
                for i in 0..operations_per_thread {
                    let input = vec![thread_id as f32, i as f32];
                    let result = sim.process_input(&input);
                    assert!(!result.is_empty());
                }
            })
        })
        .collect();
    
    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }
    
    // Verify simulator is still functional
    let final_result = simulator.process_input(&vec![1.0, 2.0, 3.0]);
    assert!(!final_result.is_empty());
}