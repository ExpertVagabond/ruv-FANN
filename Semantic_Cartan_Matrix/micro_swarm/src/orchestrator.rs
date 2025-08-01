//! Main swarm orchestrator

use alloc::{vec::Vec, boxed::Box, string::String};
use micro_core::{RootVector, MicroNet, Result, Error};
use micro_routing::{Router, RouterConfig, Context, ContextManager};
use micro_cartan_attn::{CartanAttention, CartanMatrix, AttentionConfig};
use micro_metrics::{MetricsCollector, Timer};
use crate::{MemoryManager, TaskScheduler, SwarmCoordinator};

#[cfg(feature = "serde")]
use serde::{Serialize, Deserialize};

/// Configuration for the swarm orchestrator
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SwarmConfig {
    /// Maximum number of micro-nets in the swarm
    pub max_agents: usize,
    
    /// Memory pool size for vectors
    pub memory_pool_size: usize,
    
    /// Router configuration
    pub router_config: RouterConfig,
    
    /// Attention configuration
    pub attention_config: AttentionConfig,
    
    /// Whether to enable parallel execution
    pub enable_parallel: bool,
    
    /// Whether to collect detailed metrics
    pub collect_metrics: bool,
    
    /// Session ID for context management
    pub session_id: String,
}

impl Default for SwarmConfig {
    fn default() -> Self {
        Self {
            max_agents: 16,
            memory_pool_size: 1024,
            router_config: RouterConfig::default(),
            attention_config: AttentionConfig::default(),
            enable_parallel: true,
            collect_metrics: true,
            session_id: "default".to_string(),
        }
    }
}

/// Current state of the swarm
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SwarmState {
    /// Number of active agents
    pub active_agents: usize,
    
    /// Memory utilization (0.0 to 1.0)
    pub memory_utilization: f32,
    
    /// Last processing time in milliseconds
    pub last_processing_time_ms: f64,
    
    /// Total number of processed inputs
    pub total_processed: u64,
    
    /// Current Cartan violation level
    pub cartan_violation: f32,
}

/// Main orchestrator for micro-network swarms
pub struct SwarmOrchestrator {
    /// Configuration
    config: SwarmConfig,
    
    /// Router for micro-net selection
    router: Router,
    
    /// Cartan attention mechanism
    attention: CartanAttention,
    
    /// Memory manager
    memory: MemoryManager,
    
    /// Task scheduler
    scheduler: TaskScheduler,
    
    /// Swarm coordinator for distributed operations
    coordinator: SwarmCoordinator,
    
    /// Context manager for session state
    context_manager: ContextManager,
    
    /// Metrics collector
    metrics: MetricsCollector,
    
    /// Current swarm state
    state: SwarmState,
}

impl SwarmOrchestrator {
    /// Create a new swarm orchestrator
    pub fn new(config: SwarmConfig) -> Result<Self> {
        // Initialize Cartan matrix (identity by default)
        let cartan_matrix = CartanMatrix::identity();
        
        // Create components
        let router = Router::new(config.router_config.clone());
        let attention = CartanAttention::new(config.attention_config.clone(), cartan_matrix);
        let memory = MemoryManager::new(config.memory_pool_size);
        let scheduler = TaskScheduler::new(config.enable_parallel);
        let coordinator = SwarmCoordinator::new();
        let context_manager = ContextManager::new(100); // 100 history entries
        let metrics = MetricsCollector::new(config.collect_metrics);
        
        let state = SwarmState {
            active_agents: 0,
            memory_utilization: 0.0,
            last_processing_time_ms: 0.0,
            total_processed: 0,
            cartan_violation: 0.0,
        };
        
        Ok(Self {
            config,
            router,
            attention,
            memory,
            scheduler,
            coordinator,
            context_manager,
            metrics,
            state,
        })
    }
    
    /// Register a micro-net with the swarm
    pub fn register_agent(&mut self, agent: Box<dyn MicroNet>) -> Result<()> {
        if self.state.active_agents >= self.config.max_agents {
            return Err(Error::InvalidConfiguration(
                "Maximum number of agents reached".into()
            ));
        }
        
        self.router.register_micro_net(agent);
        self.state.active_agents += 1;
        
        Ok(())
    }
    
