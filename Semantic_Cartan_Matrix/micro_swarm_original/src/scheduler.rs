//! Parallel execution scheduler for micro-swarm orchestration

use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::{Duration, Instant};
use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use tokio::sync::{RwLock, Semaphore, mpsc};
use uuid::Uuid;
use tracing::{info, warn, debug, trace};

use crate::{Result, SwarmError, AgentId};

/// Task priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Priority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

/// Task definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: Uuid,
    pub name: String,
    pub priority: Priority,
    pub agent_affinity: Option<AgentId>,
    pub required_capabilities: Vec<String>,
    pub payload: serde_json::Value,
    pub dependencies: Vec<Uuid>,
    pub timeout: Duration,
    #[serde(skip, default = "Instant::now")]
    pub created_at: Instant,
}

/// Task execution status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskStatus {
    Pending,
    Scheduled,
    Running(AgentId),
    Completed(serde_json::Value),
    Failed(String),
    Cancelled,
    Timeout,
}

/// Scheduler configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulerConfig {
    pub max_concurrent_tasks: usize,
    pub task_queue_size: usize,
    pub agent_selection_strategy: SelectionStrategy,
    pub load_balancing_enabled: bool,
    pub preemption_enabled: bool,
    pub task_stealing_enabled: bool,
    pub scheduling_interval: Duration,
}

impl Default for SchedulerConfig {
    fn default() -> Self {
        Self {
            max_concurrent_tasks: 256,
            task_queue_size: 10000,
            agent_selection_strategy: SelectionStrategy::LoadBalanced,
            load_balancing_enabled: true,
            preemption_enabled: false,
            task_stealing_enabled: true,
            scheduling_interval: Duration::from_millis(10),
        }
    }
}

/// Agent selection strategies
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum SelectionStrategy {
    RoundRobin,
    LeastLoaded,
    LoadBalanced,
    CapabilityBased,
    AffinityBased,
}

/// Execution plan for a set of tasks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionPlan {
    pub id: Uuid,
    pub tasks: Vec<Task>,
    pub dependencies: HashMap<Uuid, Vec<Uuid>>,
    pub estimated_duration: Duration,
    pub parallelism_factor: f64,
}

/// Scheduler statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulerStats {
    pub total_tasks_scheduled: u64,
    pub total_tasks_completed: u64,
    pub total_tasks_failed: u64,
    pub average_queue_time: Duration,
    pub average_execution_time: Duration,
    pub current_queue_depth: usize,
    pub active_tasks: usize,
    pub agent_utilization: HashMap<AgentId, f64>,
}

/// Agent workload information
#[derive(Debug, Clone)]
struct AgentWorkload {
    agent_id: AgentId,
    capabilities: Vec<String>,
    current_tasks: usize,
    max_tasks: usize,
    total_completed: u64,
    average_completion_time: Duration,
}

/// Parallel task scheduler
pub struct ParallelScheduler {
    config: SchedulerConfig,
    task_queue: Arc<RwLock<VecDeque<Task>>>,
    running_tasks: Arc<RwLock<HashMap<Uuid, (AgentId, Instant)>>>,
    task_status: Arc<RwLock<HashMap<Uuid, TaskStatus>>>,
    agent_workloads: Arc<RwLock<HashMap<AgentId, AgentWorkload>>>,
    stats: Arc<RwLock<SchedulerStats>>,
    concurrency_limiter: Arc<Semaphore>,
    shutdown_tx: Option<mpsc::Sender<()>>,
}

