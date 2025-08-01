//! Cartan-regularized attention mechanisms

use alloc::vec::Vec;
use micro_core::{RootVector, AttentionMechanism, Result, Error, ROOT_DIM};
use crate::{CartanMatrix, Orthogonalizer};
use nalgebra::{SMatrix, SVector, DMatrix, DVector};

#[cfg(feature = "serde")]
use serde::{Serialize, Deserialize};

/// Configuration for Cartan attention
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct AttentionConfig {
    /// Number of attention heads
    pub num_heads: usize,
    
    /// Dimension per head (should divide evenly into ROOT_DIM)
    pub head_dim: usize,
    
    /// Temperature for attention softmax
    pub temperature: f32,
    
    /// Regularization strength for Cartan constraints
    pub cartan_lambda: f32,
    
    /// Whether to enforce rank-1 constraints on some heads
    pub enforce_rank_one: bool,
    
    /// Fraction of heads to make rank-1 (0.0 to 1.0)
    pub rank_one_fraction: f32,
}

impl Default for AttentionConfig {
    fn default() -> Self {
        Self {
            num_heads: 8,
            head_dim: ROOT_DIM / 8,
            temperature: 1.0,
            cartan_lambda: 0.01,
            enforce_rank_one: true,
            rank_one_fraction: 0.3, // 30% of heads are rank-1 routing heads
        }
    }
}

/// A single attention head with Cartan regularization
#[derive(Debug, Clone)]
pub struct AttentionHead {
    /// Query projection matrix
    query_proj: SMatrix<f32, ROOT_DIM, ROOT_DIM>,
    
    /// Key projection matrix  
    key_proj: SMatrix<f32, ROOT_DIM, ROOT_DIM>,
    
    /// Value projection matrix
    value_proj: SMatrix<f32, ROOT_DIM, ROOT_DIM>,
    
    /// Whether this is a rank-1 head
    is_rank_one: bool,
    
    /// Head index
    head_idx: usize,
}

impl AttentionHead {
    /// Create a new attention head
    pub fn new(head_idx: usize, is_rank_one: bool) -> Self {
        // Initialize with small random weights
        let query_proj = SMatrix::<f32, ROOT_DIM, ROOT_DIM>::new_random() * 0.1;
        let key_proj = SMatrix::<f32, ROOT_DIM, ROOT_DIM>::new_random() * 0.1;
        let value_proj = SMatrix::<f32, ROOT_DIM, ROOT_DIM>::new_random() * 0.1;
        
        Self {
            query_proj,
            key_proj,
            value_proj,
            is_rank_one,
            head_idx,
        }
    }
    
    /// Compute attention for a set of vectors
    pub fn compute_attention(&self, 
                           vectors: &[RootVector], 
                           temperature: f32) -> Result<(Vec<RootVector>, SMatrix<f32, ROOT_DIM, ROOT_DIM>)> {
        if vectors.is_empty() {
            return Err(Error::InvalidConfiguration("Empty input vectors".into()));
        }
        
        let seq_len = vectors.len();
        
        // Project to Q, K, V
        let mut queries = Vec::with_capacity(seq_len);
        let mut keys = Vec::with_capacity(seq_len);
        let mut values = Vec::with_capacity(seq_len);
        
        for vector in vectors {
            let q = self.query_proj * vector.as_slice();
            let k = self.key_proj * vector.as_slice();
            let v = self.value_proj * vector.as_slice();
            
            queries.push(RootVector::from_slice(&q.data.as_slice()).unwrap());
            keys.push(RootVector::from_slice(&k.data.as_slice()).unwrap());
            values.push(RootVector::from_slice(&v.data.as_slice()).unwrap());
        }
        
        // Compute attention scores (Q * K^T)
        let mut attention_matrix = SMatrix::<f32, ROOT_DIM, ROOT_DIM>::zeros();
        for i in 0..seq_len.min(ROOT_DIM) {
            for j in 0..seq_len.min(ROOT_DIM) {
                let score = queries[i].dot(&keys[j]) / temperature;
                attention_matrix[(i, j)] = score;
            }
        }
        
        // Apply rank-1 constraint if needed
        if self.is_rank_one {
            self.apply_rank_one_constraint(&mut attention_matrix);
        }
        
        // Softmax normalization
        self.softmax_inplace(&mut attention_matrix, seq_len);
        
        // Apply attention to values
        let mut attended_vectors = Vec::with_capacity(seq_len);
        for i in 0..seq_len {
            let mut attended = RootVector::zeros();
            for j in 0..seq_len.min(ROOT_DIM) {
                let weight = attention_matrix[(i, j)];
                let weighted_value = values[j].map(|x| x * weight);
                *attended += weighted_value;
            }
            attended_vectors.push(attended);
        }
        
        Ok((attended_vectors, attention_matrix))
    }
    
