# micro_swarm

[![Crates.io](https://img.shields.io/crates/v/micro_swarm.svg)](https://crates.io/crates/micro_swarm)
[![Documentation](https://docs.rs/micro_swarm/badge.svg)](https://docs.rs/micro_swarm)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![Build Status](https://github.com/ruvnet/ruv-FANN/workflows/CI/badge.svg)](https://github.com/ruvnet/ruv-FANN/actions)

**Swarm orchestration and coordination for micro-neural networks**

The `micro_swarm` crate provides high-level orchestration, coordination, and lifecycle management for swarms of micro-neural networks in the Semantic Cartan Matrix system. It handles agent spawning, task scheduling, memory management, and inter-agent communication.

## 🚀 Features

- **Agent Lifecycle Management**: Spawn, monitor, and terminate micro-network agents
- **Parallel Execution**: Multi-threaded task scheduling with work-stealing
- **Memory Pooling**: Efficient allocation and reuse of neural network resources
- **Dashboard Server**: Real-time monitoring with REST API and WebSocket support
- **Plugin Architecture**: Extensible integration with rUv-FANN and other systems
- **Load Balancing**: Dynamic workload distribution across agents
- **Fault Tolerance**: Automatic agent recovery and error handling
- **Cross-Platform**: Native, WebAssembly, and embedded deployment

## 📦 Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
micro_swarm = "0.1.0"
micro_core = "0.1.0"
micro_routing = "0.1.0"

# Optional features
micro_swarm = { version = "0.1.0", features = ["std", "dashboard", "parallel"] }
```

## 🏗️ Core Components

### SwarmOrchestrator

Central coordination hub for all micro-network agents:

```rust
use micro_swarm::{SwarmOrchestrator, SwarmConfig, AgentSpec};
use micro_core::{RootVector, MicroNet};

let config = SwarmConfig {
    max_agents: 64,
    memory_pool_size: 28 * 1024 * 1024, // 28MB pool
    enable_parallel_execution: true,
    load_balancing: true,
    fault_tolerance: true,
};

let mut orchestrator = SwarmOrchestrator::new(config);

// Spawn specialized agents
orchestrator.spawn_agent(AgentSpec {
    agent_type: AgentType::Reasoning,
    name: "primary_reasoner".to_string(),
    capabilities: vec!["logical_inference", "problem_solving"],
    resource_requirements: ResourceRequirements {
        memory_mb: 2,
        cpu_cores: 1,
        priority: Priority::High,
    },
});

// Process inputs through the swarm
let input = RootVector::from_slice(&[0.1, 0.2, /* ... */]);
let result = orchestrator.process(input).await?;

println!("Swarm result: {:?}", result);
```

### Agent Management

Dynamic agent lifecycle with health monitoring:

```rust
use micro_swarm::{Agent, AgentState, HealthCheck};

// Create and configure agents
let mut agent = Agent::new(AgentType::FeatureExtraction, "feature_agent_1");

// Start agent with health monitoring
agent.start().await?;

// Monitor agent health
let health = agent.check_health().await;
match health.state {
    AgentState::Healthy => println!("Agent running normally"),
    AgentState::Degraded => println!("Agent performance degraded: {}", health.message),
    AgentState::Failed => {
        println!("Agent failed: {}", health.message);
        agent.restart().await?;
    }
}

// Graceful shutdown
agent.shutdown().await?;
```

### Task Scheduling

Priority-based task scheduling with parallel execution:

```rust
use micro_swarm::{TaskScheduler, Task, Priority, ExecutionStrategy};

let mut scheduler = TaskScheduler::new(8); // 8 worker threads

// Schedule high-priority task
let task = Task::new()
    .name("urgent_inference")
    .priority(Priority::Critical)
    .input(input_data)
    .target_agents(vec!["reasoning_agent", "feature_agent"])
    .execution_strategy(ExecutionStrategy::Parallel);

let task_id = scheduler.schedule(task).await?;

// Wait for completion
let result = scheduler.wait_for_completion(task_id).await?;

// Or use async processing
let handle = scheduler.schedule_async(task).await?;
tokio::spawn(async move {
    let result = handle.await?;
    process_result(result);
});
```

## 🧠 Agent Types

### Specialized Agent Implementations

The swarm supports multiple agent types with distinct capabilities:

```rust
use micro_swarm::{AgentType, AgentCapabilities};

// Quantum agents for quantum-inspired computation
let quantum_agent = AgentSpec {
    agent_type: AgentType::Quantum,
    capabilities: AgentCapabilities {
        quantum_simulation: true,
        superposition_states: 16,
        entanglement_pairs: 8,
        coherence_time_ms: 100,
    },
    // ... other config
};

// Neural agents for deep learning tasks
let neural_agent = AgentSpec {
    agent_type: AgentType::Neural,
    capabilities: AgentCapabilities {
        layer_count: 12,
        hidden_dim: 768,
        attention_heads: 12,
        max_sequence_length: 2048,
    },
    // ... other config
};

// Visualization agents for real-time monitoring
let viz_agent = AgentSpec {
    agent_type: AgentType::Visualization,
    capabilities: AgentCapabilities {
        render_heatmaps: true,
        real_time_graphs: true,
        metric_aggregation: true,
        export_formats: vec!["png", "svg", "json"],
    },
    // ... other config
};

// Spawn all agent types
orchestrator.spawn_agents(vec![quantum_agent, neural_agent, viz_agent]).await?;
```

### Agent Communication

Inter-agent message passing and coordination:

```rust
use micro_swarm::{Message, MessageType, AgentCommunication};

// Send message between agents
let message = Message {
    from: "reasoning_agent".to_string(),
    to: "feature_agent".to_string(),
    message_type: MessageType::DataTransfer,
    payload: serde_json::to_vec(&root_vector)?,
    priority: Priority::Normal,
};

orchestrator.send_message(message).await?;

// Broadcast to all agents
let broadcast = Message {
    from: "coordinator".to_string(),
    to: "*".to_string(), // Broadcast
    message_type: MessageType::ConfigUpdate,
    payload: serde_json::to_vec(&new_config)?,
    priority: Priority::High,
};

orchestrator.broadcast_message(broadcast).await?;

// Set up message handlers
orchestrator.on_message(|message| async move {
    match message.message_type {
        MessageType::DataTransfer => handle_data_transfer(message).await,
        MessageType::HealthCheck => handle_health_check(message).await,
        MessageType::ConfigUpdate => handle_config_update(message).await,
    }
});
```

## 💾 Memory Management

### Memory Pooling

Efficient memory allocation with region-based management:

```rust
use micro_swarm::{MemoryManager, MemoryRegion, AllocationStrategy};

let memory_config = MemoryConfig {
    total_size: 28 * 1024 * 1024,        // 28MB total
    region_size: 64 * 1024,              // 64KB per region
    alignment: 16,                       // 16-byte alignment for SIMD
    allocation_strategy: AllocationStrategy::BestFit,
    enable_compression: true,
    enable_zero_copy: true,
};

let memory_manager = MemoryManager::new(memory_config);

// Allocate memory for agents
let agent_memory = memory_manager.allocate_for_agent("reasoning_agent", 2048)?;

// Zero-copy data transfer between agents
let data_region = memory_manager.create_shared_region(1024)?;
memory_manager.share_region(&data_region, vec!["agent_1", "agent_2"])?;

// Automatic garbage collection
memory_manager.collect_garbage().await;

// Memory usage statistics
let stats = memory_manager.get_statistics();
println!("Memory usage: {:.1}% ({} MB / {} MB)", 
         stats.usage_percentage, 
         stats.used_mb, 
         stats.total_mb);
```

### Resource Monitoring

Track resource usage across the swarm:

```rust
use micro_swarm::{ResourceMonitor, ResourceType, ResourceThreshold};

let mut monitor = ResourceMonitor::new();

// Set resource thresholds
monitor.set_threshold(ResourceType::Memory, ResourceThreshold {
    warning: 80.0,  // 80% usage warning
    critical: 95.0, // 95% usage critical
});

monitor.set_threshold(ResourceType::Cpu, ResourceThreshold {
    warning: 85.0,
    critical: 98.0,
});

// Monitor continuously
tokio::spawn(async move {
    while let Some(alert) = monitor.check_resources().await {
        match alert.severity {
            Severity::Warning => log::warn!("Resource warning: {}", alert.message),
            Severity::Critical => {
                log::error!("Critical resource alert: {}", alert.message);
                // Take corrective action
                orchestrator.scale_down_agents().await;
            }
        }
    }
});
```

## 📊 Dashboard and Monitoring

### Real-Time Dashboard

Web-based monitoring interface with REST API and WebSocket support:

```rust
use micro_swarm::{DashboardServer, DashboardConfig};

let dashboard_config = DashboardConfig {
    bind_address: "0.0.0.0:8080".to_string(),
    enable_websockets: true,
    enable_cors: true,
    api_rate_limit: 1000, // requests per minute
    auth_enabled: false,  // Disable for development
};

let dashboard = DashboardServer::new(dashboard_config);

// Start dashboard server
dashboard.start().await?;

// Dashboard provides:
// - Real-time agent status and health
// - Performance metrics and graphs
// - Memory usage visualization
// - Task queue monitoring
// - Interactive swarm topology view
// - Export capabilities (JSON, CSV, PNG)

println!("Dashboard available at: http://localhost:8080");
```

### REST API Endpoints

The dashboard exposes comprehensive REST API:

```rust
// GET /api/swarm/status - Overall swarm status
// GET /api/agents - List all agents
// GET /api/agents/{id} - Specific agent details
// GET /api/agents/{id}/health - Agent health check
// POST /api/agents/{id}/restart - Restart agent
// GET /api/tasks - List active tasks
// GET /api/tasks/{id} - Task details
// GET /api/metrics - Performance metrics
// GET /api/memory - Memory usage statistics
// WebSocket /ws/live - Real-time updates

// Example API usage:
use reqwest;

let response = reqwest::get("http://localhost:8080/api/swarm/status").await?;
let status: SwarmStatus = response.json().await?;

println!("Active agents: {}", status.active_agents);
println!("Pending tasks: {}", status.pending_tasks);
println!("Memory usage: {:.1}%", status.memory_usage_percent);
```

### Metrics Export

Export metrics in multiple formats:

```rust
use micro_swarm::{MetricsExporter, ExportFormat};

let exporter = MetricsExporter::new();

// Export as JSON for dashboards
let json_metrics = exporter.export(ExportFormat::Json).await?;

// Export as Prometheus format
let prometheus_metrics = exporter.export(ExportFormat::Prometheus).await?;

// Export as CSV for analysis
let csv_metrics = exporter.export(ExportFormat::Csv).await?;

// Automatic export to external systems
exporter.configure_auto_export(AutoExportConfig {
    interval: Duration::from_secs(60),
    destinations: vec![
        ExportDestination::File("/var/log/swarm_metrics.json".to_string()),
        ExportDestination::Http("http://metrics-collector:9090/api/v1/write".to_string()),
        ExportDestination::Database("postgresql://user:pass@localhost/metrics".to_string()),
    ],
});
```

## 🔧 Integration Points

### rUv-FANN Integration

Seamless integration with the rUv-FANN neuro-synaptic simulator:

```rust
use micro_swarm::{RuvFannPlugin, PluginConfig};

let plugin_config = PluginConfig {
    enable_spike_processing: true,
    enable_plasticity: true,
    chip_interface: ChipInterface::DirectMemory,
    simulation_timestep: Duration::from_micros(100),
};

let ruv_fann_plugin = RuvFannPlugin::new(plugin_config);
orchestrator.add_plugin(Box::new(ruv_fann_plugin)).await?;

// Agents can now interface directly with neuro-synaptic chip
let spike_data = vec![/* spike train data */];
let result = orchestrator.process_spikes(spike_data).await?;
```

### Plugin Architecture

Extensible plugin system for custom integrations:

```rust
use micro_swarm::{Plugin, PluginContext, PluginResult};

struct CustomPlugin {
    name: String,
}

#[async_trait]
impl Plugin for CustomPlugin {
    async fn initialize(&mut self, context: &PluginContext) -> PluginResult<()> {
        println!("Initializing plugin: {}", self.name);
        // Plugin initialization logic
        Ok(())
    }
    
    async fn process(&mut self, data: &[u8]) -> PluginResult<Vec<u8>> {
        // Custom processing logic
        let processed = self.custom_processing(data);
        Ok(processed)
    }
    
    async fn shutdown(&mut self) -> PluginResult<()> {
        println!("Shutting down plugin: {}", self.name);
        Ok(())
    }
}

// Register plugin
let plugin = CustomPlugin { name: "custom_processor".to_string() };
orchestrator.add_plugin(Box::new(plugin)).await?;
```

## ⚡ Performance Optimization

### Load Balancing

Dynamic workload distribution:

```rust
use micro_swarm::{LoadBalancer, LoadBalancingStrategy};

let load_balancer = LoadBalancer::new(LoadBalancingStrategy::WorkStealing);

// Automatically distributes tasks based on:
// - Agent current load
// - Agent capabilities
// - Task priority
// - Historical performance
// - Resource availability

// Manual load balancing
let optimal_agent = load_balancer.select_agent_for_task(&task).await?;
orchestrator.assign_task(task_id, optimal_agent).await?;

// Automatic rebalancing
load_balancer.enable_auto_rebalancing(Duration::from_secs(30));
```

### Parallel Execution

Multi-threaded task processing:

```rust
use micro_swarm::{ParallelExecutor, ExecutionConfig};

let config = ExecutionConfig {
    worker_threads: 8,
    queue_size: 10000,
    batch_size: 32,
    enable_work_stealing: true,
    thread_affinity: true, // Pin threads to cores
};

let executor = ParallelExecutor::new(config);

// Process multiple tasks in parallel
let tasks = vec![task1, task2, task3, task4];
let results = executor.execute_parallel(tasks).await?;

// Pipeline processing
let pipeline_results = executor.execute_pipeline(
    input_data,
    vec!["preprocessor", "feature_extractor", "classifier"]
).await?;
```

## 🧪 Testing and Development

### Integration Testing

Comprehensive testing framework:

```bash
# Test basic orchestration
cargo test test_swarm_orchestration

# Test agent lifecycle
cargo test test_agent_lifecycle --features integration-tests

# Test parallel execution
cargo test test_parallel_execution --features parallel

# Test dashboard API
cargo test test_dashboard_api --features dashboard

# Stress testing
cargo test stress_test_high_load --release -- --ignored
```

### Development Tools

Built-in development and debugging tools:

```rust
use micro_swarm::{SwarmDebugger, DebugConfig};

let debug_config = DebugConfig {
    enable_tracing: true,
    trace_level: TraceLevel::Debug,
    capture_messages: true,
    export_traces: true,
};

let debugger = SwarmDebugger::new(debug_config);
orchestrator.attach_debugger(debugger);

// Debug specific agents
orchestrator.enable_agent_debugging("reasoning_agent", true);

// Trace message flow
orchestrator.trace_message_flow(true);

// Export debug information
let debug_info = orchestrator.export_debug_info().await?;
```

## 🔧 Configuration

### Feature Flags

```toml
[features]
default = ["std", "dashboard"]
std = ["dep:std", "tokio"]
dashboard = ["dep:axum", "dep:serde_json", "dep:tokio-tungstenite"]
parallel = ["dep:rayon", "dep:crossbeam"]
metrics = ["dep:prometheus", "micro_metrics/prometheus"]
plugins = ["dep:libloading"]
quantum = ["dep:qiskit-rust"]
```

### Runtime Configuration

```rust
use micro_swarm::{GlobalConfig, SwarmConfig};

let config = SwarmConfig {
    orchestrator: OrchestratorConfig {
        max_agents: 64,
        agent_timeout: Duration::from_secs(30),
        restart_policy: RestartPolicy::OnFailure,
        load_balancing_algorithm: LoadBalancingAlgorithm::WeightedRoundRobin,
    },
    memory: MemoryConfig {
        pool_size: 28 * 1024 * 1024,
        region_size: 64 * 1024,
        enable_compression: true,
        garbage_collection_interval: Duration::from_secs(60),
    },
    networking: NetworkConfig {
        max_connections: 1000,
        connection_timeout: Duration::from_secs(10),
        enable_tls: false,
        message_queue_size: 10000,
    },
    monitoring: MonitoringConfig {
        metrics_collection_interval: Duration::from_millis(100),
        health_check_interval: Duration::from_secs(30),
        alert_thresholds: Default::default(),
    },
};

GlobalConfig::set_swarm(config);
```

## 📚 Examples

See the [`examples/`](examples/) directory for:

- **Basic Orchestration**: Simple swarm setup and usage
- **Multi-Agent Coordination**: Complex agent interactions
- **Dashboard Integration**: Real-time monitoring setup
- **Plugin Development**: Creating custom plugins
- **Performance Optimization**: Load balancing and parallel execution
- **Integration Examples**: rUv-FANN and other system integrations

## 📄 License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](../LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](../LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## 🔗 Related Crates

- [`micro_core`](../micro_core): Core types and neural network primitives
- [`micro_routing`](../micro_routing): Intelligent agent routing and selection
- [`micro_cartan_attn`](../micro_cartan_attn): Attention mechanisms for coordination
- [`micro_metrics`](../micro_metrics): Performance monitoring and analytics

---

**Part of the rUv-FANN Semantic Cartan Matrix system** 🧠🤖