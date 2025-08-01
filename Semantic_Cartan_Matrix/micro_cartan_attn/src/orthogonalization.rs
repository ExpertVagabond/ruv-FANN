//! Orthogonalization methods for maintaining Cartan constraints

use alloc::vec::Vec;
use micro_core::{RootVector, Result, Error, ROOT_DIM};
use nalgebra::{QR, DMatrix};

#[cfg(feature = "serde")]
use serde::{Serialize, Deserialize};

/// Methods for orthogonalization
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum OrthogonalizationMethod {
    /// Gram-Schmidt orthogonalization
    GramSchmidt,
    /// QR decomposition
    QRDecomposition,
    /// Symmetric orthogonalization (Löwdin)
    Symmetric,
}

/// Orthogonalizer for maintaining vector orthogonality
#[derive(Debug, Clone)]
pub struct Orthogonalizer {
    /// Method to use
    method: OrthogonalizationMethod,
    
    /// Tolerance for numerical precision
    tolerance: f32,
    
    /// Whether to preserve vector norms
    preserve_norms: bool,
}

impl Orthogonalizer {
    /// Create a new orthogonalizer
    pub fn new() -> Self {
        Self {
            method: OrthogonalizationMethod::QRDecomposition,
            tolerance: 1e-6,
            preserve_norms: false,
        }
    }
    
    /// Create with specific method
    pub fn with_method(method: OrthogonalizationMethod) -> Self {
        Self {
            method,
            tolerance: 1e-6,
            preserve_norms: false,
        }
    }
    
    /// Set tolerance for numerical precision
    pub fn with_tolerance(mut self, tolerance: f32) -> Self {
        self.tolerance = tolerance;
        self
    }
    
    /// Set whether to preserve vector norms
    pub fn preserve_norms(mut self, preserve: bool) -> Self {
        self.preserve_norms = preserve;
        self
    }
    
    /// Orthogonalize a set of root vectors in place
    pub fn orthogonalize_vectors(&self, vectors: &mut [RootVector]) -> Result<()> {
        if vectors.is_empty() {
            return Ok(());
        }
        
        // Store original norms if needed
        let original_norms: Vec<f32> = if self.preserve_norms {
            vectors.iter().map(|v| v.norm()).collect()
        } else {
            Vec::new()
        };
        
        match self.method {
            OrthogonalizationMethod::GramSchmidt => {
                self.gram_schmidt_orthogonalize(vectors)?;
            }
            OrthogonalizationMethod::QRDecomposition => {
                self.qr_orthogonalize(vectors)?;
            }
            OrthogonalizationMethod::Symmetric => {
                self.symmetric_orthogonalize(vectors)?;
            }
        }
        
        // Restore norms if requested
        if self.preserve_norms {
            for (vector, &original_norm) in vectors.iter_mut().zip(original_norms.iter()) {
                let current_norm = vector.norm();
                if current_norm > self.tolerance {
                    *vector = vector.map(|x| x * original_norm / current_norm);
                }
            }
        }
        
        Ok(())
    }
    
    /// Gram-Schmidt orthogonalization
    fn gram_schmidt_orthogonalize(&self, vectors: &mut [RootVector]) -> Result<()> {
        for i in 0..vectors.len() {
            // Orthogonalize vector i against all previous vectors
            for j in 0..i {
                let projection_coeff = vectors[i].dot(&vectors[j]) / vectors[j].dot(&vectors[j]);
                
                // Subtract projection: v_i = v_i - proj_coeff * v_j
                for k in 0..ROOT_DIM {
                    vectors[i][k] -= projection_coeff * vectors[j][k];
                }
            }
            
            // Normalize if not preserving norms
            if !self.preserve_norms {
                let norm = vectors[i].norm();
                if norm > self.tolerance {
                    *vectors[i] = vectors[i].map(|x| x / norm);
                }
            }
        }
        
        Ok(())
    }
    
