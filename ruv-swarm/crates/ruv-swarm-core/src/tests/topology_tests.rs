//! Simple topology tests that match the actual API

use crate::topology::*;

#[test]
fn test_topology_creation() {
    let topology = Topology::new(TopologyType::Mesh);
    assert_eq!(topology.topology_type, TopologyType::Mesh);
    assert!(topology.connections.is_empty());
    assert!(topology.groups.is_empty());
    assert!(topology.hierarchy.is_none());
}

#[test]
fn test_mesh_topology() {
    let agents: Vec<String> = vec!["agent-1".to_string(), "agent-2".to_string(), "agent-3".to_string()];
    let topology = Topology::mesh(&agents);
    
    // Each agent should be connected to all others in mesh topology
    // Total connections: 3 agents * 2 connections each / 2 (undirected) = 3
    assert_eq!(topology.connection_count(), 3);
    assert!(topology.is_fully_connected());
}

#[test]
fn test_star_topology() {
    let center = "center".to_string();
    let agents: Vec<String> = vec!["agent-1".to_string(), "agent-2".to_string(), "agent-3".to_string(), "agent-4".to_string()];
    
    let topology = Topology::star(&center, &agents);
    
    // Center should be connected to all agents
    let center_neighbors = topology.get_neighbors(&center).unwrap();
    assert_eq!(center_neighbors.len(), 4);
    
    // Each agent should only be connected to center
    for agent in &agents {
        let neighbors = topology.get_neighbors(agent).unwrap();
        assert_eq!(neighbors.len(), 1);
        assert!(neighbors.contains(&center));
    }
}

#[test]
fn test_pipeline_topology() {
    let agents: Vec<String> = vec!["stage-1".to_string(), "stage-2".to_string(), "stage-3".to_string(), "stage-4".to_string()];
    let topology = Topology::pipeline(&agents);
    
    // First agent connects only forward
    let first_neighbors = topology.get_neighbors(&agents[0]).unwrap();
    assert_eq!(first_neighbors.len(), 1);
    assert!(first_neighbors.contains(&agents[1]));
    
    // Last agent connects only backward
    let last_neighbors = topology.get_neighbors(&agents[3]).unwrap();
    assert_eq!(last_neighbors.len(), 1);
    assert!(last_neighbors.contains(&agents[2]));
}

#[test]
fn test_add_connection() {
    let mut topology = Topology::new(TopologyType::Custom);
    
    let agent1 = "agent-1".to_string();
    let agent2 = "agent-2".to_string();
    
    topology.add_connection(agent1.clone(), agent2.clone());
    
    assert!(topology.are_connected(&agent1, &agent2));
    assert!(topology.are_connected(&agent2, &agent1)); // Undirected
}

#[test]
fn test_remove_connection() {
    let mut topology = Topology::new(TopologyType::Custom);
    
    let agent1 = "agent-1".to_string();
    let agent2 = "agent-2".to_string();
    
    topology.add_connection(agent1.clone(), agent2.clone());
    assert!(topology.are_connected(&agent1, &agent2));
    
    topology.remove_connection(&agent1, &agent2);
    assert!(!topology.are_connected(&agent1, &agent2));
    assert!(!topology.are_connected(&agent2, &agent1));
}

#[test]
fn test_topology_type_default() {
    assert_eq!(TopologyType::default(), TopologyType::Mesh);
}

#[test] 
fn test_connectivity_check() {
    // Empty topology is considered fully connected
    let empty = Topology::new(TopologyType::Custom);
    assert!(empty.is_fully_connected());
}