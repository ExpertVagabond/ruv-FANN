//! Example demonstrating the logging and verification system

use ruv_fann::logging::*;
use std::time::Duration;
use std::collections::HashMap;

fn main() {
    // Create a multi-logger that writes to both JSON and human-readable formats
    let json_logger = JsonLogger::stdout(LoggerConfig {
        min_level: LogLevel::Debug,
        pretty_json: true,
        ..Default::default()
    });
    
    let human_logger = HumanReadableLogger::stderr(LoggerConfig {
        min_level: LogLevel::Info,
        ..Default::default()
    });
    
    let multi_logger = MultiLogger::new(vec![
        Box::new(json_logger),
        Box::new(human_logger),
    ]);
    
    // Initialize global logger
    init_logger(Box::new(multi_logger));
    
    // Create a session
    let session_id = "example-session-001".to_string();
    let builder = EventBuilder::new(session_id.clone());
    
    // Log network initialization
    let network_event = LogEvent::NetworkInit(NetworkInitEvent {
        base: builder.build_base(),
        network_id: "example-network".to_string(),
        topology: NetworkTopology {
            layers: vec![
                LayerInfo {
                    layer_type: "input".to_string(),
                    neurons: 784,
                    activation: "none".to_string(),
                },
                LayerInfo {
                    layer_type: "hidden".to_string(),
                    neurons: 128,
                    activation: "relu".to_string(),
                },
                LayerInfo {
                    layer_type: "output".to_string(),
                    neurons: 10,
                    activation: "softmax".to_string(),
                },
            ],
            total_neurons: 922,
            total_connections: 101_632,
        },
        total_parameters: 101_770,
        estimated_memory_bytes: 407_080,
    });
    
    log(network_event).unwrap();
    
    // Log task start
    let task_start = LogEvent::TaskStart(TaskStartEvent {
        base: builder.build_base(),
        task_name: "training".to_string(),
        task_category: "neural_network".to_string(),
        parameters: {
            let mut params = HashMap::new();
            params.insert("epochs".to_string(), serde_json::json!(10));
            params.insert("batch_size".to_string(), serde_json::json!(32));
            params.insert("learning_rate".to_string(), serde_json::json!(0.001));
            params
        },
        parent_task_id: None,
    });
    
    let task_start_id = if let LogEvent::TaskStart(ref event) = task_start {
        event.base.event_id.clone()
    } else {
        panic!("Expected TaskStart event");
    };
    
    log(task_start).unwrap();
    
    // Log memory allocation
    let mem_alloc = LogEvent::MemoryAlloc(MemoryAllocEvent {
        base: builder.build_base(),
        size_bytes: 1_048_576, // 1MB
        memory_type: "gpu".to_string(),
        purpose: "weight_gradients".to_string(),
        pool_name: Some("training_pool".to_string()),
        total_usage_bytes: 1_048_576,
    });
    
    let alloc_id = if let LogEvent::MemoryAlloc(ref event) = mem_alloc {
        event.base.event_id.clone()
    } else {
        panic!("Expected MemoryAlloc event");
    };
    
    log(mem_alloc).unwrap();
    
    // Log training epochs
    for epoch in 1..=3 {
        let epoch_event = LogEvent::TrainingEpochEnd(TrainingEpochEvent {
            base: builder.build_base(),
            epoch,
            loss: 2.3 / (epoch as f64),
            validation_loss: Some(2.1 / (epoch as f64)),
            metrics: {
                let mut metrics = HashMap::new();
                metrics.insert("accuracy".to_string(), 0.85 + (epoch as f64) * 0.03);
                metrics.insert("precision".to_string(), 0.82 + (epoch as f64) * 0.02);
                metrics
            },
        });
        
        log(epoch_event).unwrap();
    }
    
    // Log GPU operation
    let gpu_op = LogEvent::GpuOperation(GpuOperationEvent {
        base: builder.build_base(),
        operation: "matrix_multiply".to_string(),
        device_id: "gpu:0".to_string(),
        duration: Duration::from_millis(25),
        memory_bytes: 524_288,
        success: true,
    });
    
    log(gpu_op).unwrap();
    
    // Log performance metric
    let metric = LogEvent::PerformanceMetric(PerformanceMetricEvent {
        base: builder.build_base(),
        metric_name: "throughput".to_string(),
        value: 1250.5,
        unit: "samples/sec".to_string(),
        component: "data_loader".to_string(),
        dimensions: HashMap::new(),
    });
    
    log(metric).unwrap();
    
    // Log memory free
    let mem_free = LogEvent::MemoryFree(MemoryFreeEvent {
        base: builder.build_base(),
        size_bytes: 1_048_576,
        memory_type: "gpu".to_string(),
        alloc_event_id: Some(alloc_id),
        total_usage_bytes: 0,
    });
    
    log(mem_free).unwrap();
    
    // Log task end
    let task_end = LogEvent::TaskEnd(TaskEndEvent {
        base: builder.build_base(),
        task_start_id,
        task_name: "training".to_string(),
        status: TaskStatus::Success,
        duration: Duration::from_secs(45),
        result: Some(serde_json::json!({
            "final_loss": 0.767,
            "final_accuracy": 0.91
        })),
        error: None,
    });
    
    log(task_end).unwrap();
    
    // Flush the logger
    flush().unwrap();
    
    println!("\n=== Logger Statistics ===");
    if let Some(stats) = stats() {
        println!("Events logged: {}", stats.events_logged);
        println!("Bytes written: {}", stats.bytes_written);
        println!("Events dropped: {}", stats.events_dropped);
        println!("Errors encountered: {}", stats.errors_encountered);
    }
    
    println!("\n=== Verification Example ===");
    
    // Create some test events for verification
    let mut test_events = vec![];
    
    // Add a memory allocation without corresponding free (potential leak)
    test_events.push(LogEvent::MemoryAlloc(MemoryAllocEvent {
        base: builder.build_base(),
        size_bytes: 2048,
        memory_type: "heap".to_string(),
        purpose: "temp_buffer".to_string(),
        pool_name: None,
        total_usage_bytes: 2048,
    }));
    
    // Add a task start without end (uncompleted task)
    test_events.push(LogEvent::TaskStart(TaskStartEvent {
        base: builder.build_base(),
        task_name: "optimization".to_string(),
        task_category: "training".to_string(),
        parameters: HashMap::new(),
        parent_task_id: None,
    }));
    
    // Add a slow GPU operation
    test_events.push(LogEvent::GpuOperation(GpuOperationEvent {
        base: builder.build_base(),
        operation: "matrix_multiply".to_string(),
        device_id: "gpu:0".to_string(),
        duration: Duration::from_millis(150), // Exceeds threshold
        memory_bytes: 1_048_576,
        success: true,
    }));
    
    // Run verification pipeline
    let mut pipeline = VerificationPipeline::new()
        .add_rule(Box::new(MemoryLeakDetector::new()))
        .add_rule(Box::new(TaskCompletionVerifier::new()))
        .add_rule(Box::new(GpuPerformanceVerifier::new()));
    
    let report = pipeline.verify_events(&test_events);
    
    println!("\nVerification Report:");
    println!("Total events analyzed: {}", report.total_events);
    println!("Overall status: {:?}", report.overall_status);
    
    for rule_report in &report.rule_reports {
        println!("\n{} Report:", rule_report.rule_name);
        println!("  Passed: {}/{}", rule_report.passed_events, rule_report.total_events);
        println!("  Summary: {}", rule_report.summary);
        
        if !rule_report.issues.is_empty() {
            println!("  Issues found:");
            for issue in &rule_report.issues {
                println!("    - [{:?}] {}", issue.severity, issue.message);
            }
        }
    }
}