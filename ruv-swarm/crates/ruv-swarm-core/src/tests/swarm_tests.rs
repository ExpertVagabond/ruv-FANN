//! Simple swarm tests that match the actual API

use crate::swarm::*;
use crate::topology::*;

#[test]
fn test_swarm_config_creation() {
    let mut config = SwarmConfig::default();
    config.max_agents = 10;
    config.topology_type = TopologyType::Mesh;
    
    assert_eq!(config.max_agents, 10);
    assert_eq!(config.topology_type, TopologyType::Mesh);
}

#[test]
fn test_swarm_creation() {
    let mut config = SwarmConfig::default();
    config.max_agents = 5;
    config.topology_type = TopologyType::Star;
    
    let swarm = Swarm::new(config);
    
    // Verify swarm was created successfully
    assert_eq!(swarm.metrics().total_agents, 0);
}

#[test]
fn test_swarm_metrics() {
    let swarm = Swarm::new(SwarmConfig::default());
    let metrics = swarm.metrics();
    
    // Basic metrics validation
    assert_eq!(metrics.total_agents, 0);
    assert_eq!(metrics.queued_tasks, 0);
    assert_eq!(metrics.assigned_tasks, 0);
}

#[test]
fn test_swarm_config_defaults() {
    let config = SwarmConfig::default();
    assert_eq!(config.topology_type, TopologyType::default());
    assert!(!config.enable_auto_scaling);
}