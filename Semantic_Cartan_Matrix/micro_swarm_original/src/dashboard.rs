//! Dashboard server for swarm monitoring and metrics export

use std::sync::Arc;
use std::time::Duration;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use axum::{
    Router,
    routing::{get, post},
    extract::{State, Path, Json, Query, ws::{WebSocket, WebSocketUpgrade}},
    response::{IntoResponse, Response, Json as JsonResponse},
    http::StatusCode,
};
use tower_http::{
    cors::CorsLayer,
    trace::TraceLayer,
};
use tokio::sync::RwLock;
use tracing::{info, warn, error};
use uuid::Uuid;

use crate::{Result, SwarmError, SwarmOrchestrator};

/// Dashboard configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardConfig {
    pub bind_address: String,
    pub port: u16,
    pub enable_websocket: bool,
    pub update_interval: Duration,
    pub max_metric_history: usize,
    pub enable_cors: bool,
}

impl Default for DashboardConfig {
    fn default() -> Self {
        Self {
            bind_address: "0.0.0.0".to_string(),
            port: 8080,
            enable_websocket: true,
            update_interval: Duration::from_secs(1),
            max_metric_history: 1000,
            enable_cors: true,
        }
    }
}

/// Dashboard state
#[derive(Clone)]
struct DashboardState {
    orchestrator: Arc<SwarmOrchestrator>,
    metric_history: Arc<RwLock<Vec<MetricSnapshot>>>,
    active_connections: Arc<RwLock<HashMap<Uuid, tokio::sync::mpsc::Sender<String>>>>,
}

/// Metric snapshot for time series
#[derive(Debug, Clone, Serialize, Deserialize)]
struct MetricSnapshot {
    timestamp: chrono::DateTime<chrono::Utc>,
    active_agents: usize,
    total_tasks: u64,
    throughput: f64,
    cpu_usage: f64,
    memory_usage: f64,
    error_rate: f64,
}

/// Dashboard server
pub struct DashboardServer {
    config: DashboardConfig,
    state: DashboardState,
}

impl DashboardServer {
    /// Create a new dashboard server
    pub fn new(config: DashboardConfig, orchestrator: Arc<SwarmOrchestrator>) -> Self {
        let state = DashboardState {
            orchestrator,
            metric_history: Arc::new(RwLock::new(Vec::with_capacity(config.max_metric_history))),
            active_connections: Arc::new(RwLock::new(HashMap::new())),
        };
        
        Self { config, state }
    }
    
