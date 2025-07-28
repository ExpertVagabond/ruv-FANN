use neuro_synaptic_simulator::wasm::{WasmExecutor, WasmConfig, WasmModule};
use wasmtime::*;
use std::sync::Arc;

// Test WASM module that adds two numbers
const ADD_WASM: &[u8] = &[
    0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, // WASM header
    0x01, 0x07, 0x01, 0x60, 0x02, 0x7f, 0x7f, 0x01, 0x7f, // Type section
    0x03, 0x02, 0x01, 0x00, // Function section
    0x07, 0x07, 0x01, 0x03, 0x61, 0x64, 0x64, 0x00, 0x00, // Export section
    0x0a, 0x09, 0x01, 0x07, 0x00, 0x20, 0x00, 0x20, 0x01, 0x6a, 0x0b, // Code section
];

// Test WASM with memory operations
const MEMORY_WASM: &[u8] = include_bytes!("../../fixtures/memory_test.wasm");

// Test WASM with SIMD operations
const SIMD_WASM: &[u8] = include_bytes!("../../fixtures/simd_test.wasm");

#[test]
fn test_wasm_executor_creation() {
    let config = WasmConfig::default();
    let executor = WasmExecutor::new(config).unwrap();
    
    assert!(executor.engine().is_some());
    assert_eq!(executor.loaded_modules().len(), 0);
}

#[test]
fn test_wasm_config_validation() {
    let mut config = WasmConfig::default();
    
    // Valid config
    config.enable_simd = true;
    config.memory_pages = 448; // 28MB
    config.fuel_per_instruction = 1;
    
    let executor = WasmExecutor::new(config.clone());
    assert!(executor.is_ok());
    
    // Invalid memory size
    config.memory_pages = 500; // > 28MB
    let executor = WasmExecutor::new(config);
    assert!(executor.is_err());
}

#[test]
fn test_wasm_module_loading() {
    let config = WasmConfig::default();
    let mut executor = WasmExecutor::new(config).unwrap();
    
    // Load module
    let module_id = executor.load_module("add", ADD_WASM).unwrap();
    assert_eq!(executor.loaded_modules().len(), 1);
    assert!(executor.get_module(&module_id).is_some());
    
    // Cannot load same name twice
    let result = executor.load_module("add", ADD_WASM);
    assert!(result.is_err());
    
    // Can load different name
    let module_id2 = executor.load_module("add2", ADD_WASM).unwrap();
    assert_ne!(module_id, module_id2);
}

#[test]
fn test_wasm_execution() {
    let config = WasmConfig::default();
    let mut executor = WasmExecutor::new(config).unwrap();
    
    let module_id = executor.load_module("add", ADD_WASM).unwrap();
    
    // Execute function
    let args = vec![Val::I32(5), Val::I32(3)];
    let results = executor.execute_function(&module_id, "add", &args).unwrap();
    
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].i32(), Some(8));
}

#[test]
fn test_wasm_fuel_consumption() {
    let mut config = WasmConfig::default();
    config.fuel_per_instruction = 1;
    
    let mut executor = WasmExecutor::new(config).unwrap();
    let module_id = executor.load_module("add", ADD_WASM).unwrap();
    
    // Set initial fuel
    executor.set_fuel(&module_id, 1000).unwrap();
    
    // Execute function
    let args = vec![Val::I32(100), Val::I32(200)];
    executor.execute_function(&module_id, "add", &args).unwrap();
    
    // Check fuel consumed
    let consumed = executor.fuel_consumed(&module_id).unwrap();
    assert!(consumed > 0);
    assert!(consumed < 1000);
}

#[test]
fn test_wasm_memory_access() {
    let config = WasmConfig::default();
    let mut executor = WasmExecutor::new(config).unwrap();
    let shared_memory = Arc::new(SharedMemory::new());
    
    // Attach shared memory
    executor.attach_shared_memory(shared_memory.clone());
    
    // Load memory test module
    let module_id = executor.load_module("memory_test", MEMORY_WASM).unwrap();
    
    // Write data to shared memory
    let test_data = vec![1, 2, 3, 4, 5];
    shared_memory.write(0x1000, &test_data).unwrap();
    
    // Execute function that reads from memory
    let args = vec![Val::I32(0x1000), Val::I32(5)];
    let results = executor.execute_function(&module_id, "sum_bytes", &args).unwrap();
    
    // Should sum the bytes: 1+2+3+4+5 = 15
    assert_eq!(results[0].i32(), Some(15));
}

#[test]
fn test_wasm_simd_operations() {
    let mut config = WasmConfig::default();
    config.enable_simd = true;
    
    let mut executor = WasmExecutor::new(config).unwrap();
    let module_id = executor.load_module("simd", SIMD_WASM).unwrap();
    
    // Execute SIMD vector add
    let results = executor.execute_function(&module_id, "vector_add", &[]).unwrap();
    
    // Verify SIMD was used (check performance counters)
    let stats = executor.get_stats(&module_id).unwrap();
    assert!(stats.simd_operations > 0);
}

