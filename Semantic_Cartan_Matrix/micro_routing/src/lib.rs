//! Micro Routing - Dynamic routing for micro-neural networks
//! 
//! This crate provides dynamic routing and context management for
//! micro-neural network systems in the Semantic Cartan Matrix.

#![no_std]
#![forbid(unsafe_code)]
#![warn(missing_docs)]

extern crate alloc;
use alloc::vec::Vec;
use alloc::string::String;

pub mod router;
pub mod context; 
pub mod gating;

// Main types are defined inline for simplicity

/// Dynamic router for micro-networks
pub struct DynamicRouter;
/// Router configuration
pub struct RouterConfig;
/// Routing decision result
pub struct RoutingDecision;
/// Context vector for routing
pub struct ContextVector;
/// Context manager
pub struct ContextManager;
/// Neural gate for filtering
pub struct NeuralGate;
/// Gating function implementation
pub struct GatingFunction;

// Default implementations
impl Default for DynamicRouter { fn default() -> Self { Self } }
impl Default for RouterConfig { fn default() -> Self { Self } }
impl Default for ContextVector { fn default() -> Self { Self } }
impl Default for ContextManager { fn default() -> Self { Self } }
impl Default for NeuralGate { fn default() -> Self { Self } }
impl Default for GatingFunction { fn default() -> Self { Self } }

/// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// 32-dimensional root space constant
pub const ROOT_DIM: usize = 32;

/// Simple 32D vector type for routing
#[derive(Clone, Copy, Debug)]
pub struct RootVector {
    /// Vector data
    pub data: [f32; 32],
}

impl RootVector {
    /// Create zero vector
    pub fn zero() -> Self {
        Self { data: [0.0; 32] }
    }
    
    /// Create vector from array
    pub fn from_array(data: [f32; 32]) -> Self {
        Self { data }
    }
    
    /// Get slice view
    pub fn as_slice(&self) -> &[f32] {
        &self.data[..]  
    }
    
    /// Dot product
    pub fn dot(&self, other: &Self) -> f32 {
        self.data.iter().zip(other.data.iter()).map(|(a, b)| a * b).sum()
    }
    
    /// Magnitude (returns dot product for simplicity)
    pub fn magnitude(&self) -> f32 {
        // Simplified for no_std compatibility
        self.dot(self)
    }
}

impl Default for RootVector {
    fn default() -> Self {
        Self::zero()
    }
}

/// Result type
pub type Result<T> = core::result::Result<T, &'static str>;

/// Error type  
#[derive(Debug)]
pub enum Error {
    /// Invalid input
    InvalidInput,
    /// Computation failed
    ComputationError,
}

/// Basic trait for micro networks
pub trait MicroNet {
    /// Get agent ID
    fn id(&self) -> u32;
    /// Get agent type string  
    fn agent_type(&self) -> String;
    /// Alias for agent_type
    fn net_type(&self) -> String { self.agent_type() }
}