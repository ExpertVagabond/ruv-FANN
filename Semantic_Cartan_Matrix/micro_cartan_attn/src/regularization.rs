//! Cartan regularization for training and fine-tuning

use alloc::vec::Vec;
use micro_core::{RootVector, Result, ROOT_DIM};
use crate::CartanMatrix;
use nalgebra::SMatrix;

#[cfg(feature = "serde")]
use serde::{Serialize, Deserialize};

/// Regularization loss components
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct RegularizationLoss {
    /// Total Cartan loss
    pub total_loss: f32,
    
    /// Orthogonality violation component
    pub orthogonality_loss: f32,
    
    /// Norm constraint violation
    pub norm_loss: f32,
    
    /// Custom constraint violations
    pub custom_loss: f32,
}

impl RegularizationLoss {
    /// Create a new regularization loss
    pub fn new() -> Self {
        Self {
            total_loss: 0.0,
            orthogonality_loss: 0.0,
            norm_loss: 0.0,
            custom_loss: 0.0,
        }
    }
    
    /// Compute total loss
    pub fn compute_total(&mut self) {
        self.total_loss = self.orthogonality_loss + self.norm_loss + self.custom_loss;
    }
}

/// Cartan regularizer for enforcing geometric constraints
#[derive(Debug, Clone)]
pub struct CartanRegularizer {
    /// Target Cartan matrix
    cartan_matrix: CartanMatrix,
    
    /// Regularization weights
    orthogonality_weight: f32,
    norm_weight: f32,
    custom_weight: f32,
    
    /// Target norm for vectors (sqrt(2) for Cartan normalization)
    target_norm: f32,
    
    /// Tolerance for constraint satisfaction
    tolerance: f32,
}

impl CartanRegularizer {
    /// Create a new Cartan regularizer
    pub fn new(cartan_matrix: CartanMatrix) -> Self {
        Self {
            cartan_matrix,
            orthogonality_weight: 1.0,
            norm_weight: 0.1,
            custom_weight: 0.0,
            target_norm: (2.0_f32).sqrt(), // Cartan normalization
            tolerance: 1e-6,
        }
    }
    
    /// Set regularization weights
    pub fn with_weights(mut self, 
                       orthogonality: f32, 
                       norm: f32, 
                       custom: f32) -> Self {
        self.orthogonality_weight = orthogonality;
        self.norm_weight = norm;
        self.custom_weight = custom;
        self
    }
    
    /// Set target norm for vectors
    pub fn with_target_norm(mut self, norm: f32) -> Self {
        self.target_norm = norm;
        self
    }
    
    /// Compute regularization loss for a set of vectors
    pub fn compute_loss(&self, vectors: &[RootVector]) -> RegularizationLoss {
        let mut loss = RegularizationLoss::new();
        
        // Orthogonality loss: ||C_actual - C_target||²
        loss.orthogonality_loss = self.compute_orthogonality_loss(vectors);
        
        // Norm constraint loss: Σ(||v_i|| - target_norm)²
        loss.norm_loss = self.compute_norm_loss(vectors);
        
        // Custom constraints (placeholder for future extensions)
        loss.custom_loss = 0.0;
        
        // Weight the losses
        loss.orthogonality_loss *= self.orthogonality_weight;
        loss.norm_loss *= self.norm_weight;
        loss.custom_loss *= self.custom_weight;
        
        loss.compute_total();
        loss
    }
    
    /// Compute orthogonality violation loss
    fn compute_orthogonality_loss(&self, vectors: &[RootVector]) -> f32 {
        self.cartan_matrix.compute_violation(vectors)
    }
    
    /// Compute norm constraint loss
    fn compute_norm_loss(&self, vectors: &[RootVector]) -> f32 {
        vectors.iter()
            .map(|v| {
                let norm_diff = v.norm() - self.target_norm;
                norm_diff * norm_diff
            })
            .sum()
    }
    
    /// Compute gradients for regularization (simplified)
    /// 
    /// This would be used during backpropagation to adjust vector values
    pub fn compute_gradients(&self, vectors: &[RootVector]) -> Result<Vec<RootVector>> {
        let mut gradients = vec![RootVector::zeros(); vectors.len()];
        
        // Gradient of orthogonality loss
        self.add_orthogonality_gradients(vectors, &mut gradients)?;
        
        // Gradient of norm loss
        self.add_norm_gradients(vectors, &mut gradients);
        
        Ok(gradients)
    }
    
