//! MicroNet trait and agent implementation for neural network components

use crate::types::{RootVector, RootSpace};
use alloc::vec::Vec;
use alloc::boxed::Box;
use core::fmt;

/// Trait for micro neural network agents
/// 
/// This trait defines the interface for neural network components that
/// operate within the Semantic Cartan Matrix framework.
pub trait MicroNet {
    /// Get the agent's unique identifier
    fn id(&self) -> u32;
    
    /// Get the agent's current state in root space
    fn state(&self) -> &AgentState;
    
    /// Update the agent's state
    fn update_state(&mut self, new_state: RootVector);
    
    /// Process input through the micro network
    /// 
    /// Takes a root-space vector and returns a processed output
    fn forward(&mut self, input: &RootVector) -> RootVector;
    
    /// Check if this agent is a rank-1 routing head
    fn is_routing_head(&self) -> bool;
    
    /// Get the agent's specialization type
    fn agent_type(&self) -> AgentType;
    
    /// Get the agent's network type (alias for agent_type)
    fn net_type(&self) -> AgentType {
        self.agent_type()
    }
    
    /// Compute compatibility score with another agent
    /// 
    /// Returns a value between 0.0 (incompatible) and 1.0 (fully compatible)
    fn compatibility(&self, other: &dyn MicroNet) -> f32 {
        // Default implementation using inner product in root space
        let self_state = self.state().root_vector;
        let other_state = other.state().root_vector;
        
        let dot = self_state.dot(&other_state);
        let self_mag = self_state.magnitude();
        let other_mag = other_state.magnitude();
        
        if self_mag > 0.0 && other_mag > 0.0 {
            // Cosine similarity, normalized to [0, 1]
            (dot / (self_mag * other_mag) + 1.0) / 2.0
        } else {
            0.5 // Neutral compatibility if either has zero state
        }
    }
}

/// Agent state in the Semantic Cartan Matrix system
#[derive(Clone, Debug)]
pub struct AgentState {
    /// The agent's position in 32-dimensional root space
    pub root_vector: RootVector,
    /// Activation level (0.0 to 1.0)
    pub activation: f32,
    /// Confidence in current state (0.0 to 1.0)
    pub confidence: f32,
    /// Number of updates performed
    pub update_count: u32,
}

impl AgentState {
    /// Create a new agent state
    pub fn new() -> Self {
        Self {
            root_vector: RootVector::zero(),
            activation: 0.0,
            confidence: 0.0,
            update_count: 0,
        }
    }
    
    /// Create state with initial root vector
    pub fn with_vector(root_vector: RootVector) -> Self {
        Self {
            root_vector,
            activation: 1.0,
            confidence: 1.0,
            update_count: 0,
        }
    }
    
    /// Update the state with decay
    pub fn update(&mut self, new_vector: RootVector, learning_rate: f32) {
        // Exponential moving average update
        for i in 0..32 {
            self.root_vector.data[i] = 
                (1.0 - learning_rate) * self.root_vector.data[i] + 
                learning_rate * new_vector.data[i];
        }
        
        self.update_count += 1;
        
        // Update confidence based on consistency
        let similarity = self.root_vector.dot(&new_vector) / 
            (self.root_vector.magnitude() * new_vector.magnitude());
        self.confidence = 0.9 * self.confidence + 0.1 * similarity.abs();
    }
}

impl Default for AgentState {
    fn default() -> Self {
        Self::new()
    }
}

/// Types of agents in the system
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AgentType {
    /// Full-rank attention head for complex reasoning
    Reasoning,
    /// Rank-1 routing head for efficient gating
    Routing,
    /// Feature extraction agent
    Feature,
    /// Embedding projection agent
    Embedding,
    /// Specialized domain expert
    Expert,
}

impl fmt::Display for AgentType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AgentType::Reasoning => write!(f, "Reasoning"),
            AgentType::Routing => write!(f, "Routing"),
            AgentType::Feature => write!(f, "Feature"),
            AgentType::Embedding => write!(f, "Embedding"),
            AgentType::Expert => write!(f, "Expert"),
        }
    }
}