#[test]
fn test_wasm_instance_isolation() {
    let config = WasmConfig::default();
    let mut executor = WasmExecutor::new(config).unwrap();
    
    // Load same module twice with different names
    let module_id1 = executor.load_module("instance1", ADD_WASM).unwrap();
    let module_id2 = executor.load_module("instance2", ADD_WASM).unwrap();
    
    // Execute on both instances
    let args1 = vec![Val::I32(10), Val::I32(20)];
    let args2 = vec![Val::I32(30), Val::I32(40)];
    
    let results1 = executor.execute_function(&module_id1, "add", &args1).unwrap();
    let results2 = executor.execute_function(&module_id2, "add", &args2).unwrap();
    
    assert_eq!(results1[0].i32(), Some(30));
    assert_eq!(results2[0].i32(), Some(70));
    
    // Verify instances have separate fuel counts
    executor.set_fuel(&module_id1, 1000).unwrap();
    executor.set_fuel(&module_id2, 2000).unwrap();
    
    assert_ne!(
        executor.fuel_remaining(&module_id1).unwrap(),
        executor.fuel_remaining(&module_id2).unwrap()
    );
}

#[test]
fn test_wasm_error_handling() {
    let config = WasmConfig::default();
    let mut executor = WasmExecutor::new(config).unwrap();
    
    // Invalid WASM
    let result = executor.load_module("invalid", b"not wasm");
    assert!(result.is_err());
    
    // Load valid module
    let module_id = executor.load_module("add", ADD_WASM).unwrap();
    
    // Call non-existent function
    let args = vec![Val::I32(1)];
    let result = executor.execute_function(&module_id, "nonexistent", &args);
    assert!(result.is_err());
    
    // Wrong number of arguments
    let args = vec![Val::I32(1)]; // Need 2 args
    let result = executor.execute_function(&module_id, "add", &args);
    assert!(result.is_err());
}

#[test]
fn test_wasm_resource_limits() {
    let mut config = WasmConfig::default();
    config.max_instances = 2;
    
    let mut executor = WasmExecutor::new(config).unwrap();
    
    // Load up to limit
    executor.load_module("module1", ADD_WASM).unwrap();
    executor.load_module("module2", ADD_WASM).unwrap();
    
    // Should fail on third
    let result = executor.load_module("module3", ADD_WASM);
    assert!(result.is_err());
}

#[test]
fn test_wasm_concurrent_execution() {
    use std::thread;
    use std::sync::Arc;
    
    let config = WasmConfig::default();
    let executor = Arc::new(WasmExecutor::new(config).unwrap());
    
    // Load module
    let mut exec = executor.clone();
    let exec_mut = Arc::get_mut(&mut exec).unwrap();
    let module_id = exec_mut.load_module("add", ADD_WASM).unwrap();
    drop(exec);
    
    // Execute concurrently from multiple threads
    let mut handles = vec![];
    
    for i in 0..10 {
        let executor_clone = executor.clone();
        let mod_id = module_id.clone();
        
        let handle = thread::spawn(move || {
            let args = vec![Val::I32(i), Val::I32(i * 2)];
            let results = executor_clone.execute_function(&mod_id, "add", &args).unwrap();
            assert_eq!(results[0].i32(), Some(i + i * 2));
        });
        
        handles.push(handle);
    }
    
    for handle in handles {
        handle.join().unwrap();
    }
}

#[test]
fn test_wasm_memory_growth() {
    let config = WasmConfig::default();
    let mut executor = WasmExecutor::new(config).unwrap();
    
    // Load module that tries to grow memory
    let grow_wasm = include_bytes!("../../fixtures/memory_grow.wasm");
    let module_id = executor.load_module("grow", grow_wasm).unwrap();
    
    // Initial memory size
    let initial_pages = executor.memory_pages(&module_id).unwrap();
    
    // Try to grow within limits
    let args = vec![Val::I32(10)]; // Grow by 10 pages
    let result = executor.execute_function(&module_id, "grow_memory", &args).unwrap();
    assert!(result[0].i32().unwrap() >= 0); // Success
    
    // Try to grow beyond 28MB limit
    let args = vec![Val::I32(500)]; // Try to grow by 500 pages
    let result = executor.execute_function(&module_id, "grow_memory", &args);
    assert!(result.is_err() || result.unwrap()[0].i32().unwrap() == -1);
}

// Test fixture generation
#[test]
#[ignore] // Run manually to generate test WASM files
fn generate_test_fixtures() {
    use std::fs;
    use std::path::Path;
    
    let fixtures_dir = Path::new("tests/fixtures/wasm_modules");
    fs::create_dir_all(fixtures_dir).unwrap();
    
    // Generate simple add.wasm
    fs::write(fixtures_dir.join("add.wasm"), ADD_WASM).unwrap();
    
    // Generate other test modules using wat2wasm...
    // This would require the wabt crate or external tools
}

// Property-based tests
#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;
    
    proptest! {
        #[test]
        fn test_wasm_arithmetic_correctness(
            a in any::<i32>(),
            b in any::<i32>()
        ) {
            let config = WasmConfig::default();
            let mut executor = WasmExecutor::new(config).unwrap();
            let module_id = executor.load_module("add", ADD_WASM).unwrap();
            
            let args = vec![Val::I32(a), Val::I32(b)];
            let results = executor.execute_function(&module_id, "add", &args).unwrap();
            
            // Handle overflow the same way WASM does
            let expected = a.wrapping_add(b);
            prop_assert_eq!(results[0].i32(), Some(expected));
        }
    }
}