    /// Add orthogonality loss gradients
    fn add_orthogonality_gradients(&self, 
                                  vectors: &[RootVector], 
                                  gradients: &mut [RootVector]) -> Result<()> {
        // For each vector pair, add gradient contribution
        for i in 0..vectors.len().min(ROOT_DIM) {
            for j in 0..vectors.len().min(ROOT_DIM) {
                if i == j {
                    continue;
                }
                
                let target_inner = self.cartan_matrix.target_inner_product(i, j);
                let actual_inner = vectors[i].dot(&vectors[j]);
                let error = actual_inner - target_inner;
                
                // Gradient: 2 * error * other_vector
                let grad_contribution = vectors[j].map(|x| 2.0 * error * x * self.orthogonality_weight);
                *gradients[i] += grad_contribution;
            }
        }
        
        Ok(())
    }
    
    /// Add norm loss gradients
    fn add_norm_gradients(&self, vectors: &[RootVector], gradients: &mut [RootVector]) {
        for (i, vector) in vectors.iter().enumerate() {
            let norm = vector.norm();
            if norm > self.tolerance {
                let norm_error = norm - self.target_norm;
                let grad_factor = 2.0 * norm_error * self.norm_weight / norm;
                
                let grad_contribution = vector.map(|x| grad_factor * x);
                *gradients[i] += grad_contribution;
            }
        }
    }
    
    /// Apply regularization step (adjust vectors to reduce constraint violations)
    pub fn regularization_step(&self, 
                              vectors: &mut [RootVector], 
                              step_size: f32) -> Result<RegularizationLoss> {
        // Compute gradients
        let gradients = self.compute_gradients(vectors)?;
        
        // Apply gradient step
        for (vector, gradient) in vectors.iter_mut().zip(gradients.iter()) {
            *vector = vector.map(|x| x) - gradient.map(|g| g * step_size);
        }
        
        // Return updated loss
        Ok(self.compute_loss(vectors))
    }
    
    /// Check if constraints are satisfied within tolerance
    pub fn constraints_satisfied(&self, vectors: &[RootVector]) -> bool {
        let loss = self.compute_loss(vectors);
        loss.total_loss < self.tolerance
    }
    
    /// Get the target Cartan matrix
    pub fn cartan_matrix(&self) -> &CartanMatrix {
        &self.cartan_matrix
    }
    
    /// Update regularization weights (for curriculum learning)
    pub fn update_weights(&mut self, 
                         orthogonality: f32, 
                         norm: f32, 
                         custom: f32) {
        self.orthogonality_weight = orthogonality;
        self.norm_weight = norm;
        self.custom_weight = custom;
    }
}

/// Training schedule for gradually introducing Cartan constraints
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct RegularizationSchedule {
    /// Current epoch
    current_epoch: usize,
    
    /// Warm-up epochs before regularization starts
    warmup_epochs: usize,
    
    /// Epochs over which to ramp up regularization
    rampup_epochs: usize,
    
    /// Maximum regularization weight
    max_weight: f32,
    
    /// Current weight
    current_weight: f32,
}

impl RegularizationSchedule {
    /// Create a new regularization schedule
    pub fn new(warmup_epochs: usize, rampup_epochs: usize, max_weight: f32) -> Self {
        Self {
            current_epoch: 0,
            warmup_epochs,
            rampup_epochs,
            max_weight,
            current_weight: 0.0,
        }
    }
    
    /// Advance to the next epoch and update weight
    pub fn step(&mut self) -> f32 {
        self.current_epoch += 1;
        
        if self.current_epoch <= self.warmup_epochs {
            // No regularization during warmup
            self.current_weight = 0.0;
        } else if self.current_epoch <= self.warmup_epochs + self.rampup_epochs {
            // Linear ramp-up
            let progress = (self.current_epoch - self.warmup_epochs) as f32 / self.rampup_epochs as f32;
            self.current_weight = self.max_weight * progress;
        } else {
            // Full regularization
            self.current_weight = self.max_weight;
        }
        
        self.current_weight
    }
    
    /// Get current regularization weight
    pub fn current_weight(&self) -> f32 {
        self.current_weight
    }
    
    /// Check if in warmup phase
    pub fn is_warmup(&self) -> bool {
        self.current_epoch <= self.warmup_epochs
    }
    
    /// Check if in rampup phase
    pub fn is_rampup(&self) -> bool {
        self.current_epoch > self.warmup_epochs && 
        self.current_epoch <= self.warmup_epochs + self.rampup_epochs
    }
}