impl ParallelScheduler {
    /// Create a new scheduler
    pub fn new(config: SchedulerConfig) -> Self {
        let concurrency_limiter = Arc::new(Semaphore::new(config.max_concurrent_tasks));
        
        Self {
            config,
            task_queue: Arc::new(RwLock::new(VecDeque::with_capacity(10000))),
            running_tasks: Arc::new(RwLock::new(HashMap::new())),
            task_status: Arc::new(RwLock::new(HashMap::new())),
            agent_workloads: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(SchedulerStats {
                total_tasks_scheduled: 0,
                total_tasks_completed: 0,
                total_tasks_failed: 0,
                average_queue_time: Duration::ZERO,
                average_execution_time: Duration::ZERO,
                current_queue_depth: 0,
                active_tasks: 0,
                agent_utilization: HashMap::new(),
            })),
            concurrency_limiter,
            shutdown_tx: None,
        }
    }
    
    /// Register an agent with the scheduler
    pub async fn register_agent(
        &self,
        agent_id: AgentId,
        capabilities: Vec<String>,
        max_tasks: usize,
    ) -> Result<()> {
        let mut workloads = self.agent_workloads.write().await;
        workloads.insert(agent_id, AgentWorkload {
            agent_id,
            capabilities,
            current_tasks: 0,
            max_tasks,
            total_completed: 0,
            average_completion_time: Duration::ZERO,
        });
        
        info!("Registered agent {:?} with scheduler", agent_id);
        Ok(())
    }
    
    /// Unregister an agent
    pub async fn unregister_agent(&self, agent_id: AgentId) -> Result<()> {
        let mut workloads = self.agent_workloads.write().await;
        workloads.remove(&agent_id);
        
        info!("Unregistered agent {:?} from scheduler", agent_id);
        Ok(())
    }
    
    /// Submit a task for execution
    pub async fn submit_task(&self, task: Task) -> Result<()> {
        // Check queue size
        let queue_len = self.task_queue.read().await.len();
        if queue_len >= self.config.task_queue_size {
            return Err(SwarmError::Scheduler("Task queue full".into()));
        }
        
        // Add to queue
        let mut queue = self.task_queue.write().await;
        let mut status = self.task_status.write().await;
        
        status.insert(task.id, TaskStatus::Pending);
        
        // Insert based on priority
        let insert_pos = queue.iter().position(|t| t.priority < task.priority)
            .unwrap_or(queue.len());
        queue.insert(insert_pos, task);
        
        // Update stats
        let mut stats = self.stats.write().await;
        stats.total_tasks_scheduled += 1;
        stats.current_queue_depth = queue.len();
        
        debug!("Task submitted to scheduler queue");
        Ok(())
    }
    
    /// Submit multiple tasks as a batch
    pub async fn submit_batch(&self, tasks: Vec<Task>) -> Result<()> {
        for task in tasks {
            self.submit_task(task).await?;
        }
        Ok(())
    }
    
    /// Create an execution plan for a set of tasks
    pub async fn create_execution_plan(&self, tasks: Vec<Task>) -> Result<ExecutionPlan> {
        // Build dependency graph
        let mut dependencies = HashMap::new();
        for task in &tasks {
            dependencies.insert(task.id, task.dependencies.clone());
        }
        
        // Calculate parallelism factor
        let max_parallel = self.calculate_max_parallelism(&tasks, &dependencies);
        let parallelism_factor = max_parallel as f64 / tasks.len() as f64;
        
        // Estimate duration
        let estimated_duration = self.estimate_plan_duration(&tasks, max_parallel).await;
        
        Ok(ExecutionPlan {
            id: Uuid::new_v4(),
            tasks,
            dependencies,
            estimated_duration,
            parallelism_factor,
        })
    }
    
    /// Start the scheduler
    pub async fn start(&mut self) -> Result<()> {
        info!("Starting parallel scheduler");
        
        let (shutdown_tx, mut shutdown_rx) = mpsc::channel(1);
        self.shutdown_tx = Some(shutdown_tx);
        
        // Spawn scheduling loop
        let queue = self.task_queue.clone();
        let running = self.running_tasks.clone();
        let status = self.task_status.clone();
        let workloads = self.agent_workloads.clone();
        let stats = self.stats.clone();
        let limiter = self.concurrency_limiter.clone();
        let config = self.config.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(config.scheduling_interval);
            
            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        // Schedule pending tasks
                        Self::schedule_tasks(
                            queue.clone(),
                            running.clone(),
                            status.clone(),
                            workloads.clone(),
                            stats.clone(),
                            limiter.clone(),
                            &config,
                        ).await;
                        
                        // Check for timeouts
                        Self::check_timeouts(
                            running.clone(),
                            status.clone(),
                            stats.clone(),
                        ).await;
                        
                        // Perform load balancing if enabled
                        if config.load_balancing_enabled {
                            Self::balance_load(
                                queue.clone(),
                                workloads.clone(),
                            ).await;
                        }
                    }
                    _ = shutdown_rx.recv() => {
                        info!("Scheduler shutting down");
                        break;
                    }
                }
            }
        });
        
        Ok(())
    }
    
    /// Stop the scheduler
    pub async fn stop(&mut self) -> Result<()> {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(()).await;
        }
        Ok(())
    }
    
    /// Get current scheduler statistics
    pub async fn get_stats(&self) -> SchedulerStats {
        self.stats.read().await.clone()
    }
    
    /// Get task status
    pub async fn get_task_status(&self, task_id: Uuid) -> Option<TaskStatus> {
        self.task_status.read().await.get(&task_id).cloned()
    }
    
    /// Internal: Schedule tasks from queue
    async fn schedule_tasks(
        queue: Arc<RwLock<VecDeque<Task>>>,
        running: Arc<RwLock<HashMap<Uuid, (AgentId, Instant)>>>,
        status: Arc<RwLock<HashMap<Uuid, TaskStatus>>>,
        workloads: Arc<RwLock<HashMap<AgentId, AgentWorkload>>>,
        stats: Arc<RwLock<SchedulerStats>>,
        limiter: Arc<Semaphore>,
        config: &SchedulerConfig,
    ) {
        let permit = match limiter.try_acquire() {
            Ok(permit) => permit,
            Err(_) => return, // At capacity
        };
        
        let mut queue = queue.write().await;
        let workloads = workloads.read().await;
        
        // Find next schedulable task
        let task_idx = queue.iter().position(|task| {
            // Check if dependencies are satisfied
            // In real implementation, would check task_status for dependencies
            true
        });
        
        if let Some(idx) = task_idx {
            let task = queue.remove(idx).unwrap();
            
            // Select agent based on strategy
            let agent_id = match config.agent_selection_strategy {
                SelectionStrategy::LeastLoaded => {
                    workloads.values()
                        .filter(|w| {
                            w.current_tasks < w.max_tasks &&
                            task.required_capabilities.iter()
                                .all(|cap| w.capabilities.contains(cap))
                        })
                        .min_by_key(|w| w.current_tasks)
                        .map(|w| w.agent_id)
                }
                _ => {
                    // Simple selection for other strategies
                    workloads.values()
                        .find(|w| {
                            w.current_tasks < w.max_tasks &&
                            task.required_capabilities.iter()
                                .all(|cap| w.capabilities.contains(cap))
                        })
                        .map(|w| w.agent_id)
                }
            };
            
            if let Some(agent_id) = agent_id {
                // Schedule task to agent
                let mut running = running.write().await;
                let mut status = status.write().await;
                
                running.insert(task.id, (agent_id, Instant::now()));
                status.insert(task.id, TaskStatus::Running(agent_id));
                
                trace!("Scheduled task {} to agent {:?}", task.id, agent_id);
                
                // Update stats
                let mut stats = stats.write().await;
                stats.active_tasks = running.len();
                stats.current_queue_depth = queue.len();
            } else {
                // No suitable agent, put task back
                queue.push_front(task);
                permit.forget(); // Don't consume permit
            }
        } else {
            permit.forget(); // Don't consume permit
        }
    }
    
    /// Internal: Check for task timeouts
    async fn check_timeouts(
        running: Arc<RwLock<HashMap<Uuid, (AgentId, Instant)>>>,
        status: Arc<RwLock<HashMap<Uuid, TaskStatus>>>,
        stats: Arc<RwLock<SchedulerStats>>,
    ) {
        let running = running.read().await;
        let mut status = status.write().await;
        let mut timeouts = Vec::new();
        
        for (task_id, (_, start_time)) in running.iter() {
            // In real implementation, would check against task timeout
            if start_time.elapsed() > Duration::from_secs(300) {
                timeouts.push(*task_id);
            }
        }
        
        for task_id in timeouts {
            status.insert(task_id, TaskStatus::Timeout);
            warn!("Task {} timed out", task_id);
            
            let mut stats = stats.write().await;
            stats.total_tasks_failed += 1;
        }
    }
    
    /// Internal: Balance load across agents
    async fn balance_load(
        queue: Arc<RwLock<VecDeque<Task>>>,
        workloads: Arc<RwLock<HashMap<AgentId, AgentWorkload>>>,
    ) {
        // Simple load balancing - in real implementation would be more sophisticated
        let workloads = workloads.read().await;
        
        // Calculate average load
        let total_load: usize = workloads.values().map(|w| w.current_tasks).sum();
        let avg_load = total_load as f64 / workloads.len() as f64;
        
        // Identify overloaded and underloaded agents
        let _overloaded: Vec<_> = workloads.values()
            .filter(|w| (w.current_tasks as f64) > avg_load * 1.5)
            .collect();
            
        let _underloaded: Vec<_> = workloads.values()
            .filter(|w| (w.current_tasks as f64) < avg_load * 0.5)
            .collect();
        
        // In real implementation, would redistribute tasks
    }
    
    /// Calculate maximum parallelism for a set of tasks
    fn calculate_max_parallelism(
        &self,
        tasks: &[Task],
        dependencies: &HashMap<Uuid, Vec<Uuid>>,
    ) -> usize {
        // Simple calculation - in real implementation would use graph analysis
        let max_depth = self.calculate_dependency_depth(tasks, dependencies);
        tasks.len() / max_depth.max(1)
    }
    
    /// Calculate dependency depth
    fn calculate_dependency_depth(
        &self,
        _tasks: &[Task],
        _dependencies: &HashMap<Uuid, Vec<Uuid>>,
    ) -> usize {
        // Simplified - return 3 for demo
        3
    }
    
    /// Estimate plan duration
    async fn estimate_plan_duration(
        &self,
        tasks: &[Task],
        max_parallel: usize,
    ) -> Duration {
        // Simple estimation
        let avg_task_duration = Duration::from_secs(1);
        let total_duration = avg_task_duration * tasks.len() as u32;
        total_duration / max_parallel as u32
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_scheduler_basic() {
        let mut scheduler = ParallelScheduler::new(SchedulerConfig::default());
        
        // Register agents
        scheduler.register_agent(
            AgentId::new(),
            vec!["compute".to_string()],
            4,
        ).await.unwrap();
        
        // Submit task
        let task = Task {
            id: Uuid::new_v4(),
            name: "test_task".to_string(),
            priority: Priority::Normal,
            agent_affinity: None,
            required_capabilities: vec!["compute".to_string()],
            payload: serde_json::json!({"test": "data"}),
            dependencies: vec![],
            timeout: Duration::from_secs(60),
            created_at: Instant::now(),
        };
        
        scheduler.submit_task(task.clone()).await.unwrap();
        
        // Check stats
        let stats = scheduler.get_stats().await;
        assert_eq!(stats.total_tasks_scheduled, 1);
        assert_eq!(stats.current_queue_depth, 1);
    }
}