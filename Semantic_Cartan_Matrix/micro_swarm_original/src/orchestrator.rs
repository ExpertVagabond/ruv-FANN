//! Swarm orchestrator - Main coordination layer

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use serde::{Serialize, Deserialize};
use tokio::sync::{RwLock, mpsc};
use uuid::Uuid;
use tracing::{info, warn, error, debug};

use crate::{
    Result, SwarmError,
    Agent, AgentId, AgentState, AgentLifecycle,
    ParallelScheduler, SchedulerConfig, Task, Priority, ExecutionPlan,
    MemoryPool, PoolConfig,
};
use crate::agent::AgentFactory;

/// Swarm orchestrator configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestratorConfig {
    pub name: String,
    pub max_agents: usize,
    pub scheduler_config: SchedulerConfig,
    pub memory_pool_config: PoolConfig,
    pub health_check_interval: Duration,
    pub metrics_export_interval: Duration,
    pub enable_auto_scaling: bool,
    pub enable_fault_tolerance: bool,
}

impl Default for OrchestratorConfig {
    fn default() -> Self {
        Self {
            name: "micro_swarm".to_string(),
            max_agents: 256, // Match chip core count
            scheduler_config: SchedulerConfig::default(),
            memory_pool_config: PoolConfig::default(),
            health_check_interval: Duration::from_secs(10),
            metrics_export_interval: Duration::from_secs(60),
            enable_auto_scaling: true,
            enable_fault_tolerance: true,
        }
    }
}

/// Swarm state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SwarmState {
    Initializing,
    Running,
    Scaling,
    Degraded,
    Stopping,
    Stopped,
}

/// Swarm metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwarmMetrics {
    pub active_agents: usize,
    pub total_tasks_processed: u64,
    pub average_task_latency: Duration,
    pub memory_utilization: f64,
    pub cpu_utilization: f64,
    pub error_rate: f64,
    pub throughput: f64, // tasks per second
}

/// Agent registration info
struct AgentRegistration {
    agent: Box<dyn AgentLifecycle + Send>,
    registration_time: Instant,
    last_health_check: Instant,
    health_status: bool,
}

/// Main swarm orchestrator
pub struct SwarmOrchestrator {
    config: OrchestratorConfig,
    state: Arc<RwLock<SwarmState>>,
    agents: Arc<RwLock<HashMap<AgentId, AgentRegistration>>>,
    scheduler: Arc<RwLock<ParallelScheduler>>,
    memory_pool: Arc<MemoryPool>,
    metrics: Arc<RwLock<SwarmMetrics>>,
    shutdown_tx: Option<mpsc::Sender<()>>,
}

impl SwarmOrchestrator {
    /// Create a new swarm orchestrator
    pub fn new(config: OrchestratorConfig) -> Self {
        let scheduler = ParallelScheduler::new(config.scheduler_config.clone());
        let memory_pool = MemoryPool::new(config.memory_pool_config.clone());
        
        Self {
            config,
            state: Arc::new(RwLock::new(SwarmState::Initializing)),
            agents: Arc::new(RwLock::new(HashMap::new())),
            scheduler: Arc::new(RwLock::new(scheduler)),
            memory_pool: Arc::new(memory_pool),
            metrics: Arc::new(RwLock::new(SwarmMetrics {
                active_agents: 0,
                total_tasks_processed: 0,
                average_task_latency: Duration::ZERO,
                memory_utilization: 0.0,
                cpu_utilization: 0.0,
                error_rate: 0.0,
                throughput: 0.0,
            })),
            shutdown_tx: None,
        }
    }
    
    /// Initialize the swarm
    pub async fn initialize(&mut self) -> Result<()> {
        info!("Initializing swarm orchestrator: {}", self.config.name);
        
        // Initialize memory pool
        self.memory_pool.initialize().await?;
        
        // Start scheduler
        self.scheduler.write().await.start().await?;
        
        // Update state
        *self.state.write().await = SwarmState::Running;
        
        // Start monitoring loops
        self.start_monitoring().await?;
        
        info!("Swarm orchestrator initialized successfully");
        Ok(())
    }
    