    /// Apply rank-1 constraint to attention matrix
    fn apply_rank_one_constraint(&self, attention_matrix: &mut SMatrix<f32, ROOT_DIM, ROOT_DIM>) {
        // For rank-1, we want the matrix to be of the form u * v^T
        // Simple approximation: use the first singular vector
        
        // Compute row and column means as approximation to rank-1 factorization
        let mut row_mean = SVector::<f32, ROOT_DIM>::zeros();
        let mut col_mean = SVector::<f32, ROOT_DIM>::zeros();
        
        for i in 0..ROOT_DIM {
            for j in 0..ROOT_DIM {
                row_mean[i] += attention_matrix[(i, j)];
                col_mean[j] += attention_matrix[(i, j)];
            }
        }
        
        // Normalize
        let row_norm = row_mean.norm();
        let col_norm = col_mean.norm();
        
        if row_norm > 1e-8 && col_norm > 1e-8 {
            row_mean /= row_norm;
            col_mean /= col_norm;
            
            // Reconstruct as rank-1: u * v^T
            for i in 0..ROOT_DIM {
                for j in 0..ROOT_DIM {
                    attention_matrix[(i, j)] = row_mean[i] * col_mean[j];
                }
            }
        }
    }
    
    /// Apply softmax to attention matrix rows
    fn softmax_inplace(&self, matrix: &mut SMatrix<f32, ROOT_DIM, ROOT_DIM>, seq_len: usize) {
        for i in 0..seq_len.min(ROOT_DIM) {
            // Find max for numerical stability
            let mut max_val = f32::NEG_INFINITY;
            for j in 0..seq_len.min(ROOT_DIM) {
                max_val = max_val.max(matrix[(i, j)]);
            }
            
            // Exp and sum
            let mut sum = 0.0;
            for j in 0..seq_len.min(ROOT_DIM) {
                matrix[(i, j)] = (matrix[(i, j)] - max_val).exp();
                sum += matrix[(i, j)];
            }
            
            // Normalize
            if sum > 1e-8 {
                for j in 0..seq_len.min(ROOT_DIM) {
                    matrix[(i, j)] /= sum;
                }
            }
        }
    }
    
    /// Check if this is a rank-1 head
    pub fn is_rank_one(&self) -> bool {
        self.is_rank_one
    }
    
    /// Get head index
    pub fn head_index(&self) -> usize {
        self.head_idx
    }
}

/// Multi-head Cartan attention mechanism
#[derive(Debug)]
pub struct CartanAttention {
    /// Configuration
    config: AttentionConfig,
    
    /// Attention heads
    heads: Vec<AttentionHead>,
    
    /// Output projection
    output_proj: SMatrix<f32, ROOT_DIM, ROOT_DIM>,
    
    /// Cartan matrix for regularization
    cartan_matrix: CartanMatrix,
    
    /// Orthogonalizer for post-processing
    orthogonalizer: Orthogonalizer,
}

