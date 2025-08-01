//! Integration tests for micro_swarm orchestration

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::timeout;

use micro_swarm::{
    SwarmOrchestrator, OrchestratorConfig,
    PluginManager,
    Priority,
    MetricsCollector,
};

#[tokio::test]
async fn test_orchestrator_lifecycle() {
    let config = OrchestratorConfig {
        name: "test_swarm".to_string(),
        max_agents: 8,
        ..Default::default()
    };
    
    let orchestrator = SwarmOrchestrator::new(config);
    let orchestrator_arc = Arc::new(orchestrator);
    
    // Test metrics
    let metrics = orchestrator_arc.get_metrics().await;
    assert_eq!(metrics.active_agents, 0);
    
    // Test JSON export
    let json_result = orchestrator_arc.export_metrics_json().await;
    assert!(json_result.is_ok());
    
    let json = json_result.unwrap();
    assert!(json.contains("swarm"));
    assert!(json.contains("test_swarm"));
}

#[tokio::test]
async fn test_plugin_manager() {
    let plugin_dir = PathBuf::from("./test_plugins");
    let plugin_manager = PluginManager::new(plugin_dir);
    
    // Load built-in plugins
    let result = plugin_manager.load_plugins_from_dir().await;
    assert!(result.is_ok());
    
    // Check plugin status
    let status_result = plugin_manager.get_plugins_status().await;
    assert!(status_result.is_ok());
    
    let status = status_result.unwrap();
    assert!(status.contains_key("ruv_fann_integration"));
    assert!(status.contains_key("metrics_exporter"));
    assert!(status.contains_key("auto_scaler"));
}

#[tokio::test]
async fn test_metrics_collector() {
    let collector = MetricsCollector::new();
    
    // Record some metrics
    collector.record("test_metric", 42.0, std::collections::HashMap::new()).await;
    collector.record("test_metric", 43.0, std::collections::HashMap::new()).await;
    
    // Get metrics
    let metric = collector.get("test_metric").await;
    assert!(metric.is_some());
    
    let metric = metric.unwrap();
    assert_eq!(metric.points.len(), 2);
    assert_eq!(metric.points[0].value, 42.0);
    assert_eq!(metric.points[1].value, 43.0);
    
    // Test Prometheus export
    let prometheus = collector.export_prometheus().await;
    assert!(prometheus.contains("test_metric"));
    assert!(prometheus.contains("43"));
}

#[tokio::test]
async fn test_plugin_command_execution() {
    let plugin_dir = PathBuf::from("./test_plugins");
    let plugin_manager = PluginManager::new(plugin_dir);
    
    // Load plugins
    plugin_manager.load_plugins_from_dir().await.unwrap();
    
    // Test RUV-FANN plugin
    let result = plugin_manager.execute_plugin_command(
        "ruv_fann_integration",
        "get_chip_status".to_string(),
        serde_json::json!({}),
    ).await;
    
    assert!(result.is_ok());
    let response = result.unwrap();
    assert!(response.get("total_cores").is_some());
    
    // Test auto-scaler plugin
    let result = plugin_manager.execute_plugin_command(
        "auto_scaler",
        "scale_up".to_string(),
        serde_json::json!({}),
    ).await;
    
    assert!(result.is_ok());
    let response = result.unwrap();
    assert!(response.get("status").is_some());
}

#[tokio::test]
async fn test_orchestrator_integration() {
    let config = OrchestratorConfig {
        name: "integration_test".to_string(),
        max_agents: 16,
        ..Default::default()
    };
    
    let orchestrator = SwarmOrchestrator::new(config);
    let orchestrator_arc = Arc::new(orchestrator);
    
    // Set up plugin manager
    let plugin_dir = PathBuf::from("./test_plugins");
    let mut plugin_manager = PluginManager::new(plugin_dir);
    plugin_manager.set_orchestrator(orchestrator_arc.clone());
    
    // Load plugins
    plugin_manager.load_plugins_from_dir().await.unwrap();
    
    // Test metrics export with plugins
    let metrics_json = orchestrator_arc.export_metrics_json().await.unwrap();
    assert!(metrics_json.contains("integration_test"));
    
    // Test plugin status
    let plugin_status = plugin_manager.get_plugins_status().await.unwrap();
    assert!(!plugin_status.is_empty());
    
    // Verify we can interact with all built-in plugins
    for plugin_name in ["ruv_fann_integration", "metrics_exporter", "auto_scaler"] {
        assert!(plugin_status.contains_key(plugin_name));
        
        let status = &plugin_status[plugin_name];
        assert!(status.get("enabled").is_some());
    }
}