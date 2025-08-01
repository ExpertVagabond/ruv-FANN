//! Agent lifecycle management for micro-swarm orchestration

use std::sync::Arc;
use std::time::{Duration, Instant};
use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use tokio::sync::{RwLock, mpsc, oneshot};
use uuid::Uuid;
use tracing::{info, warn, debug};

use crate::{Result, SwarmError};

/// Unique identifier for agents
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AgentId(pub Uuid);

impl AgentId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

/// Agent state machine
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentState {
    /// Agent is initializing
    Initializing,
    /// Agent is ready to accept work
    Ready,
    /// Agent is actively processing
    Running,
    /// Agent is paused
    Paused,
    /// Agent is shutting down
    Stopping,
    /// Agent has terminated
    Terminated,
    /// Agent encountered an error
    Failed(u32), // error code
}

/// Agent capabilities and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInfo {
    pub id: AgentId,
    pub name: String,
    pub agent_type: String,
    pub capabilities: Vec<String>,
    pub max_parallel_tasks: usize,
    pub memory_limit_mb: usize,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Agent metrics for monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMetrics {
    pub tasks_completed: u64,
    pub tasks_failed: u64,
    pub total_execution_time: Duration,
    pub average_task_duration: Duration,
    pub memory_usage_mb: f64,
    pub cpu_usage_percent: f64,
    #[serde(skip, default = "Instant::now")]
    pub last_heartbeat: Instant,
}

/// Message types for agent communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentMessage {
    /// Execute a task
    ExecuteTask {
        task_id: Uuid,
        payload: serde_json::Value,
    },
    /// Update agent state
    UpdateState(AgentState),
    /// Request status
    GetStatus,
    /// Pause execution
    Pause,
    /// Resume execution
    Resume,
    /// Graceful shutdown
    Shutdown,
    /// Emergency stop
    Kill,
}

/// Response from agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentResponse {
    /// Task completed successfully
    TaskCompleted {
        task_id: Uuid,
        result: serde_json::Value,
        duration: Duration,
    },
    /// Task failed
    TaskFailed {
        task_id: Uuid,
        error: String,
    },
    /// Status report
    Status {
        state: AgentState,
        metrics: AgentMetrics,
    },
    /// Acknowledgment
    Ack,
    /// Error response
    Error(String),
}

/// Agent lifecycle trait
#[async_trait]
pub trait AgentLifecycle: Send + Sync {
    /// Initialize the agent
    async fn initialize(&mut self) -> Result<()>;
    
    /// Start the agent
    async fn start(&mut self) -> Result<()>;
    
    /// Handle incoming message
    async fn handle_message(&mut self, msg: AgentMessage) -> Result<AgentResponse>;
    
    /// Pause the agent
    async fn pause(&mut self) -> Result<()>;
    
    /// Resume the agent
    async fn resume(&mut self) -> Result<()>;
    
    /// Stop the agent gracefully
    async fn stop(&mut self) -> Result<()>;
    
    /// Get current state
    fn state(&self) -> AgentState;
    
    /// Get agent info
    fn info(&self) -> &AgentInfo;
    
    /// Get current metrics
    fn metrics(&self) -> &AgentMetrics;
}

/// Base agent implementation
pub struct Agent {
    info: AgentInfo,
    state: Arc<RwLock<AgentState>>,
    metrics: Arc<RwLock<AgentMetrics>>,
    message_tx: mpsc::Sender<(AgentMessage, oneshot::Sender<AgentResponse>)>,
    message_rx: Arc<RwLock<mpsc::Receiver<(AgentMessage, oneshot::Sender<AgentResponse>)>>>,
    shutdown_tx: Option<oneshot::Sender<()>>,
}

impl Agent {
    /// Create a new agent
    pub fn new(
        name: String,
        agent_type: String,
        capabilities: Vec<String>,
        max_parallel_tasks: usize,
        memory_limit_mb: usize,
    ) -> Self {
        let (message_tx, message_rx) = mpsc::channel(1000);
        
        Self {
            info: AgentInfo {
                id: AgentId::new(),
                name,
                agent_type,
                capabilities,
                max_parallel_tasks,
                memory_limit_mb,
                created_at: chrono::Utc::now(),
            },
            state: Arc::new(RwLock::new(AgentState::Initializing)),
            metrics: Arc::new(RwLock::new(AgentMetrics {
                tasks_completed: 0,
                tasks_failed: 0,
                total_execution_time: Duration::ZERO,
                average_task_duration: Duration::ZERO,
                memory_usage_mb: 0.0,
                cpu_usage_percent: 0.0,
                last_heartbeat: Instant::now(),
            })),
            message_tx,
            message_rx: Arc::new(RwLock::new(message_rx)),
            shutdown_tx: None,
        }
    }
    
    /// Send message to agent
    pub async fn send_message(&self, msg: AgentMessage) -> Result<AgentResponse> {
        let (tx, rx) = oneshot::channel();
        self.message_tx
            .send((msg, tx))
            .await
            .map_err(|_| SwarmError::Communication("Failed to send message".into()))?;
        
        rx.await
            .map_err(|_| SwarmError::Communication("Failed to receive response".into()))
    }
    
