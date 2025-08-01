//! Integration layer for rUv-FANN

use crate::types::{RootVector, RootSpace, CartanMatrix};
use crate::micronet::{MicroNet, AgentType, BasicAgent};
use crate::projection::{project_to_root, embed_from_root};
use alloc::vec::Vec;
use alloc::boxed::Box;

/// Bridge between rUv-FANN and the Semantic Cartan Matrix
pub struct RuvFannBridge {
    /// The root space for projections
    pub root_space: RootSpace,
    /// Cartan regularization strength
    pub lambda: f32,
    /// Target Cartan matrix for regularization
    pub target_cartan: CartanMatrix,
    /// Whether to enable rank-1 routing optimization
    pub enable_routing: bool,
    /// Cosine annealing schedule for lambda
    annealing: AnnealingSchedule,
}

impl RuvFannBridge {
    /// Create a new bridge with default settings
    pub fn new() -> Self {
        Self {
            root_space: RootSpace::new(),
            lambda: 0.0, // Start with no regularization
            target_cartan: CartanMatrix::default(),
            enable_routing: true,
            annealing: AnnealingSchedule::new(0.0, 0.01, 1000),
        }
    }
    
    /// Create bridge with custom root space
    pub fn with_root_space(root_space: RootSpace) -> Self {
        Self {
            root_space,
            lambda: 0.0,
            target_cartan: CartanMatrix::default(),
            enable_routing: true,
            annealing: AnnealingSchedule::new(0.0, 0.01, 1000),
        }
    }
    
    /// Process tokens through the Cartan-regularized attention
    pub fn forward(&mut self, tokens: &[Vec<f32>]) -> Vec<RootVector> {
        let mut outputs = Vec::with_capacity(tokens.len());
        
        for token in tokens {
            let root_vec = project_to_root(token, &self.root_space);
            outputs.push(root_vec);
        }
        
        outputs
    }
    
    /// Compute Cartan regularization loss
    pub fn cartan_loss(&self) -> f32 {
        let current_cartan = CartanMatrix::from_basis(&self.root_space.basis);
        current_cartan.frobenius_distance(&self.target_cartan)
    }
    
    /// Update regularization strength with annealing
    pub fn step_regularization(&mut self, epoch: u32) {
        self.lambda = self.annealing.get_value(epoch);
    }
    
    /// Create a routing agent (rank-1 head)
    pub fn create_routing_agent(&self, id: u32) -> Box<dyn MicroNet> {
        Box::new(BasicAgent::new_routing(id))
    }
    
    /// Create a reasoning agent (full-rank head)
    pub fn create_reasoning_agent(&self, id: u32) -> Box<dyn MicroNet> {
        Box::new(BasicAgent::new_reasoning(id))
    }
    
    /// Check if an agent should be converted to routing type
    pub fn should_convert_to_routing(&self, attention_weights: &[f32]) -> bool {
        if !self.enable_routing {
            return false;
        }
        
        // Use spectral analysis to detect rank-1 behavior
        let rank = crate::projection::compute_attention_rank(attention_weights, 32);
        rank == 1
    }
    
    /// Export current state for dashboard visualization
    pub fn export_metrics(&self) -> MetricsExport {
        MetricsExport {
            cartan_loss: self.cartan_loss(),
            lambda: self.lambda,
            basis_orthogonality: self.compute_orthogonality(),
            root_magnitudes: self.compute_root_magnitudes(),
        }
    }
    
    /// Compute orthogonality metric for monitoring
    fn compute_orthogonality(&self) -> f32 {
        let mut sum = 0.0f32;
        let mut count = 0;
        
        for i in 0..32 {
            for j in (i+1)..32 {
                let dot = self.root_space.basis[i].dot(&self.root_space.basis[j]);
                sum += libm::fabsf(dot);
                count += 1;
            }
        }
        
        if count > 0 {
            sum / count as f32
        } else {
            0.0
        }
    }
    
