# micro_metrics

[![Crates.io](https://img.shields.io/crates/v/micro_metrics.svg)](https://crates.io/crates/micro_metrics)
[![Documentation](https://docs.rs/micro_metrics/badge.svg)](https://docs.rs/micro_metrics)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![Build Status](https://github.com/ruvnet/ruv-FANN/workflows/CI/badge.svg)](https://github.com/ruvnet/ruv-FANN/actions)

**Performance monitoring and metrics collection for micro-neural networks**

The `micro_metrics` crate provides comprehensive performance monitoring, metrics collection, and analytics for the Semantic Cartan Matrix system. It offers real-time insights into system behavior, resource usage, and neural network performance.

## 🚀 Features

- **Real-Time Monitoring**: Sub-microsecond timing precision
- **Comprehensive Metrics**: Latency, throughput, memory, FLOP counts
- **Dashboard Integration**: JSON export for React/web dashboards
- **Drift Tracking**: Monitor concept drift and performance degradation
- **Cross-Platform Timing**: Accurate timing on native, WASM, and embedded
- **Memory Profiling**: Allocation tracking and leak detection
- **Prometheus Export**: Integration with monitoring infrastructure
- **no_std Compatible**: Works in embedded environments

## 📦 Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
micro_metrics = "0.1.0"

# Optional features
micro_metrics = { version = "0.1.0", features = ["std", "prometheus", "dashboard"] }
```

## 🏗️ Core Components

### MetricsCollector

Central hub for all performance metrics:

```rust
use micro_metrics::{MetricsCollector, MetricsConfig};
use micro_core::RootVector;

let config = MetricsConfig {
    enable_timing: true,
    enable_memory_tracking: true,
    enable_flop_counting: true,
    buffer_size: 10000,
    export_interval: Duration::from_secs(60),
};

let mut collector = MetricsCollector::new(config);

// Collect metrics during operations
let timer = collector.start_timer("inference");
let output = neural_network.forward(&input);
let latency = timer.stop();

// Record custom metrics
collector.record_counter("total_inferences", 1);
collector.record_gauge("memory_usage_mb", 42.5);
collector.record_histogram("batch_size", batch.len() as f64);

// Export metrics for dashboard
let json_export = collector.export_json();
```

### High-Precision Timing

Cross-platform timing with nanosecond precision:

```rust
use micro_metrics::{Timer, TimingConfig};

let config = TimingConfig {
    precision: TimingPrecision::Nanosecond,
    enable_cpu_cycles: true, // x86/ARM only
    enable_rdtsc: true,      // x86 only
};

let mut timer = Timer::new(config);

// Measure operation latency
timer.start();
expensive_operation();
let duration = timer.stop();

println!("Operation took: {:.3} μs", duration.as_micros());

// Measure with automatic recording
let _guard = timer.time_scope("neural_forward");
// Automatically recorded when guard drops
```

### Memory Tracking

Monitor memory allocation and usage patterns:

```rust
use micro_metrics::{MemoryTracker, MemoryConfig};

let config = MemoryConfig {
    track_allocations: true,
    track_deallocations: true,
    detect_leaks: true,
    sample_rate: 0.01, // Sample 1% of allocations
};

let mut tracker = MemoryTracker::new(config);

// Track memory usage
tracker.start_tracking();

let vectors = vec![RootVector::zeros(); 1000];
let allocation_size = tracker.get_current_usage();

drop(vectors);
let final_size = tracker.get_current_usage();

println!("Peak memory: {} KB", tracker.get_peak_usage() / 1024);
println!("Current usage: {} KB", final_size / 1024);

// Generate memory report
let report = tracker.generate_report();
```

## 📊 Metrics Types

### Performance Metrics

Track system performance across multiple dimensions:

```rust
use micro_metrics::{PerformanceMetrics, MetricType};

let mut metrics = PerformanceMetrics::new();

// Latency metrics
metrics.record(MetricType::Latency, "inference", 1.2); // milliseconds
metrics.record(MetricType::Latency, "routing", 0.05);

// Throughput metrics  
metrics.record(MetricType::Throughput, "tokens_per_second", 15420.0);
metrics.record(MetricType::Throughput, "inferences_per_second", 850.0);

// Resource utilization
metrics.record(MetricType::CpuUsage, "core_0", 85.2);
metrics.record(MetricType::MemoryUsage, "heap", 42.8);
metrics.record(MetricType::GpuUsage, "utilization", 67.3);

// FLOP counting
metrics.record(MetricType::FlopCount, "matrix_multiply", 2048000);
metrics.record(MetricType::FlopCount, "attention", 1536000);

// Get aggregated statistics
let latency_stats = metrics.get_statistics(MetricType::Latency);
println!("Mean latency: {:.3} ms", latency_stats.mean);
println!("P95 latency: {:.3} ms", latency_stats.percentile_95);
```

### Agent-Specific Metrics

Track performance per micro-network agent:

```rust
use micro_metrics::{AgentMetrics, AgentId};

let mut agent_metrics = AgentMetrics::new();

// Record agent-specific performance
agent_metrics.record_inference("reasoning_agent", 2.1, true); // latency, success
agent_metrics.record_inference("feature_agent", 0.8, true);
agent_metrics.record_inference("routing_agent", 0.1, true);

// Get agent statistics
let reasoning_stats = agent_metrics.get_agent_stats("reasoning_agent");
println!("Success rate: {:.1}%", reasoning_stats.success_rate * 100.0);
println!("Average latency: {:.3} ms", reasoning_stats.avg_latency);
println!("Total invocations: {}", reasoning_stats.total_invocations);

// Identify performance bottlenecks
let bottlenecks = agent_metrics.identify_bottlenecks();
for bottleneck in bottlenecks {
    println!("Bottleneck: {} ({:.3} ms avg)", bottleneck.agent_id, bottleneck.avg_latency);
}
```

## 🔍 Drift Tracking

Monitor concept drift and performance degradation:

```rust
use micro_metrics::{DriftTracker, DriftConfig, DriftMetric};

let config = DriftConfig {
    window_size: 1000,          // Track last 1000 samples
    drift_threshold: 0.05,      // 5% change triggers alert
    methods: vec![
        DriftMetric::MeanShift,
        DriftMetric::VarianceChange,
        DriftMetric::DistributionDistance,
    ],
};

let mut drift_tracker = DriftTracker::new(config);

// Track outputs over time
for output in neural_outputs {
    drift_tracker.add_sample(output);
    
    // Check for drift
    if let Some(drift_detected) = drift_tracker.check_drift() {
        println!("Drift detected: {:?}", drift_detected);
        println!("Confidence: {:.3}", drift_detected.confidence);
        
        // Take corrective action
        if drift_detected.confidence > 0.8 {
            retrain_model();
        }
    }
}

// Get drift analysis
let drift_report = drift_tracker.generate_report();
```

### Root Space Drift

Monitor orthogonality drift in the 32-dimensional root space:

```rust
use micro_metrics::{RootSpaceDriftTracker, OrthogonalityConfig};
use micro_core::RootVector;

let config = OrthogonalityConfig {
    tolerance: 1e-6,
    check_interval: 100,
    store_history: true,
};

let mut drift_tracker = RootSpaceDriftTracker::new(config);

// Monitor root space vectors
let root_vectors = vec![
    RootVector::from_slice(&[1.0, 0.0, 0.0, /* ... */]),
    RootVector::from_slice(&[0.0, 1.0, 0.0, /* ... */]),
    // ... more vectors
];

let orthogonality_score = drift_tracker.check_orthogonality(&root_vectors);

if orthogonality_score < 0.95 {
    println!("Warning: Orthogonality degraded to {:.3}", orthogonality_score);
    
    // Trigger re-orthogonalization
    reorthogonalize_vectors(&mut root_vectors);
}
```

## 📱 Dashboard Integration

### JSON Export

Export metrics in dashboard-ready format:

```rust
use micro_metrics::{DashboardExporter, ExportConfig};

let config = ExportConfig {
    format: ExportFormat::Json,
    include_metadata: true,
    compress: true,
    anonymize_sensitive: true,
};

let exporter = DashboardExporter::new(config);

// Export comprehensive metrics
let dashboard_data = exporter.export(&metrics_collector);

// dashboard_data contains:
// - Real-time performance metrics
// - Agent utilization heatmaps
// - Attention matrices for visualization
// - Resource usage graphs
// - Drift detection alerts

// Send to React dashboard
send_to_dashboard(&dashboard_data);
```

### WebSocket Streaming

Real-time metrics streaming:

```rust
use micro_metrics::{MetricsStreamer, StreamConfig};

let config = StreamConfig {
    update_interval: Duration::from_millis(100),
    buffer_size: 1000,
    compression: CompressionType::Gzip,
};

let streamer = MetricsStreamer::new(config);

// Stream metrics to dashboard
tokio::spawn(async move {
    let mut stream = streamer.create_stream().await;
    
    while let Some(metrics_update) = stream.next().await {
        broadcast_to_clients(metrics_update).await;
    }
});
```

### Heatmap Generation

Generate attention and correlation heatmaps:

```rust
use micro_metrics::{HeatmapGenerator, HeatmapConfig};

let config = HeatmapConfig {
    dimensions: (32, 32),      // 32x32 root space
    color_scheme: ColorScheme::Viridis,
    normalize: true,
    interpolation: InterpolationType::Bilinear,
};

let generator = HeatmapGenerator::new(config);

// Generate attention heatmap
let attention_matrix = get_attention_weights();
let heatmap_data = generator.generate_heatmap(&attention_matrix);

// Export for dashboard visualization
let heatmap_json = serde_json::to_string(&heatmap_data).unwrap();
```

## 🔧 Integration Interfaces

### Prometheus Export

Export metrics to Prometheus for monitoring infrastructure:

```rust
use micro_metrics::{PrometheusExporter, PrometheusConfig};

let config = PrometheusConfig {
    namespace: "cartan_matrix",
    job_name: "micro_neural_network",
    push_gateway: Some("http://prometheus-gateway:9091".to_string()),
    push_interval: Duration::from_secs(15),
};

let exporter = PrometheusExporter::new(config);

// Register custom metrics
exporter.register_counter("inferences_total", "Total number of inferences");
exporter.register_histogram("inference_duration_seconds", "Inference latency distribution");
exporter.register_gauge("active_agents", "Number of active agents");

// Export metrics
exporter.export(&metrics_collector);
```

### Custom Backends

Implement custom metrics backends:

```rust
use micro_metrics::{MetricsBackend, MetricsData};

struct CustomBackend {
    endpoint: String,
}

impl MetricsBackend for CustomBackend {
    fn export(&self, data: &MetricsData) -> Result<(), Error> {
        // Custom export logic
        let json_data = serde_json::to_string(data)?;
        self.send_to_custom_endpoint(&json_data)
    }
    
    fn health_check(&self) -> bool {
        // Check backend connectivity
        self.ping_endpoint()
    }
}

// Use custom backend
let backend = CustomBackend { endpoint: "https://api.example.com/metrics".to_string() };
metrics_collector.add_backend(Box::new(backend));
```

## 📈 Performance Analysis

### Benchmarking Integration

Integration with Rust's criterion benchmarking:

```rust
use micro_metrics::{BenchmarkIntegration, BenchmarkConfig};
use criterion::{Criterion, BenchmarkId};

let config = BenchmarkConfig {
    capture_metrics: true,
    export_results: true,
    compare_baselines: true,
};

let benchmark_integration = BenchmarkIntegration::new(config);

// Enhanced benchmarking with metrics
fn bench_inference(c: &mut Criterion) {
    let mut group = c.benchmark_group("inference");
    
    for size in [32, 64, 128, 256].iter() {
        group.bench_with_input(
            BenchmarkId::new("cartan_attention", size),
            size,
            |b, &size| {
                let metrics = benchmark_integration.start_benchmark();
                
                b.iter(|| {
                    // Benchmark code with automatic metrics collection
                    attention_layer.forward(&input);
                });
                
                benchmark_integration.finish_benchmark(metrics);
            }
        );
    }
}
```

### Regression Detection

Automatic performance regression detection:

```rust
use micro_metrics::{RegressionDetector, RegressionConfig};

let config = RegressionConfig {
    baseline_window: 100,
    detection_threshold: 0.1, // 10% regression
    min_samples: 50,
    significance_level: 0.05,
};

let detector = RegressionDetector::new(config);

// Monitor for performance regressions
for measurement in performance_measurements {
    detector.add_measurement(measurement);
    
    if let Some(regression) = detector.check_regression() {
        println!("Performance regression detected!");
        println!("Current: {:.3} ms, Baseline: {:.3} ms", 
                regression.current_value, regression.baseline_value);
        println!("Regression: {:.1}%", regression.percentage * 100.0);
        
        // Alert or rollback
        send_alert(&regression);
    }
}
```

## 🧪 Testing and Validation

### Metrics Testing

Validate metrics collection accuracy:

```bash
# Test timing accuracy
cargo test test_timing_accuracy

# Test memory tracking
cargo test test_memory_tracking --features std

# Test cross-platform compatibility
cargo test --target wasm32-unknown-unknown

# Benchmark metrics overhead
cargo bench metrics_overhead
```

### Property-Based Testing

```rust
use micro_metrics::testing::*;
use quickcheck::TestResult;

#[quickcheck]
fn test_timing_monotonicity(operations: Vec<u32>) -> TestResult {
    let mut timer = Timer::new(Default::default());
    
    let mut last_time = 0;
    for op_duration in operations {
        timer.start();
        std::thread::sleep(Duration::from_nanos(op_duration as u64));
        let measured = timer.stop().as_nanos();
        
        // Timer should be monotonic
        if measured < last_time {
            return TestResult::failed();
        }
        last_time = measured;
    }
    
    TestResult::passed()
}
```

## 🔧 Configuration

### Feature Flags

```toml
[features]
default = ["timing"]
std = ["dep:std"]
timing = []
memory-tracking = ["dep:tikv-jemalloc-ctl"]
prometheus = ["dep:prometheus"]
dashboard = ["dep:serde_json", "dep:tokio"]
drift-detection = ["dep:statrs"]
```

### Runtime Configuration

```rust
use micro_metrics::{GlobalConfig, MetricsConfig};

let config = MetricsConfig {
    default_precision: TimingPrecision::Microsecond,
    enable_sampling: true,
    sample_rate: 0.1,
    max_memory_usage: 100 * 1024 * 1024, // 100MB
    export_batch_size: 1000,
    retention_period: Duration::from_hours(24),
};

GlobalConfig::set_metrics(config);
```

## 📚 Examples

See the [`examples/`](examples/) directory for:

- **Basic Monitoring**: Simple metrics collection
- **Dashboard Integration**: Real-time web dashboard
- **Drift Detection**: Monitoring concept drift
- **Custom Backends**: Implementing custom metrics exporters
- **Performance Analysis**: Regression detection and benchmarking

## 📄 License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](../LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](../LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## 🔗 Related Crates

- [`micro_core`](../micro_core): Core types and performance-critical operations
- [`micro_routing`](../micro_routing): Routing performance monitoring
- [`micro_cartan_attn`](../micro_cartan_attn): Attention mechanism analytics
- [`micro_swarm`](../micro_swarm): Swarm-level orchestration metrics

---

**Part of the rUv-FANN Semantic Cartan Matrix system** 🧠📊