use neuro_synaptic_simulator::logging::{Logger, LogEvent, SimulationResult, LogConfig};
use serde_json::{json, Value};
use std::io::Cursor;
use std::sync::Arc;
use std::thread;
use std::time::SystemTime;

#[test]
fn test_logger_creation() {
    let config = LogConfig {
        format: LogFormat::Json,
        level: LogLevel::Info,
        buffer_size: 1000,
    };
    
    let logger = Logger::new(config);
    assert_eq!(logger.event_count(), 0);
}

#[test]
fn test_log_event_creation() {
    let event = LogEvent::TaskStart {
        time_ns: 1000,
        core_id: 42,
        task_id: "test_task".to_string(),
    };
    
    match event {
        LogEvent::TaskStart { time_ns, core_id, task_id } => {
            assert_eq!(time_ns, 1000);
            assert_eq!(core_id, 42);
            assert_eq!(task_id, "test_task");
        }
        _ => panic!("Wrong event type"),
    }
}

#[test]
fn test_json_serialization() {
    let event = LogEvent::TaskComplete {
        time_ns: 2000,
        core_id: 0,
        task_id: "task1".to_string(),
        cycles: 1_000_000,
        instructions: 950_000,
    };
    
    let json = serde_json::to_string(&event).unwrap();
    let parsed: Value = serde_json::from_str(&json).unwrap();
    
    assert_eq!(parsed["type"], "TaskComplete");
    assert_eq!(parsed["time_ns"], 2000);
    assert_eq!(parsed["core_id"], 0);
    assert_eq!(parsed["cycles"], 1_000_000);
}

#[test]
fn test_logger_event_recording() {
    let config = LogConfig::default();
    let mut logger = Logger::new(config);
    
    // Record various events
    logger.record(LogEvent::TaskStart {
        time_ns: 0,
        core_id: 0,
        task_id: "task1".to_string(),
    });
    
    logger.record(LogEvent::MemoryAllocation {
        time_ns: 100,
        offset: 0x1000,
        size: 4096,
        purpose: "weights".to_string(),
    });
    
    logger.record(LogEvent::TaskComplete {
        time_ns: 1000,
        core_id: 0,
        task_id: "task1".to_string(),
        cycles: 900,
        instructions: 850,
    });
    
    assert_eq!(logger.event_count(), 3);
}

#[test]
fn test_simulation_result_json() {
    let mut logger = Logger::new(LogConfig::default());
    
    // Add some events
    logger.record(LogEvent::TaskStart {
        time_ns: 0,
        core_id: 0,
        task_id: "inference".to_string(),
    });
    
    logger.record(LogEvent::TaskComplete {
        time_ns: 5000,
        core_id: 0,
        task_id: "inference".to_string(),
        cycles: 4500,
        instructions: 4000,
    });
    
    // Create simulation result
    let result = SimulationResult {
        model_name: "test_network.wasm".to_string(),
        cores_active: 1,
        total_time_us: 5,
        energy_mj: 0.01,
        throughput_gips: 0.8,
        events: logger.events().to_vec(),
        timestamp: SystemTime::now(),
    };
    
    // Serialize to JSON
    let json = serde_json::to_string_pretty(&result).unwrap();
    
    // Parse and verify
    let parsed: Value = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed["simulation"]["model"], "test_network.wasm");
    assert_eq!(parsed["simulation"]["cores_active"], 1);
    assert_eq!(parsed["simulation"]["time_us"], 5);
    assert_eq!(parsed["simulation"]["events"].as_array().unwrap().len(), 2);
}

#[test]
fn test_logger_human_readable_output() {
    let mut config = LogConfig::default();
    config.format = LogFormat::Human;
    
    let mut logger = Logger::new(config);
    let mut output = Cursor::new(Vec::new());
    
    logger.record(LogEvent::TaskStart {
        time_ns: 0,
        core_id: 0,
        task_id: "test".to_string(),
    });
    
    logger.write_human_readable(&mut output).unwrap();
    
    let output_str = String::from_utf8(output.into_inner()).unwrap();
    assert!(output_str.contains("Core 0"));
    assert!(output_str.contains("START"));
    assert!(output_str.contains("test"));
}

#[test]
fn test_logger_thread_safety() {
    let logger = Arc::new(Logger::new(LogConfig::default()));
    let mut handles = vec![];
    
    // Multiple threads logging concurrently
    for thread_id in 0..10 {
        let logger_clone = logger.clone();
        
        let handle = thread::spawn(move || {
            for i in 0..100 {
                logger_clone.record(LogEvent::TaskStart {
                    time_ns: i * 1000,
                    core_id: thread_id,
                    task_id: format!("task_{}_{}", thread_id, i),
                });
            }
        });
        
        handles.push(handle);
    }
    
    for handle in handles {
        handle.join().unwrap();
    }
    
    // Should have recorded all events
    assert_eq!(logger.event_count(), 1000);
}

#[test]
fn test_error_event_logging() {
    let mut logger = Logger::new(LogConfig::default());
    
    logger.record(LogEvent::Error {
        time_ns: 1000,
        core_id: Some(5),
        error_type: "OutOfMemory".to_string(),
        message: "Failed to allocate 100MB".to_string(),
    });
    
    logger.record(LogEvent::Error {
        time_ns: 2000,
        core_id: None,
        error_type: "WasmTrap".to_string(),
        message: "Integer overflow in module".to_string(),
    });
    
    let events = logger.events();
    assert_eq!(events.len(), 2);
    
    // Verify JSON includes error details
    let json = serde_json::to_string(&events[0]).unwrap();
    assert!(json.contains("OutOfMemory"));
}