/// A basic implementation of a MicroNet agent
pub struct BasicAgent {
    id: u32,
    state: AgentState,
    agent_type: AgentType,
    weights: RootVector,
    bias: f32,
    is_rank_one: bool,
}

impl BasicAgent {
    /// Create a new basic agent
    pub fn new(id: u32, agent_type: AgentType) -> Self {
        Self {
            id,
            state: AgentState::new(),
            agent_type,
            weights: RootVector::zero(),
            bias: 0.0,
            is_rank_one: agent_type == AgentType::Routing,
        }
    }
    
    /// Create a routing agent (rank-1)
    pub fn new_routing(id: u32) -> Self {
        Self::new(id, AgentType::Routing)
    }
    
    /// Create a reasoning agent (full-rank)
    pub fn new_reasoning(id: u32) -> Self {
        Self::new(id, AgentType::Reasoning)
    }
    
    /// Initialize weights from a direction vector
    pub fn with_weights(mut self, weights: RootVector) -> Self {
        self.weights = weights;
        self.weights.normalize();
        self
    }
}

impl MicroNet for BasicAgent {
    fn id(&self) -> u32 {
        self.id
    }
    
    fn state(&self) -> &AgentState {
        &self.state
    }
    
    fn update_state(&mut self, new_state: RootVector) {
        self.state.update(new_state, 0.1); // Default learning rate
    }
    
    fn forward(&mut self, input: &RootVector) -> RootVector {
        let mut output = RootVector::zero();
        
        if self.is_rank_one {
            // Rank-1 operation: project onto weight direction
            let projection = input.dot(&self.weights);
            for i in 0..32 {
                output.data[i] = projection * self.weights.data[i] + self.bias;
            }
        } else {
            // Full operation: element-wise with weights
            for i in 0..32 {
                output.data[i] = input.data[i] * self.weights.data[i] + self.bias;
            }
        }
        
        // Update internal state
        self.state.root_vector = output;
        self.state.activation = output.magnitude().min(1.0);
        
        output
    }
    
    fn is_routing_head(&self) -> bool {
        self.is_rank_one
    }
    
    fn agent_type(&self) -> AgentType {
        self.agent_type
    }
}

/// Agent swarm coordinator
pub struct AgentSwarm {
    agents: Vec<Box<dyn MicroNet>>,
    root_space: RootSpace,
}

impl AgentSwarm {
    /// Create a new agent swarm
    pub fn new(root_space: RootSpace) -> Self {
        Self {
            agents: Vec::new(),
            root_space,
        }
    }
    
    /// Add an agent to the swarm
    pub fn add_agent(&mut self, agent: Box<dyn MicroNet>) {
        self.agents.push(agent);
    }
    
    /// Get agents by type
    pub fn agents_by_type(&self, agent_type: AgentType) -> Vec<&dyn MicroNet> {
        self.agents
            .iter()
            .filter(|a| a.agent_type() == agent_type)
            .map(|a| a.as_ref())
            .collect()
    }
    
    /// Find compatible agents for collaboration
    pub fn find_compatible(&self, agent: &dyn MicroNet, threshold: f32) -> Vec<&dyn MicroNet> {
        self.agents
            .iter()
            .filter(|a| {
                let compat = agent.compatibility(a.as_ref());
                compat >= threshold && a.id() != agent.id()
            })
            .map(|a| a.as_ref())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_creation() {
        let agent = BasicAgent::new(1, AgentType::Reasoning);
        assert_eq!(agent.id(), 1);
        assert_eq!(agent.agent_type(), AgentType::Reasoning);
        assert!(!agent.is_routing_head());
    }

    #[test]
    fn test_routing_agent() {
        let agent = BasicAgent::new_routing(2);
        assert_eq!(agent.agent_type(), AgentType::Routing);
        assert!(agent.is_routing_head());
    }

    #[test]
    fn test_agent_compatibility() {
        let mut agent1 = BasicAgent::new(1, AgentType::Reasoning);
        let mut agent2 = BasicAgent::new(2, AgentType::Reasoning);
        
        // Set similar states
        let state = RootVector::from_array([1.0; 32]);
        agent1.update_state(state);
        agent2.update_state(state);
        
        let compat = agent1.compatibility(&agent2);
        assert!(compat > 0.9); // Should be highly compatible
    }
}