    /// Update agent metrics
    async fn update_metrics<F>(&self, updater: F)
    where
        F: FnOnce(&mut AgentMetrics),
    {
        let mut metrics = self.metrics.write().await;
        updater(&mut *metrics);
        metrics.last_heartbeat = Instant::now();
    }
    
    /// Get agent health status
    pub async fn health_check(&self) -> bool {
        let metrics = self.metrics.read().await;
        let state = self.state.read().await;
        
        // Check if heartbeat is recent (within 30 seconds)
        let heartbeat_ok = metrics.last_heartbeat.elapsed() < Duration::from_secs(30);
        
        // Check if state is healthy
        let state_ok = matches!(
            *state,
            AgentState::Ready | AgentState::Running | AgentState::Paused
        );
        
        heartbeat_ok && state_ok
    }
}

#[async_trait]
impl AgentLifecycle for Agent {
    async fn initialize(&mut self) -> Result<()> {
        info!("Initializing agent: {} ({})", self.info.name, self.info.id.0);
        
        // Update state
        *self.state.write().await = AgentState::Initializing;
        
        // Perform initialization tasks
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        // Mark as ready
        *self.state.write().await = AgentState::Ready;
        
        info!("Agent initialized: {}", self.info.name);
        Ok(())
    }
    
    async fn start(&mut self) -> Result<()> {
        info!("Starting agent: {}", self.info.name);
        
        // Mark as running
        *self.state.write().await = AgentState::Running;
        
        // In a real implementation, would spawn a proper message handling loop
        // For now, just mark as started
        Ok(())
    }
    
    async fn handle_message(&mut self, msg: AgentMessage) -> Result<AgentResponse> {
        self.send_message(msg).await
    }
    
    async fn pause(&mut self) -> Result<()> {
        self.send_message(AgentMessage::Pause).await?;
        Ok(())
    }
    
    async fn resume(&mut self) -> Result<()> {
        self.send_message(AgentMessage::Resume).await?;
        Ok(())
    }
    
    async fn stop(&mut self) -> Result<()> {
        self.send_message(AgentMessage::Shutdown).await?;
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
        }
        Ok(())
    }
    
    fn state(&self) -> AgentState {
        // Return a default state since we can't do async here
        AgentState::Ready
    }
    
    fn info(&self) -> &AgentInfo {
        &self.info
    }
    
    fn metrics(&self) -> &AgentMetrics {
        // This is a simplified implementation - in practice would need proper async handling
        // For now, return a reference to a lazy static default
        use once_cell::sync::Lazy;
        static DEFAULT_METRICS: Lazy<AgentMetrics> = Lazy::new(|| AgentMetrics {
            tasks_completed: 0,
            tasks_failed: 0,
            total_execution_time: Duration::ZERO,
            average_task_duration: Duration::ZERO,
            memory_usage_mb: 0.0,
            cpu_usage_percent: 0.0,
            last_heartbeat: Instant::now(),
        });
        &DEFAULT_METRICS
    }
}

/// Agent factory for creating specialized agents
pub struct AgentFactory;

impl AgentFactory {
    /// Create a quantum processing agent
    pub fn create_quantum_agent(name: String) -> Agent {
        Agent::new(
            name,
            "quantum".to_string(),
            vec![
                "quantum_simulation".to_string(),
                "entanglement_analysis".to_string(),
                "superposition_compute".to_string(),
            ],
            4, // max parallel tasks
            512, // memory limit MB
        )
    }
    
    /// Create a neural processing agent
    pub fn create_neural_agent(name: String) -> Agent {
        Agent::new(
            name,
            "neural".to_string(),
            vec![
                "neural_inference".to_string(),
                "backpropagation".to_string(),
                "activation_compute".to_string(),
            ],
            8, // max parallel tasks
            1024, // memory limit MB
        )
    }
    
    /// Create a visualization agent
    pub fn create_viz_agent(name: String) -> Agent {
        Agent::new(
            name,
            "visualization".to_string(),
            vec![
                "render_graph".to_string(),
                "generate_heatmap".to_string(),
                "export_metrics".to_string(),
            ],
            2, // max parallel tasks
            256, // memory limit MB
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_agent_lifecycle() {
        let mut agent = Agent::new(
            "test_agent".to_string(),
            "test".to_string(),
            vec!["test_capability".to_string()],
            4,
            128,
        );
        
        // Initialize
        agent.initialize().await.unwrap();
        
        // Start
        agent.start().await.unwrap();
        
        // Send task
        let response = agent.send_message(AgentMessage::ExecuteTask {
            task_id: Uuid::new_v4(),
            payload: serde_json::json!({"test": "data"}),
        }).await.unwrap();
        
        match response {
            AgentResponse::TaskCompleted { .. } => {}
            _ => panic!("Expected TaskCompleted response"),
        }
        
        // Stop
        agent.stop().await.unwrap();
    }
}