#[test]
fn test_performance_event_logging() {
    let mut logger = Logger::new(LogConfig::default());
    
    logger.record(LogEvent::PerformanceMetric {
        time_ns: 10_000,
        metric_name: "cache_hit_rate".to_string(),
        value: 0.95,
        unit: "ratio".to_string(),
    });
    
    logger.record(LogEvent::PerformanceMetric {
        time_ns: 20_000,
        metric_name: "memory_bandwidth".to_string(),
        value: 25.6,
        unit: "GB/s".to_string(),
    });
    
    // Export as metrics
    let metrics = logger.export_metrics();
    assert_eq!(metrics.len(), 2);
    assert_eq!(metrics[0].0, "cache_hit_rate");
    assert_eq!(metrics[0].1, 0.95);
}

#[test]
fn test_log_filtering() {
    let mut config = LogConfig::default();
    config.level = LogLevel::Warning;
    
    let mut logger = Logger::new(config);
    
    // These should be logged
    logger.record_with_level(LogLevel::Error, LogEvent::Error {
        time_ns: 1000,
        core_id: None,
        error_type: "Critical".to_string(),
        message: "System failure".to_string(),
    });
    
    logger.record_with_level(LogLevel::Warning, LogEvent::TaskComplete {
        time_ns: 2000,
        core_id: 0,
        task_id: "slow_task".to_string(),
        cycles: 10_000_000,
        instructions: 9_000_000,
    });
    
    // This should be filtered out
    logger.record_with_level(LogLevel::Info, LogEvent::TaskStart {
        time_ns: 0,
        core_id: 0,
        task_id: "normal_task".to_string(),
    });
    
    assert_eq!(logger.event_count(), 2);
}

#[test]
fn test_log_event_ordering() {
    let mut logger = Logger::new(LogConfig::default());
    
    // Add events out of order
    logger.record(LogEvent::TaskComplete {
        time_ns: 5000,
        core_id: 0,
        task_id: "task1".to_string(),
        cycles: 4000,
        instructions: 3800,
    });
    
    logger.record(LogEvent::TaskStart {
        time_ns: 0,
        core_id: 0,
        task_id: "task1".to_string(),
    });
    
    logger.record(LogEvent::TaskStart {
        time_ns: 2500,
        core_id: 1,
        task_id: "task2".to_string(),
    });
    
    // Sort by timestamp
    logger.sort_events();
    
    let events = logger.events();
    assert_eq!(events[0].timestamp(), 0);
    assert_eq!(events[1].timestamp(), 2500);
    assert_eq!(events[2].timestamp(), 5000);
}

#[test]
fn test_log_validation() {
    let result = SimulationResult {
        model_name: "test.wasm".to_string(),
        cores_active: 256,
        total_time_us: 1000,
        energy_mj: 2.0,
        throughput_gips: 5.0,
        events: vec![
            LogEvent::TaskStart {
                time_ns: 0,
                core_id: 0,
                task_id: "task1".to_string(),
            },
            LogEvent::TaskComplete {
                time_ns: 500_000,
                core_id: 0,
                task_id: "task1".to_string(),
                cycles: 450_000,
                instructions: 400_000,
            },
        ],
        timestamp: SystemTime::now(),
    };
    
    // Validate results
    assert!(result.validate().is_ok());
    
    // Test invalid result (task completes before it starts)
    let invalid_result = SimulationResult {
        events: vec![
            LogEvent::TaskComplete {
                time_ns: 100,
                core_id: 0,
                task_id: "task1".to_string(),
                cycles: 50,
                instructions: 45,
            },
            LogEvent::TaskStart {
                time_ns: 200,
                core_id: 0,
                task_id: "task1".to_string(),
            },
        ],
        ..result
    };
    
    assert!(invalid_result.validate().is_err());
}

#[test]
fn test_log_compression() {
    let config = LogConfig {
        format: LogFormat::CompressedJson,
        level: LogLevel::Info,
        buffer_size: 10000,
    };
    
    let mut logger = Logger::new(config);
    
    // Add many events
    for i in 0..1000 {
        logger.record(LogEvent::TaskStart {
            time_ns: i * 100,
            core_id: (i % 256) as u8,
            task_id: format!("task_{}", i),
        });
    }
    
    // Export compressed
    let compressed = logger.export_compressed().unwrap();
    let uncompressed = logger.export_json().unwrap();
    
    // Compressed should be smaller
    assert!(compressed.len() < uncompressed.len() / 2);
}

// Property-based tests
#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;
    
    proptest! {
        #[test]
        fn test_event_serialization_roundtrip(
            time_ns in any::<u64>(),
            core_id in 0u8..=255,
            task_id in "[a-z0-9_]{1,20}",
            cycles in any::<u64>(),
            instructions in any::<u64>()
        ) {
            let event = LogEvent::TaskComplete {
                time_ns,
                core_id,
                task_id: task_id.clone(),
                cycles,
                instructions,
            };
            
            // Serialize and deserialize
            let json = serde_json::to_string(&event).unwrap();
            let deserialized: LogEvent = serde_json::from_str(&json).unwrap();
            
            // Should match
            match deserialized {
                LogEvent::TaskComplete { 
                    time_ns: t, 
                    core_id: c, 
                    task_id: tid, 
                    cycles: cy, 
                    instructions: i 
                } => {
                    prop_assert_eq!(t, time_ns);
                    prop_assert_eq!(c, core_id);
                    prop_assert_eq!(tid, task_id);
                    prop_assert_eq!(cy, cycles);
                    prop_assert_eq!(i, instructions);
                }
                _ => panic!("Wrong event type after deserialization"),
            }
        }
    }
}