//! Execution planning for micro-net orchestration

use alloc::{vec::Vec, string::String};

#[cfg(feature = "serde")]
use serde::{Serialize, Deserialize};

/// Execution mode for a plan
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum ExecutionMode {
    /// Execute micro-nets sequentially
    Sequential,
    /// Execute micro-nets in parallel
    Parallel,
    /// Pipeline execution (for streaming)
    Pipeline,
}

/// A single step in an execution plan
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ExecutionStep {
    /// Index of the micro-net in the router's registry
    pub net_index: usize,
    
    /// ID of the micro-net
    pub net_id: String,
    
    /// Type of the micro-net
    pub net_type: String,
    
    /// Whether this is a routing head
    pub is_routing: bool,
}

/// An execution plan describing how to process an input
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ExecutionPlan {
    /// Steps to execute
    pub steps: Vec<ExecutionStep>,
    
    /// Execution mode
    pub mode: ExecutionMode,
}

impl ExecutionPlan {
    /// Create a new execution plan
    pub fn new(steps: Vec<ExecutionStep>, mode: ExecutionMode) -> Self {
        Self { steps, mode }
    }
    
    /// Check if the plan is empty
    pub fn is_empty(&self) -> bool {
        self.steps.is_empty()
    }
    
    /// Get the number of steps
    pub fn len(&self) -> usize {
        self.steps.len()
    }
    
    /// Check if the plan can be executed in parallel
    pub fn is_parallel(&self) -> bool {
        matches!(self.mode, ExecutionMode::Parallel)
    }
    
    /// Check if the plan is sequential
    pub fn is_sequential(&self) -> bool {
        matches!(self.mode, ExecutionMode::Sequential)
    }
    
    /// Get routing steps only
    pub fn routing_steps(&self) -> Vec<&ExecutionStep> {
        self.steps.iter()
            .filter(|step| step.is_routing)
            .collect()
    }
    
    /// Get non-routing (processing) steps only
    pub fn processing_steps(&self) -> Vec<&ExecutionStep> {
        self.steps.iter()
            .filter(|step| !step.is_routing)
            .collect()
    }
    
    /// Estimate total computational cost
    /// (Placeholder - would use actual FLOP estimates from micro-nets)
    pub fn estimated_cost(&self) -> u64 {
        self.steps.iter()
            .map(|step| if step.is_routing { 1000 } else { 10000 })
            .sum()
    }
}