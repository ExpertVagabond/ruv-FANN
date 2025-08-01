# Micro Swarm Orchestration

A high-performance orchestration layer for coordinating multiple micro-neural networks in the Semantic Cartan Matrix system.

## Features

- **Agent Lifecycle Management**: Complete agent registration, monitoring, and health checking
- **Parallel Execution Scheduler**: Advanced task scheduling with priority queues and load balancing
- **Memory Pooling System**: Efficient inter-agent communication with 28MB shared memory
- **Real-time Dashboard**: Web-based monitoring with JSON metrics export and WebSocket updates
- **Plugin System**: Extensible architecture with built-in RUV-FANN integration

## Architecture

The micro_swarm crate provides several key components:

### 1. Agent Management (`agent.rs`)
- **AgentLifecycle**: Trait for agent behavior
- **AgentFactory**: Creates specialized agents (quantum, neural, visualization)
- **Health Monitoring**: Automatic agent health checks and recovery

### 2. Orchestrator (`orchestrator.rs`)
- **SwarmOrchestrator**: Main coordination layer
- **Bootstrap**: Automatic agent deployment
- **Metrics Export**: JSON metrics for external systems

### 3. Scheduler (`scheduler.rs`)
- **ParallelScheduler**: Task scheduling with priority queues
- **Load Balancing**: Agent workload distribution
- **Execution Plans**: Complex task dependency management

### 4. Memory Pool (`memory.rs`)
- **MemoryPool**: 28MB shared memory management (matching chip spec)
- **Memory Regions**: Allocated memory chunks for agents
- **Zero-copy Transfers**: Efficient inter-agent data movement

### 5. Dashboard (`dashboard.rs`)
- **REST API**: Complete management interface
- **WebSocket**: Real-time metric streaming
- **Web UI**: Built-in monitoring dashboard

### 6. Plugin System (`plugin.rs`)
- **PluginInterface**: Extensible plugin architecture
- **Built-in Plugins**:
  - **RUV-FANN Integration**: Direct neuro-synaptic simulator interface
  - **Metrics Exporter**: Prometheus/JSON/CSV export
  - **Auto-Scaler**: Dynamic agent scaling

## Usage

### Basic Orchestration

```rust
use micro_swarm::{SwarmOrchestrator, OrchestratorConfig, Priority};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create orchestrator
    let config = OrchestratorConfig {
        name: "my_swarm".to_string(),
        max_agents: 256, // Match chip core count
        ..Default::default()
    };
    
    let mut orchestrator = SwarmOrchestrator::new(config);
    orchestrator.initialize().await?;
    
    // Bootstrap default agents
    orchestrator.bootstrap_default_agents().await?;
    
    // Submit task
    let task_id = orchestrator.submit_task(
        "neural_computation".to_string(),
        Priority::High,
        vec!["neural_inference".to_string()],
        serde_json::json!({
            "model": "semantic_cartan_matrix",
            "input_size": 1024
        }),
    ).await?;
    
    // Get metrics
    let metrics = orchestrator.get_metrics().await;
    println!("Active agents: {}", metrics.active_agents);
    
    Ok(())
}
```

### Dashboard Integration

```rust
use micro_swarm::{DashboardServer, DashboardConfig};

// Create dashboard
let dashboard_config = DashboardConfig {
    port: 8080,
    enable_websocket: true,
    ..Default::default()
};

let dashboard = DashboardServer::new(dashboard_config, orchestrator_arc);

// Start dashboard (non-blocking)
tokio::spawn(async move {
    dashboard.start().await.unwrap();
});

// Dashboard available at http://localhost:8080
```

### Plugin System

```rust
use micro_swarm::{PluginManager};
use std::path::PathBuf;

// Create plugin manager
let mut plugin_manager = PluginManager::new(PathBuf::from("./plugins"));
plugin_manager.set_orchestrator(orchestrator_arc);

// Load plugins
plugin_manager.load_plugins_from_dir().await?;

// Execute plugin command
let result = plugin_manager.execute_plugin_command(
    "ruv_fann_integration",
    "run_neural_simulation".to_string(),
    serde_json::json!({
        "cores": 64,
        "memory_mb": 7
    }),
).await?;
```

## Integration with RUV-FANN

The micro_swarm orchestrator is designed to integrate seamlessly with the RUV-FANN neuro-synaptic chip simulator:

### Memory Compatibility
- **28MB Total Memory**: Matches chip specification
- **Region-based Allocation**: Efficient memory management
- **256-Core Support**: Full chip utilization

### Task Distribution
- **Neural Agents**: Handle neural network inference
- **Quantum Agents**: Process quantum computations
- **Visualization Agents**: Generate metrics and graphs

### Performance Metrics
- **Task Throughput**: Real-time performance monitoring
- **Memory Utilization**: Efficient resource usage
- **Agent Health**: Continuous health monitoring

## API Endpoints

The dashboard provides a comprehensive REST API:

### Metrics
- `GET /api/metrics` - Current metrics
- `GET /api/metrics/history` - Historical data
- `GET /api/metrics/export` - JSON export

### Agents
- `GET /api/agents` - List all agents
- `GET /api/agents/:id` - Agent details
- `POST /api/agents/:id/stop` - Stop agent

### Tasks
- `POST /api/tasks` - Submit task
- `GET /api/tasks/:id` - Task status

### Swarm
- `GET /api/swarm/status` - Swarm status
- `POST /api/swarm/bootstrap` - Bootstrap agents

### WebSocket
- `WS /ws` - Real-time metric stream

## Performance

- **Agent Management**: Sub-millisecond agent coordination
- **Task Scheduling**: Priority-based with load balancing
- **Memory Pooling**: Zero-copy transfers between agents
- **Dashboard**: Real-time updates with 1-second intervals
- **Plugin System**: Minimal overhead extensibility

## Configuration

### Orchestrator Config
```rust
OrchestratorConfig {
    name: "swarm_name".to_string(),
    max_agents: 256,
    scheduler_config: SchedulerConfig::default(),
    memory_pool_config: PoolConfig::default(),
    health_check_interval: Duration::from_secs(10),
    enable_auto_scaling: true,
    enable_fault_tolerance: true,
}
```

### Memory Pool Config
```rust
PoolConfig {
    total_size: 28 * 1024 * 1024, // 28MB
    region_size: 64 * 1024,       // 64KB regions
    max_regions_per_agent: 16,
    compression_enabled: true,
    eviction_policy: EvictionPolicy::LRU,
}
```

## Examples

Run the basic orchestration example:

```bash
cd micro_swarm
cargo run --example basic_orchestration
```

## License

MIT License - see LICENSE file for details.

## Contributing

1. Fork the repository
2. Create feature branch
3. Add tests for new functionality
4. Ensure all tests pass
5. Submit pull request

## Integration Points

This crate integrates with:
- **RUV-FANN Simulator**: Direct neuro-synaptic chip interface
- **Quantum Agents**: Quantum computation coordination
- **Neural Networks**: Deep learning model execution
- **Visualization**: Real-time metrics and graph generation