    /// QR decomposition orthogonalization
    fn qr_orthogonalize(&self, vectors: &mut [RootVector]) -> Result<()> {
        if vectors.len() > ROOT_DIM {
            return Err(Error::DimensionMismatch {
                expected: ROOT_DIM,
                actual: vectors.len(),
            });
        }
        
        // Build matrix from vectors (columns)
        let mut matrix = DMatrix::<f32>::zeros(ROOT_DIM, vectors.len());
        for (j, vector) in vectors.iter().enumerate() {
            for i in 0..ROOT_DIM {
                matrix[(i, j)] = vector[i];
            }
        }
        
        // QR decomposition
        let qr = QR::new(matrix);
        let q = qr.q();
        
        // Extract orthogonal vectors from Q
        for (j, vector) in vectors.iter_mut().enumerate() {
            for i in 0..ROOT_DIM {
                vector[i] = q[(i, j)];
            }
        }
        
        Ok(())
    }
    
    /// Symmetric (Löwdin) orthogonalization
    fn symmetric_orthogonalize(&self, vectors: &mut [RootVector]) -> Result<()> {
        if vectors.len() > ROOT_DIM {
            return Err(Error::DimensionMismatch {
                expected: ROOT_DIM,
                actual: vectors.len(),
            });
        }
        
        // Build overlap matrix S = V^T * V
        let n = vectors.len();
        let mut overlap = DMatrix::<f32>::zeros(n, n);
        
        for i in 0..n {
            for j in 0..n {
                overlap[(i, j)] = vectors[i].dot(&vectors[j]);
            }
        }
        
        // Compute S^(-1/2) using eigendecomposition
        // This is simplified - in practice you'd use proper eigendecomposition
        let mut s_inv_sqrt = DMatrix::<f32>::identity(n, n);
        
        // Simple diagonal approximation for now
        for i in 0..n {
            let diag_val = overlap[(i, i)];
            if diag_val > self.tolerance {
                s_inv_sqrt[(i, i)] = 1.0 / diag_val.sqrt();
            }
        }
        
        // Apply transformation: V_new = V * S^(-1/2)
        let mut new_vectors = vec![RootVector::zeros(); n];
        
        for i in 0..n {
            for j in 0..n {
                for k in 0..ROOT_DIM {
                    new_vectors[i][k] += vectors[j][k] * s_inv_sqrt[(j, i)];
                }
            }
        }
        
        // Copy back
        for (orig, new) in vectors.iter_mut().zip(new_vectors.iter()) {
            *orig = *new;
        }
        
        Ok(())
    }
    
    /// Check if vectors are orthogonal within tolerance
    pub fn check_orthogonality(&self, vectors: &[RootVector]) -> bool {
        for i in 0..vectors.len() {
            for j in (i + 1)..vectors.len() {
                let dot_product = vectors[i].dot(&vectors[j]);
                if dot_product.abs() > self.tolerance {
                    return false;
                }
            }
        }
        true
    }
    
    /// Compute orthogonality violation (sum of squared off-diagonal terms)
    pub fn compute_orthogonality_violation(&self, vectors: &[RootVector]) -> f32 {
        let mut violation = 0.0;
        
        for i in 0..vectors.len() {
            for j in (i + 1)..vectors.len() {
                let dot_product = vectors[i].dot(&vectors[j]);
                violation += dot_product * dot_product;
            }
        }
        
        violation
    }
}

impl Default for Orthogonalizer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use micro_core::RootVector;
    
    #[test]
    fn test_gram_schmidt_orthogonalization() {
        let mut vectors = vec![
            RootVector::from([1.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]),
            RootVector::from([1.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]),
        ];
        
        let orthogonalizer = Orthogonalizer::with_method(OrthogonalizationMethod::GramSchmidt);
        orthogonalizer.orthogonalize_vectors(&mut vectors).unwrap();
        
        assert!(orthogonalizer.check_orthogonality(&vectors));
    }
    
    #[test]
    fn test_qr_orthogonalization() {
        let mut vectors = vec![
            RootVector::from([2.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]),
            RootVector::from([0.5, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]),
        ];
        
        let orthogonalizer = Orthogonalizer::with_method(OrthogonalizationMethod::QRDecomposition);
        orthogonalizer.orthogonalize_vectors(&mut vectors).unwrap();
        
        assert!(orthogonalizer.check_orthogonality(&vectors));
    }
}