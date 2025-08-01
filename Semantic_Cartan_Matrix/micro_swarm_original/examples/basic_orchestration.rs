//! Basic micro-swarm orchestration example

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{info, Level};
use tracing_subscriber;

use micro_swarm::{
    SwarmOrchestrator, OrchestratorConfig,
    DashboardServer, DashboardConfig,
    PluginManager,
    Priority,
    AgentFactory,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    info!("Starting micro-swarm orchestration example");

    // Create orchestrator
    let config = OrchestratorConfig {
        name: "example_swarm".to_string(),
        max_agents: 32,
        ..Default::default()
    };
    
    let mut orchestrator = SwarmOrchestrator::new(config);
    let orchestrator_arc = Arc::new(orchestrator);

    // Initialize orchestrator
    // orchestrator.initialize().await?;

    // Create plugin manager
    let plugin_dir = PathBuf::from("./plugins");
    let mut plugin_manager = PluginManager::new(plugin_dir);
    plugin_manager.set_orchestrator(orchestrator_arc.clone());
    
    // Load plugins
    plugin_manager.load_plugins_from_dir().await?;

    // Create dashboard
    let dashboard_config = DashboardConfig {
        port: 8081,
        ..Default::default()
    };
    let dashboard = DashboardServer::new(dashboard_config, orchestrator_arc.clone());

    // Bootstrap agents (commented out due to trait object issues)
    // orchestrator_arc.bootstrap_default_agents().await?;

    info!("Swarm orchestrator initialized with plugins and dashboard");

    // Simulate some work
    info!("Submitting tasks...");
    
    // Submit some example tasks (commented out due to trait object issues)
    // let task1 = orchestrator_arc.submit_task(
    //     "neural_computation".to_string(),
    //     Priority::High,
    //     vec!["neural_inference".to_string()],
    //     serde_json::json!({
    //         "model": "semantic_cartan_matrix",
    //         "input_size": 1024,
    //         "layers": [512, 256, 128]
    //     }),
    // ).await?;

    // let task2 = orchestrator_arc.submit_task(
    //     "quantum_simulation".to_string(),
    //     Priority::Normal,
    //     vec!["quantum_simulation".to_string()],
    //     serde_json::json!({
    //         "qubits": 8,
    //         "gates": ["H", "CNOT", "RZ"],
    //         "depth": 10
    //     }),
    // ).await?;

    // info!("Submitted tasks: {} and {}", task1, task2);

    // Get metrics
    let metrics = orchestrator_arc.get_metrics().await;
    info!("Current metrics: active_agents={}, total_tasks={}", 
          metrics.active_agents, metrics.total_tasks_processed);

    // Export metrics
    let metrics_json = orchestrator_arc.export_metrics_json().await?;
    info!("Exported metrics (first 200 chars): {}", 
          &metrics_json[..std::cmp::min(200, metrics_json.len())]);

    // Check plugin status
    let plugin_status = plugin_manager.get_plugins_status().await?;
    info!("Plugin status: {}", serde_json::to_string_pretty(&plugin_status)?);

    // Start dashboard in background (commented out to avoid blocking)
    // tokio::spawn(async move {
    //     if let Err(e) = dashboard.start().await {
    //         eprintln!("Dashboard error: {}", e);
    //     }
    // });

    info!("Dashboard would be available at http://localhost:8081");

    // Simulate running for a bit
    info!("Running simulation for 10 seconds...");
    sleep(Duration::from_secs(10)).await;

    // Shutdown (commented out due to trait object issues)
    // info!("Shutting down orchestrator...");
    // orchestrator_arc.shutdown().await?;

    info!("Example completed successfully");
    Ok(())
}

/// Demonstrate plugin interaction
async fn demonstrate_plugin_interaction(plugin_manager: &PluginManager) -> Result<(), Box<dyn std::error::Error>> {
    info!("Demonstrating plugin interaction...");

    // Execute RUV-FANN plugin command
    let result = plugin_manager.execute_plugin_command(
        "ruv_fann_integration",
        "get_chip_status".to_string(),
        serde_json::json!({}),
    ).await?;

    info!("RUV-FANN chip status: {}", serde_json::to_string_pretty(&result)?);

    // Execute auto-scaler command
    let result = plugin_manager.execute_plugin_command(
        "auto_scaler",
        "scale_up".to_string(),
        serde_json::json!({}),
    ).await?;

    info!("Auto-scaler response: {}", serde_json::to_string_pretty(&result)?);

    Ok(())
}