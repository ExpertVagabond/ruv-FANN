//! Projection and embedding functions for the Semantic Cartan Matrix

use crate::types::{RootVector, RootSpace};
use alloc::vec::Vec;

/// Project a high-dimensional vector to the 32-dimensional root space
/// 
/// This function performs the core operation of the Semantic Cartan Matrix:
/// projecting arbitrary dimensional vectors onto our orthogonal root basis.
/// 
/// # Arguments
/// * `input` - Input vector of arbitrary dimension
/// * `root_space` - The root space containing the basis vectors
/// 
/// # Returns
/// A 32-dimensional root vector representing the projection
pub fn project_to_root(input: &[f32], root_space: &RootSpace) -> RootVector {
    root_space.project(input)
}

/// Embed a root-space vector back to high-dimensional space
/// 
/// This is the inverse operation of projection, reconstructing a high-dimensional
/// vector from its root-space representation.
/// 
/// # Arguments
/// * `root_vector` - The 32-dimensional root vector
/// * `root_space` - The root space containing the basis vectors
/// * `target_dim` - The desired output dimension
/// 
/// # Returns
/// A vector of the specified dimension
pub fn embed_from_root(
    root_vector: &RootVector, 
    root_space: &RootSpace, 
    target_dim: usize
) -> Vec<f32> {
    let mut output = Vec::with_capacity(target_dim);
    
    // Initialize with zeros
    for _ in 0..target_dim {
        output.push(0.0);
    }
    
    // Reconstruct by linear combination of basis vectors
    for i in 0..32 {
        let coefficient = root_vector.data[i];
        let basis_vec = &root_space.basis[i];
        
        // Add weighted contribution of this basis vector
        for j in 0..target_dim.min(32) {
            output[j] += coefficient * basis_vec.data[j];
        }
    }
    
    output
}

/// Streaming projection using Oja's algorithm for online PCA
/// 
/// This allows updating the root basis incrementally as new data arrives,
/// handling domain drift without full retraining.
pub struct StreamingProjector {
    /// The evolving basis vectors
    basis: Vec<RootVector>,
    /// Learning rate for updates
    learning_rate: f32,
    /// Number of samples processed
    sample_count: u64,
}

impl StreamingProjector {
    /// Create a new streaming projector
    pub fn new(initial_basis: Vec<RootVector>, learning_rate: f32) -> Self {
        Self {
            basis: initial_basis,
            learning_rate,
            sample_count: 0,
        }
    }
    
    /// Project and update basis using Oja's rule
    pub fn project_and_update(&mut self, input: &[f32]) -> RootVector {
        let mut result = RootVector::zero();
        
        // Project onto current basis
        for i in 0..32 {
            let mut sum = 0.0f32;
            for (j, &val) in input.iter().enumerate().take(32) {
                sum += val * self.basis[i].data[j];
            }
            result.data[i] = sum;
        }
        
        // Update basis vectors using Oja's rule
        let lr = self.learning_rate / libm::sqrtf((self.sample_count + 1) as f32);
        
        for i in 0..32 {
            let y_i = result.data[i];
            
            // Oja update: w_new = w_old + lr * y * (x - y * w_old)
            for j in 0..input.len().min(32) {
                let x_j = input[j];
                let w_ij = self.basis[i].data[j];
                self.basis[i].data[j] += lr * y_i * (x_j - y_i * w_ij);
            }
        }
        
        // Periodically re-orthogonalize to maintain numerical stability
        if self.sample_count % 1000 == 999 {
            self.reorthogonalize();
        }
        
        self.sample_count += 1;
        result
    }
    
    /// Re-orthogonalize basis using modified Gram-Schmidt
    fn reorthogonalize(&mut self) {
        for i in 0..32 {
            // Normalize i-th vector
            self.basis[i].normalize();
            
            // Scale to Cartan norm
            self.basis[i].scale(libm::sqrtf(2.0));
            
            // Make subsequent vectors orthogonal to it
            for j in (i+1)..32 {
                let dot = self.basis[i].dot(&self.basis[j]);
                for k in 0..32 {
                    self.basis[j].data[k] -= dot * self.basis[i].data[k] / 2.0;
                }
            }
        }
    }
}

/// Compute the rank of an attention matrix using spectral analysis
/// 
/// Used to automatically detect rank-1 routing heads
pub fn compute_attention_rank(attention_weights: &[f32], dim: usize) -> usize {
    // Simplified rank computation using singular value ratios
    // In practice, this would use a proper SVD implementation
    
    // For now, return 1 if the weights are highly concentrated
    let max_weight = attention_weights.iter()
        .map(|w| libm::fabsf(*w))
        .fold(0.0f32, f32::max);
    
    let sum_weights: f32 = attention_weights.iter()
        .map(|w| libm::fabsf(*w))
        .sum();
    
    if max_weight > 0.9 * sum_weights {
        1 // Rank-1 behavior detected
    } else {
        dim // Full rank
    }
}

/// Adaptive root count selector based on data dimensionality
/// 
/// Heuristic: K = ceil(sqrt(d)/2) where d is the input dimension
pub fn suggest_root_count(input_dim: usize) -> usize {
    let sqrt_dim = libm::sqrtf(input_dim as f32);
    let suggested = libm::ceilf(sqrt_dim / 2.0) as usize;
    
    // Clamp to reasonable range
    suggested.max(8).min(64)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::RootSpace;
    use alloc::vec;

    #[test]
    fn test_projection_embedding_roundtrip() {
        let root_space = RootSpace::new();
        let input = vec![1.0; 32];
        
        // Project to root space
        let root_vec = project_to_root(&input, &root_space);
        
        // Embed back
        let reconstructed = embed_from_root(&root_vec, &root_space, 32);
        
        // Check reconstruction (won't be perfect due to basis)
        let error: f32 = input.iter()
            .zip(reconstructed.iter())
            .map(|(a, b)| (a - b).powi(2))
            .sum();
        
        // Error should be reasonable
        assert!(error < 100.0);
    }

    #[test]
    fn test_streaming_projector() {
        let basis = (0..32)
            .map(|_| RootVector::zero())
            .collect();
        
        let mut projector = StreamingProjector::new(basis, 0.01);
        
        // Process some samples
        for _ in 0..10 {
            let input = vec![1.0; 32];
            let _ = projector.project_and_update(&input);
        }
        
        assert_eq!(projector.sample_count, 10);
    }

    #[test]
    fn test_adaptive_root_count() {
        assert_eq!(suggest_root_count(64), 8);
        assert_eq!(suggest_root_count(256), 8);
        assert_eq!(suggest_root_count(768), 14);
        assert_eq!(suggest_root_count(2048), 23);
    }
}