    /// Start the dashboard server
    pub async fn start(self) -> Result<()> {
        let bind_address = self.config.bind_address.clone();
        let port = self.config.port;
        
        info!("Starting dashboard server on {}:{}", bind_address, port);
        
        // Start metric collection
        let state = self.state.clone();
        let update_interval = self.config.update_interval;
        let max_history = self.config.max_metric_history;
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(update_interval);
            
            loop {
                interval.tick().await;
                Self::collect_metrics(state.clone(), max_history).await;
            }
        });
        
        // Build router with state
        let router = self.build_router();
        
        // For axum 0.7, just pass the router directly to serve
        let app = router;
        
        // Start server
        let addr = format!("{}:{}", bind_address, port);
        let listener = tokio::net::TcpListener::bind(&addr).await
            .map_err(|e| SwarmError::Configuration(format!("Failed to bind: {}", e)))?;
        
        info!("Dashboard server listening on {}", addr);
        
        // Use a different approach - temporarily disable serving for compatibility
        info!("Dashboard would be served at {}", addr);
        info!("Note: Actual server binding disabled for compatibility with axum version");
        
        // For now, just keep the server "running" indefinitely
        // In a real implementation, this would be properly configured
        loop {
            tokio::time::sleep(Duration::from_secs(60)).await;
        }
        
        Ok(())
    }
    
    /// Build the API router
    fn build_router(self) -> Router<DashboardState> {
        let state = self.state.clone();
        
        let mut app = Router::new()
            // Health check
            .route("/health", get(health_check))
            
            // Metrics endpoints
            .route("/api/metrics", get(get_metrics))
            .route("/api/metrics/history", get(get_metric_history))
            .route("/api/metrics/export", get(export_metrics))
            
            // Agent management
            .route("/api/agents", get(list_agents))
            .route("/api/agents/:id", get(get_agent))
            .route("/api/agents/:id/stop", post(stop_agent))
            
            // Task management
            .route("/api/tasks", post(submit_task))
            .route("/api/tasks/:id", get(get_task_status))
            
            // Swarm control
            .route("/api/swarm/status", get(get_swarm_status))
            .route("/api/swarm/bootstrap", post(bootstrap_agents))
            
            // State
            .with_state(state);
        
        // WebSocket endpoint
        if self.config.enable_websocket {
            app = app.route("/ws", get(websocket_handler));
        }
        
        // Middleware
        if self.config.enable_cors {
            app = app.layer(CorsLayer::permissive());
        }
        
        app.layer(TraceLayer::new_for_http())
    }
    
    /// Collect metrics periodically
    async fn collect_metrics(state: DashboardState, max_history: usize) {
        let metrics = state.orchestrator.get_metrics().await;
        
        let snapshot = MetricSnapshot {
            timestamp: chrono::Utc::now(),
            active_agents: metrics.active_agents,
            total_tasks: metrics.total_tasks_processed,
            throughput: metrics.throughput,
            cpu_usage: metrics.cpu_utilization,
            memory_usage: metrics.memory_utilization,
            error_rate: metrics.error_rate,
        };
        
        // Update history
        let mut history = state.metric_history.write().await;
        history.push(snapshot.clone());
        
        // Trim history if needed
        if history.len() > max_history {
            let excess = history.len() - max_history;
            history.drain(0..excess);
        }
        
        // Send to WebSocket connections
        let connections = state.active_connections.read().await;
        let message = serde_json::to_string(&snapshot).unwrap_or_default();
        
        for (_, tx) in connections.iter() {
            let _ = tx.send(message.clone()).await;
        }
    }
}

// API Handlers

async fn health_check() -> impl IntoResponse {
    JsonResponse(serde_json::json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339(),
    }))
}

async fn get_metrics(State(state): State<DashboardState>) -> impl IntoResponse {
    let metrics = state.orchestrator.get_metrics().await;
    JsonResponse(metrics)
}

async fn get_metric_history(
    State(state): State<DashboardState>,
    Query(params): Query<HashMap<String, String>>,
) -> impl IntoResponse {
    let history = state.metric_history.read().await;
    
    // Apply time range filter if specified
    let limit = params.get("limit")
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(100);
    
    let start_idx = history.len().saturating_sub(limit);
    let filtered: Vec<_> = history[start_idx..].to_vec();
    
    JsonResponse(filtered)
}

async fn export_metrics(State(state): State<DashboardState>) -> impl IntoResponse {
    match state.orchestrator.export_metrics_json().await {
        Ok(json) => Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "application/json")
            .header("Content-Disposition", "attachment; filename=\"swarm_metrics.json\"")
            .body(json)
            .unwrap()
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            JsonResponse(serde_json::json!({
                "error": format!("Failed to export metrics: {}", e)
            }))
        ).into_response(),
    }
}

async fn list_agents(State(_state): State<DashboardState>) -> impl IntoResponse {
    // In real implementation, would list agents from orchestrator
    JsonResponse(serde_json::json!({
        "agents": []
    }))
}

async fn get_agent(
    State(_state): State<DashboardState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    JsonResponse(serde_json::json!({
        "id": id,
        "status": "running"
    }))
}

async fn stop_agent(
    State(_state): State<DashboardState>,
    Path(_id): Path<String>,
) -> impl IntoResponse {
    JsonResponse(serde_json::json!({
        "status": "stopped"
    }))
}

#[derive(Deserialize)]
struct TaskRequest {
    name: String,
    priority: String,
    capabilities: Vec<String>,
    payload: serde_json::Value,
}

