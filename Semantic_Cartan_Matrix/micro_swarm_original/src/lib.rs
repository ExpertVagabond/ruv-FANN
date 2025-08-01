//! Micro-swarm orchestration for neuro-synaptic chip simulation
//! 
//! This crate provides a high-performance orchestration layer for coordinating
//! multiple micro-neural networks in the Semantic Cartan Matrix system.

pub mod agent;
pub mod orchestrator;
pub mod scheduler;
pub mod memory;
pub mod dashboard;
pub mod plugin;
pub mod metrics;

// Re-export main types
pub use agent::{Agent, AgentId, AgentState, AgentLifecycle};
pub use orchestrator::{SwarmOrchestrator, OrchestratorConfig};
pub use scheduler::{ParallelScheduler, SchedulerConfig, ExecutionPlan, Priority, Task};
pub use memory::{MemoryPool, MemoryRegion, PoolConfig};
pub use dashboard::{DashboardServer, DashboardConfig};
pub use plugin::{Plugin, PluginManager, PluginInterface};
pub use metrics::{MetricsCollector, TimeSeries, MetricPoint};

// Error types
#[derive(Debug, thiserror::Error)]
pub enum SwarmError {
    #[error("Agent error: {0}")]
    Agent(String),
    
    #[error("Scheduler error: {0}")]
    Scheduler(String),
    
    #[error("Memory error: {0}")]
    Memory(String),
    
    #[error("Plugin error: {0}")]
    Plugin(String),
    
    #[error("Communication error: {0}")]
    Communication(String),
    
    #[error("Configuration error: {0}")]
    Configuration(String),
}

pub type Result<T> = std::result::Result<T, SwarmError>;