    /// Register an agent with the swarm
    pub async fn register_agent(&self, mut agent: Box<dyn AgentLifecycle + Send>) -> Result<AgentId> {
        // Check max agents limit
        let agent_count = self.agents.read().await.len();
        if agent_count >= self.config.max_agents {
            return Err(SwarmError::Configuration("Max agents limit reached".into()));
        }
        
        // Initialize agent
        agent.initialize().await?;
        agent.start().await?;
        
        let agent_id = agent.info().id;
        let capabilities = agent.info().capabilities.clone();
        let max_tasks = agent.info().max_parallel_tasks;
        
        // Register with scheduler
        self.scheduler.write().await
            .register_agent(agent_id, capabilities, max_tasks)
            .await?;
        
        // Add to agents map
        let mut agents = self.agents.write().await;
        agents.insert(agent_id, AgentRegistration {
            agent,
            registration_time: Instant::now(),
            last_health_check: Instant::now(),
            health_status: true,
        });
        
        // Update metrics
        let mut metrics = self.metrics.write().await;
        metrics.active_agents = agents.len();
        
        info!("Registered agent {:?} with swarm", agent_id);
        Ok(agent_id)
    }
    
    /// Unregister an agent
    pub async fn unregister_agent(&self, agent_id: AgentId) -> Result<()> {
        // Remove from agents map
        let mut agents = self.agents.write().await;
        if let Some(mut registration) = agents.remove(&agent_id) {
            // Stop agent
            registration.agent.stop().await?;
            
            // Unregister from scheduler
            self.scheduler.write().await
                .unregister_agent(agent_id)
                .await?;
            
            // Update metrics
            let mut metrics = self.metrics.write().await;
            metrics.active_agents = agents.len();
            
            info!("Unregistered agent {:?} from swarm", agent_id);
            Ok(())
        } else {
            Err(SwarmError::Agent("Agent not found".into()))
        }
    }
    
    /// Submit a task to the swarm
    pub async fn submit_task(
        &self,
        name: String,
        priority: Priority,
        required_capabilities: Vec<String>,
        payload: serde_json::Value,
    ) -> Result<Uuid> {
        let task = Task {
            id: Uuid::new_v4(),
            name,
            priority,
            agent_affinity: None,
            required_capabilities,
            payload,
            dependencies: vec![],
            timeout: Duration::from_secs(300),
            created_at: Instant::now(),
        };
        
        let task_id = task.id;
        self.scheduler.write().await.submit_task(task).await?;
        
        debug!("Submitted task {} to swarm", task_id);
        Ok(task_id)
    }
    
    /// Create and execute an execution plan
    pub async fn execute_plan(&self, tasks: Vec<Task>) -> Result<ExecutionPlan> {
        let plan = self.scheduler.write().await
            .create_execution_plan(tasks.clone())
            .await?;
        
        // Submit all tasks
        self.scheduler.write().await
            .submit_batch(tasks)
            .await?;
        
        Ok(plan)
    }
    
    /// Bootstrap swarm with default agents
    pub async fn bootstrap_default_agents(&self) -> Result<()> {
        info!("Bootstrapping default agents");
        
        // Create quantum agents
        for i in 0..4 {
            let agent = Box::new(AgentFactory::create_quantum_agent(
                format!("quantum_{}", i)
            ));
            self.register_agent(agent).await?;
        }
        
        // Create neural agents
        for i in 0..8 {
            let agent = Box::new(AgentFactory::create_neural_agent(
                format!("neural_{}", i)
            ));
            self.register_agent(agent).await?;
        }
        
        // Create visualization agents
        for i in 0..2 {
            let agent = Box::new(AgentFactory::create_viz_agent(
                format!("viz_{}", i)
            ));
            self.register_agent(agent).await?;
        }
        
        info!("Default agents bootstrapped successfully");
        Ok(())
    }
    
    /// Get swarm metrics
    pub async fn get_metrics(&self) -> SwarmMetrics {
        self.metrics.read().await.clone()
    }
    
    /// Export metrics as JSON
    pub async fn export_metrics_json(&self) -> Result<String> {
        let metrics = self.get_metrics().await;
        let scheduler_stats = self.scheduler.read().await.get_stats().await;
        let memory_stats = self.memory_pool.get_stats().await;
        
        let export = serde_json::json!({
            "swarm": {
                "name": self.config.name,
                "state": format!("{:?}", *self.state.read().await),
                "metrics": metrics,
            },
            "scheduler": scheduler_stats,
            "memory": memory_stats,
            "timestamp": chrono::Utc::now().to_rfc3339(),
        });
        
        Ok(serde_json::to_string_pretty(&export)
            .map_err(|e| SwarmError::Configuration(format!("Failed to export metrics: {}", e)))?)
    }
    