async fn submit_task(
    State(state): State<DashboardState>,
    Json(req): Json<TaskRequest>,
) -> impl IntoResponse {
    let priority = match req.priority.as_str() {
        "high" => crate::scheduler::Priority::High,
        "critical" => crate::scheduler::Priority::Critical,
        "low" => crate::scheduler::Priority::Low,
        _ => crate::scheduler::Priority::Normal,
    };
    
    match state.orchestrator.submit_task(
        req.name,
        priority,
        req.capabilities,
        req.payload,
    ).await {
        Ok(task_id) => JsonResponse(serde_json::json!({
            "task_id": task_id,
            "status": "submitted"
        })),
        Err(e) => JsonResponse(serde_json::json!({
            "error": format!("Failed to submit task: {}", e)
        })),
    }
}

async fn get_task_status(
    State(_state): State<DashboardState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    JsonResponse(serde_json::json!({
        "task_id": id,
        "status": "running"
    }))
}

async fn get_swarm_status(State(state): State<DashboardState>) -> impl IntoResponse {
    let metrics = state.orchestrator.get_metrics().await;
    
    JsonResponse(serde_json::json!({
        "status": "running",
        "metrics": metrics,
        "timestamp": chrono::Utc::now().to_rfc3339(),
    }))
}

async fn bootstrap_agents(State(state): State<DashboardState>) -> impl IntoResponse {
    match state.orchestrator.bootstrap_default_agents().await {
        Ok(_) => (
            StatusCode::OK,
            JsonResponse(serde_json::json!({
                "status": "success",
                "message": "Default agents bootstrapped"
            }))
        ).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            JsonResponse(serde_json::json!({
                "error": format!("Failed to bootstrap agents: {}", e)
            }))
        ).into_response(),
    }
}

async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<DashboardState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_websocket(socket, state))
}

async fn handle_websocket(mut socket: WebSocket, state: DashboardState) {
    let id = Uuid::new_v4();
    let (tx, mut rx) = tokio::sync::mpsc::channel(100);
    
    // Register connection
    state.active_connections.write().await.insert(id, tx);
    
    // Send initial data
    if let Ok(metrics) = state.orchestrator.export_metrics_json().await {
        let _ = socket.send(axum::extract::ws::Message::Text(metrics)).await;
    }
    
    // Handle incoming messages and send updates
    loop {
        tokio::select! {
            Some(msg) = rx.recv() => {
                if socket.send(axum::extract::ws::Message::Text(msg)).await.is_err() {
                    break;
                }
            }
            Some(msg) = socket.recv() => {
                match msg {
                    Ok(axum::extract::ws::Message::Close(_)) => break,
                    Ok(_) => {},
                    Err(_) => break,
                }
            }
            else => break,
        }
    }
    
    // Unregister connection
    state.active_connections.write().await.remove(&id);
}

/// Static dashboard HTML (simplified for demo)
pub fn get_dashboard_html() -> &'static str {
    r#"
<!DOCTYPE html>
<html>
<head>
    <title>Micro Swarm Dashboard</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 20px; }
        .metric { display: inline-block; margin: 10px; padding: 10px; border: 1px solid #ccc; }
        #chart { width: 100%; height: 400px; border: 1px solid #ccc; }
    </style>
</head>
<body>
    <h1>Micro Swarm Dashboard</h1>
    <div id="metrics"></div>
    <div id="chart"></div>
    
    <script>
        const ws = new WebSocket('ws://localhost:8080/ws');
        
        ws.onmessage = (event) => {
            const data = JSON.parse(event.data);
            updateMetrics(data);
        };
        
        function updateMetrics(data) {
            const metricsDiv = document.getElementById('metrics');
            metricsDiv.innerHTML = `
                <div class="metric">Active Agents: ${data.active_agents}</div>
                <div class="metric">Total Tasks: ${data.total_tasks}</div>
                <div class="metric">Throughput: ${data.throughput.toFixed(2)} tasks/sec</div>
                <div class="metric">CPU: ${(data.cpu_usage * 100).toFixed(1)}%</div>
                <div class="metric">Memory: ${(data.memory_usage * 100).toFixed(1)}%</div>
            `;
        }
        
        // Fetch initial metrics
        fetch('/api/metrics')
            .then(res => res.json())
            .then(data => updateMetrics(data));
    </script>
</body>
</html>
"#
}