impl CartanAttention {
    /// Create a new Cartan attention mechanism
    pub fn new(config: AttentionConfig, cartan_matrix: CartanMatrix) -> Self {
        let mut heads = Vec::with_capacity(config.num_heads);
        
        // Determine which heads should be rank-1
        let num_rank_one = (config.num_heads as f32 * config.rank_one_fraction).round() as usize;
        
        for i in 0..config.num_heads {
            let is_rank_one = i < num_rank_one && config.enforce_rank_one;
            heads.push(AttentionHead::new(i, is_rank_one));
        }
        
        let output_proj = SMatrix::<f32, ROOT_DIM, ROOT_DIM>::new_random() * 0.1;
        let orthogonalizer = Orthogonalizer::new();
        
        Self {
            config,
            heads,
            output_proj,
            cartan_matrix,
            orthogonalizer,
        }
    }
    
    /// Forward pass through all attention heads
    pub fn forward(&self, vectors: &[RootVector]) -> Result<Vec<RootVector>> {
        if vectors.is_empty() {
            return Ok(Vec::new());
        }
        
        let mut head_outputs = Vec::with_capacity(self.config.num_heads);
        let mut all_attention_matrices = Vec::with_capacity(self.config.num_heads);
        
        // Process each head
        for head in &self.heads {
            let (attended, attention_matrix) = head.compute_attention(vectors, self.config.temperature)?;
            head_outputs.push(attended);
            all_attention_matrices.push(attention_matrix);
        }
        
        // Combine head outputs (simple concatenation and projection)
        let mut combined_outputs = Vec::with_capacity(vectors.len());
        
        for seq_idx in 0..vectors.len() {
            // Concatenate outputs from all heads for this sequence position
            let mut combined = RootVector::zeros();
            
            // Simple averaging of head outputs (could be more sophisticated)
            for head_output in &head_outputs {
                if seq_idx < head_output.len() {
                    *combined += head_output[seq_idx];
                }
            }
            
            // Scale by number of heads
            *combined = combined.map(|x| x / self.config.num_heads as f32);
            
            // Apply output projection
            let projected = self.output_proj * combined.as_slice();
            let projected_vector = RootVector::from_slice(projected.as_slice()).unwrap();
            
            combined_outputs.push(projected_vector);
        }
        
        // Apply Cartan regularization and orthogonalization
        self.apply_cartan_constraints(&mut combined_outputs)?;
        
        Ok(combined_outputs)
    }
    
    /// Apply Cartan constraints to the outputs
    fn apply_cartan_constraints(&self, outputs: &mut [RootVector]) -> Result<()> {
        // Check current violation
        let violation = self.cartan_matrix.compute_violation(outputs);
        
        // If violation is too high, apply orthogonalization
        if violation > self.config.cartan_lambda {
            self.orthogonalizer.orthogonalize_vectors(outputs)?;
        }
        
        Ok(())
    }
    
    /// Get the number of rank-1 heads
    pub fn num_rank_one_heads(&self) -> usize {
        self.heads.iter().filter(|h| h.is_rank_one()).count()
    }
    
    /// Get the current Cartan violation for a set of vectors
    pub fn compute_cartan_violation(&self, vectors: &[RootVector]) -> f32 {
        self.cartan_matrix.compute_violation(vectors)
    }
}

impl AttentionMechanism for CartanAttention {
    fn apply_attention(&self, vectors: &[RootVector]) -> Result<Vec<RootVector>> {
        self.forward(vectors)
    }
    
    fn get_attention_matrix(&self, vectors: &[RootVector]) -> Result<Vec<Vec<f32>>> {
        // Return the attention matrix from the first head for visualization
        if let Some(head) = self.heads.first() {
            let (_, attention_matrix) = head.compute_attention(vectors, self.config.temperature)?;
            
            let mut result = Vec::with_capacity(ROOT_DIM);
            for i in 0..ROOT_DIM {
                let mut row = Vec::with_capacity(ROOT_DIM);
                for j in 0..ROOT_DIM {
                    row.push(attention_matrix[(i, j)]);
                }
                result.push(row);
            }
            
            Ok(result)
        } else {
            Err(Error::InvalidConfiguration("No attention heads available".into()))
        }
    }
    
    fn is_rank_one(&self) -> bool {
        self.num_rank_one_heads() > 0
    }
}