    /// Process an input through the swarm
    pub fn process(&mut self, input: &RootVector) -> Result<RootVector> {
        let timer = Timer::start("swarm_process".to_string());
        
        // Get context for this session
        let context = self.context_manager.get_or_create(&self.config.session_id);
        
        // Route the input to appropriate micro-nets
        let execution_plan = self.router.route(input, Some(context))?;
        
        // Execute the plan
        let intermediate_results = self.scheduler.execute_plan(&execution_plan, input, context)?;
        
        // Apply Cartan attention to combine results
        let attended_results = self.attention.apply_attention(&intermediate_results)?;
        
        // Get final result (take first attended result for now)
        let final_result = attended_results.into_iter().next()
            .unwrap_or_else(|| RootVector::zeros());
        
        // Update state and metrics
        let timing = timer.stop();
        self.state.last_processing_time_ms = timing.elapsed_ms();
        self.state.total_processed += 1;
        self.state.memory_utilization = self.memory.utilization();
        self.state.cartan_violation = self.attention.compute_cartan_violation(&[final_result]);
        
        // Record in context (assume success for now)
        context.record_activation(
            "swarm".to_string(), 
            1.0, 
            self.state.total_processed
        );
        context.update_context_vector(final_result);
        
        if self.config.collect_metrics {
            self.metrics.record_timing(timing);
            self.metrics.record_processing(&final_result);
        }
        
        Ok(final_result)
    }
    
    /// Process a batch of inputs
    pub fn process_batch(&mut self, inputs: &[RootVector]) -> Result<Vec<RootVector>> {
        let mut results = Vec::with_capacity(inputs.len());
        
        for input in inputs {
            let result = self.process(input)?;
            results.push(result);
        }
        
        Ok(results)
    }
    
    /// Get current swarm state
    pub fn state(&self) -> &SwarmState {
        &self.state
    }
    
    /// Get metrics report as JSON
    pub fn get_metrics_json(&self) -> Result<String> {
        if !self.config.collect_metrics {
            return Ok("{}".to_string());
        }
        
        let report = self.metrics.generate_report();
        serde_json::to_string(&report)
            .map_err(|_| Error::InvalidConfiguration("Failed to serialize metrics".into()))
    }
    
    /// Reset the swarm state
    pub fn reset(&mut self) {
        self.memory.reset();
        self.context_manager.clear_all();
        self.metrics.reset();
        self.state = SwarmState {
            active_agents: self.state.active_agents, // Keep agent count
            memory_utilization: 0.0,
            last_processing_time_ms: 0.0,
            total_processed: 0,
            cartan_violation: 0.0,
        };
    }
    
    /// List all registered agents
    pub fn list_agents(&self) -> Vec<(String, String, bool)> {
        self.router.list_nets()
    }
    
    /// Get detailed system status
    pub fn get_status(&self) -> Result<String> {
        let status = serde_json::json!({
            "state": self.state,
            "agents": self.list_agents(),
            "memory_utilization": self.memory.utilization(),
            "context_sessions": self.context_manager.num_contexts(),
        });
        
        serde_json::to_string_pretty(&status)
            .map_err(|_| Error::InvalidConfiguration("Failed to serialize status".into()))
    }
    
    /// Update swarm configuration
    pub fn update_config(&mut self, config: SwarmConfig) -> Result<()> {
        // Validate new configuration
        if config.max_agents < self.state.active_agents {
            return Err(Error::InvalidConfiguration(
                "Cannot reduce max_agents below current active count".into()
            ));
        }
        
        self.config = config;
        Ok(())
    }
}

/// Builder for creating swarm orchestrators
pub struct SwarmBuilder {
    config: SwarmConfig,
}

impl SwarmBuilder {
    /// Create a new swarm builder with default configuration
    pub fn new() -> Self {
        Self {
            config: SwarmConfig::default(),
        }
    }
    
    /// Set maximum number of agents
    pub fn max_agents(mut self, max_agents: usize) -> Self {
        self.config.max_agents = max_agents;
        self
    }
    
    /// Set memory pool size
    pub fn memory_pool_size(mut self, size: usize) -> Self {
        self.config.memory_pool_size = size;
        self
    }
    
    /// Enable or disable parallel execution
    pub fn parallel(mut self, enable: bool) -> Self {
        self.config.enable_parallel = enable;
        self
    }
    
    /// Enable or disable metrics collection
    pub fn metrics(mut self, enable: bool) -> Self {
        self.config.collect_metrics = enable;
        self
    }
    
    /// Set session ID
    pub fn session_id(mut self, id: String) -> Self {
        self.config.session_id = id;
        self
    }
    
    /// Build the orchestrator
    pub fn build(self) -> Result<SwarmOrchestrator> {
        SwarmOrchestrator::new(self.config)
    }
}

impl Default for SwarmBuilder {
    fn default() -> Self {
        Self::new()
    }
}