    /// Start monitoring loops
    async fn start_monitoring(&mut self) -> Result<()> {
        let (shutdown_tx, mut shutdown_rx) = mpsc::channel(1);
        self.shutdown_tx = Some(shutdown_tx);
        
        // Health check loop
        let agents = self.agents.clone();
        let state = self.state.clone();
        let health_interval = self.config.health_check_interval;
        let fault_tolerance = self.config.enable_fault_tolerance;
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(health_interval);
            
            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        Self::perform_health_checks(
                            agents.clone(),
                            state.clone(),
                            fault_tolerance,
                        ).await;
                    }
                    _ = shutdown_rx.recv() => {
                        info!("Health check monitor shutting down");
                        break;
                    }
                }
            }
        });
        
        // Metrics export loop
        let metrics = self.metrics.clone();
        let export_interval = self.config.metrics_export_interval;
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(export_interval);
            
            loop {
                interval.tick().await;
                Self::update_metrics(metrics.clone()).await;
                
                // For demo, just run once every interval
                // In practice would have proper shutdown handling
            }
        });
        
        Ok(())
    }
    
    /// Perform health checks on agents
    async fn perform_health_checks(
        agents: Arc<RwLock<HashMap<AgentId, AgentRegistration>>>,
        state: Arc<RwLock<SwarmState>>,
        fault_tolerance: bool,
    ) {
        let mut agents = agents.write().await;
        let mut unhealthy_count = 0;
        let total_count = agents.len();
        
        for (agent_id, registration) in agents.iter_mut() {
            // Simple health check - in real implementation would be more sophisticated
            let health_ok = registration.agent.state() != AgentState::Failed(0);
            
            registration.last_health_check = Instant::now();
            registration.health_status = health_ok;
            
            if !health_ok {
                unhealthy_count += 1;
                warn!("Agent {:?} is unhealthy", agent_id);
                
                if fault_tolerance {
                    // Attempt to restart agent
                    match registration.agent.stop().await {
                        Ok(_) => {
                            match registration.agent.start().await {
                                Ok(_) => info!("Restarted unhealthy agent {:?}", agent_id),
                                Err(e) => error!("Failed to restart agent {:?}: {}", agent_id, e),
                            }
                        }
                        Err(e) => error!("Failed to stop unhealthy agent {:?}: {}", agent_id, e),
                    }
                }
            }
        }
        
        // Update swarm state based on health
        if unhealthy_count > 0 {
            let unhealthy_ratio = unhealthy_count as f64 / total_count as f64;
            if unhealthy_ratio > 0.5 {
                *state.write().await = SwarmState::Degraded;
                error!("Swarm degraded: {}/{} agents unhealthy", unhealthy_count, total_count);
            }
        } else {
            *state.write().await = SwarmState::Running;
        }
    }
    
    /// Update swarm metrics
    async fn update_metrics(metrics: Arc<RwLock<SwarmMetrics>>) {
        // In real implementation, would collect actual metrics
        let mut m = metrics.write().await;
        
        // Simulate metric updates
        m.cpu_utilization = (m.cpu_utilization * 0.9) + (rand::random::<f64>() * 0.1);
        m.memory_utilization = (m.memory_utilization * 0.9) + (rand::random::<f64>() * 0.1);
        m.throughput = m.total_tasks_processed as f64 / 60.0; // tasks per minute
    }
    
    /// Shutdown the swarm
    pub async fn shutdown(&mut self) -> Result<()> {
        info!("Shutting down swarm orchestrator");
        
        *self.state.write().await = SwarmState::Stopping;
        
        // Stop monitoring
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(()).await;
        }
        
        // Stop all agents
        let agents = self.agents.read().await;
        for (agent_id, _) in agents.iter() {
            self.unregister_agent(*agent_id).await?;
        }
        
        // Stop scheduler
        self.scheduler.write().await.stop().await?;
        
        *self.state.write().await = SwarmState::Stopped;
        
        info!("Swarm orchestrator shutdown complete");
        Ok(())
    }
}

// Placeholder for rand - in real implementation would use proper rand crate
mod rand {
    pub fn random<T>() -> T
    where
        T: From<f64>,
    {
        T::from(0.5)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_orchestrator_lifecycle() {
        let mut orchestrator = SwarmOrchestrator::new(OrchestratorConfig::default());
        
        // Initialize
        orchestrator.initialize().await.unwrap();
        
        // Bootstrap agents
        orchestrator.bootstrap_default_agents().await.unwrap();
        
        // Submit task
        let task_id = orchestrator.submit_task(
            "test_task".to_string(),
            Priority::Normal,
            vec!["neural_inference".to_string()],
            serde_json::json!({"test": "data"}),
        ).await.unwrap();
        
        // Get metrics
        let metrics = orchestrator.get_metrics().await;
        assert!(metrics.active_agents > 0);
        
        // Export metrics
        let json = orchestrator.export_metrics_json().await.unwrap();
        assert!(json.contains("swarm"));
        
        // Shutdown
        orchestrator.shutdown().await.unwrap();
    }
}