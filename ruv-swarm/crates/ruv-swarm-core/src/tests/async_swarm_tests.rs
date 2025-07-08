//! Simple async swarm tests that match the actual async API

#[cfg(feature = "std")]
use crate::async_swarm::*;
#[cfg(feature = "std")]
use crate::agent::*;
#[cfg(feature = "std")]
use crate::task::*;
#[cfg(feature = "std")]
use crate::topology::*;

#[cfg(feature = "std")]
#[tokio::test]
async fn test_async_swarm_config_creation() {
    let mut config = AsyncSwarmConfig::default();
    config.max_agents = 10;
    config.topology_type = TopologyType::Mesh;
    config.max_concurrent_tasks_per_agent = 5;
    
    assert_eq!(config.max_agents, 10);
    assert_eq!(config.topology_type, TopologyType::Mesh);
    assert_eq!(config.max_concurrent_tasks_per_agent, 5);
}

#[cfg(feature = "std")]
#[tokio::test]
async fn test_async_swarm_creation() {
    let mut config = AsyncSwarmConfig::default();
    config.max_agents = 5;
    config.topology_type = TopologyType::Star;
    
    let swarm = AsyncSwarm::new(config);
    
    // Verify swarm was created successfully
    let metrics = swarm.metrics().await;
    assert_eq!(metrics.total_agents, 0);
    assert_eq!(metrics.queued_tasks, 0);
    assert_eq!(metrics.assigned_tasks, 0);
}

#[cfg(feature = "std")]
#[tokio::test]
async fn test_async_swarm_metrics() {
    let swarm = AsyncSwarm::new(AsyncSwarmConfig::default());
    let metrics = swarm.metrics().await;
    
    // Basic metrics validation
    assert_eq!(metrics.total_agents, 0);
    assert_eq!(metrics.queued_tasks, 0);
    assert_eq!(metrics.assigned_tasks, 0);
    assert_eq!(metrics.avg_agent_load, 0.0);
}

#[cfg(feature = "std")]
#[tokio::test]
async fn test_async_swarm_config_defaults() {
    let config = AsyncSwarmConfig::default();
    assert_eq!(config.topology_type, TopologyType::default());
    assert!(!config.enable_auto_scaling);
    assert_eq!(config.max_concurrent_tasks_per_agent, 10);
    assert_eq!(config.task_timeout_ms, 30000);
}

#[cfg(feature = "std")]
#[tokio::test]
async fn test_async_swarm_task_submission() {
    let swarm = AsyncSwarm::new(AsyncSwarmConfig::default());
    
    // Create a simple task
    let task = Task::new("test-task-1", "compute")
        .with_payload(TaskPayload::Binary(vec![1, 2, 3, 4]));
    
    // Submit task to swarm
    let result = swarm.submit_task(task).await;
    assert!(result.is_ok());
    
    // Verify task was queued
    let queue_size = swarm.task_queue_size().await;
    assert_eq!(queue_size, 1);
}

#[cfg(feature = "std")]
#[tokio::test]
async fn test_async_swarm_agent_registration() {
    let swarm = AsyncSwarm::new(AsyncSwarmConfig::default());
    
    // Create a dynamic agent
    let agent = DynamicAgent::new("test-agent".to_string(), vec!["compute".to_string()]);
    let agent_id = agent.id().to_string();
    
    // Register the agent
    let result = swarm.register_agent(agent).await;
    assert!(result.is_ok());
    
    // Verify agent was registered
    let metrics = swarm.metrics().await;
    assert_eq!(metrics.total_agents, 1);
    
    // Verify agent exists
    let has_agent = swarm.has_agent(&agent_id).await;
    assert!(has_agent);
}

#[cfg(feature = "std")]
#[tokio::test]
async fn test_async_swarm_agent_unregistration() {
    let swarm = AsyncSwarm::new(AsyncSwarmConfig::default());
    
    // Create and register a dynamic agent
    let agent = DynamicAgent::new("test-agent".to_string(), vec!["compute".to_string()]);
    let agent_id = agent.id().to_string();
    
    swarm.register_agent(agent).await.unwrap();
    
    // Verify agent is registered
    let metrics = swarm.metrics().await;
    assert_eq!(metrics.total_agents, 1);
    
    // Unregister the agent
    let result = swarm.unregister_agent(&agent_id).await;
    assert!(result.is_ok());
    
    // Verify agent was removed
    let metrics = swarm.metrics().await;
    assert_eq!(metrics.total_agents, 0);
    
    // Verify agent no longer exists
    let has_agent = swarm.has_agent(&agent_id).await;
    assert!(!has_agent);
}

#[cfg(feature = "std")]
#[tokio::test]
async fn test_async_swarm_task_distribution() {
    let swarm = AsyncSwarm::new(AsyncSwarmConfig::default());
    
    // Create and register a dynamic agent
    let agent = DynamicAgent::new("test-agent".to_string(), vec!["compute".to_string()]);
    swarm.register_agent(agent).await.unwrap();
    
    // Create and submit a task
    let task = Task::new("test-task-1", "compute")
        .with_payload(TaskPayload::Binary(vec![1, 2, 3, 4]));
    
    swarm.submit_task(task).await.unwrap();
    
    // Distribute tasks
    let assignments = swarm.distribute_tasks().await.unwrap();
    
    // Verify task was assigned
    assert_eq!(assignments.len(), 1);
    assert_eq!(assignments[0].0.to_string(), "test-task-1");
    assert_eq!(assignments[0].1, "test-agent");
}

#[cfg(feature = "std")]
#[tokio::test]
async fn test_async_swarm_agent_status() {
    let swarm = AsyncSwarm::new(AsyncSwarmConfig::default());
    
    // Create and register a dynamic agent
    let agent = DynamicAgent::new("test-agent".to_string(), vec!["compute".to_string()]);
    swarm.register_agent(agent).await.unwrap();
    
    // Get agent statuses
    let statuses = swarm.agent_statuses().await;
    
    // Verify agent status
    assert_eq!(statuses.len(), 1);
    assert_eq!(statuses.get("test-agent"), Some(&AgentStatus::Running));
}