    /// Compute root vector magnitudes for monitoring
    fn compute_root_magnitudes(&self) -> Vec<f32> {
        self.root_space.basis.iter()
            .map(|v| v.magnitude())
            .collect()
    }
}

impl Default for RuvFannBridge {
    fn default() -> Self {
        Self::new()
    }
}

/// Cosine annealing schedule for regularization strength
struct AnnealingSchedule {
    min_value: f32,
    max_value: f32,
    period: u32,
}

impl AnnealingSchedule {
    fn new(min_value: f32, max_value: f32, period: u32) -> Self {
        Self { min_value, max_value, period }
    }
    
    fn get_value(&self, epoch: u32) -> f32 {
        if epoch < 3 {
            // Warm start: no regularization for first 3 epochs
            self.min_value
        } else {
            // Cosine annealing
            let t = (epoch - 3) % self.period;
            let ratio = t as f32 / self.period as f32;
            let cos_val = libm::cosf(core::f32::consts::PI * ratio);
            
            self.min_value + (self.max_value - self.min_value) * (1.0 + cos_val) / 2.0
        }
    }
}

/// Metrics export for dashboard visualization
#[derive(Debug, Clone)]
pub struct MetricsExport {
    /// Current Cartan regularization loss
    pub cartan_loss: f32,
    /// Current lambda value
    pub lambda: f32,
    /// Average off-diagonal orthogonality
    pub basis_orthogonality: f32,
    /// Magnitudes of all root vectors
    pub root_magnitudes: Vec<f32>,
}

/// WASM-specific bindings
#[cfg(feature = "wasm")]
pub mod wasm {
    use super::*;
    use wasm_bindgen::prelude::*;
    
    #[wasm_bindgen]
    pub struct WasmBridge {
        inner: RuvFannBridge,
    }
    
    #[wasm_bindgen]
    impl WasmBridge {
        #[wasm_bindgen(constructor)]
        pub fn new() -> Self {
            Self {
                inner: RuvFannBridge::new(),
            }
        }
        
        /// Project a flat array to root space
        #[wasm_bindgen]
        pub fn project(&self, input: &[f32]) -> Vec<f32> {
            let root_vec = project_to_root(input, &self.inner.root_space);
            root_vec.data.to_vec()
        }
        
        /// Get current Cartan loss
        #[wasm_bindgen]
        pub fn get_cartan_loss(&self) -> f32 {
            self.inner.cartan_loss()
        }
        
        /// Step regularization schedule
        #[wasm_bindgen]
        pub fn step_regularization(&mut self, epoch: u32) {
            self.inner.step_regularization(epoch);
        }
        
        /// Export metrics as JSON string
        #[wasm_bindgen]
        pub fn export_metrics_json(&self) -> alloc::string::String {
            let metrics = self.inner.export_metrics();
            
            // Simplified JSON for no_std compatibility
            // Return just the cartan loss for now
            alloc::format!("{{\"cartan_loss\":{:.4}}}", metrics.cartan_loss)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bridge_creation() {
        let bridge = RuvFannBridge::new();
        assert_eq!(bridge.lambda, 0.0);
        assert!(bridge.enable_routing);
    }

    #[test]
    fn test_annealing_schedule() {
        let schedule = AnnealingSchedule::new(0.0, 1.0, 100);
        
        // Warm start
        assert_eq!(schedule.get_value(0), 0.0);
        assert_eq!(schedule.get_value(2), 0.0);
        
        // After warm start
        assert!(schedule.get_value(3) > 0.0);
        assert!(schedule.get_value(53) > 0.0); // Mid-cycle
    }

    #[test]
    fn test_metrics_export() {
        let bridge = RuvFannBridge::new();
        let metrics = bridge.export_metrics();
        
        assert_eq!(metrics.root_magnitudes.len(), 32);
        assert!(metrics.basis_orthogonality >= 0.0);
    }
}