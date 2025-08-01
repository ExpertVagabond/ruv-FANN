//! Cartan matrix implementation and root system management

use alloc::vec::Vec;
use micro_core::{RootVector, Result, Error, ROOT_DIM};
use nalgebra::{SMatrix, SVector};
use num_traits::Float;

#[cfg(feature = "serde")]
use serde::{Serialize, Deserialize};

/// Configuration for the Cartan matrix
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct CartanConfig {
    /// Diagonal values (should be 2.0 for standard Cartan matrices)
    pub diagonal_value: f32,
    
    /// Off-diagonal values (typically 0.0 for orthogonal, or -1.0, -2.0 for specific angles)
    pub off_diagonal_value: f32,
    
    /// Tolerance for orthogonality checks
    pub orthogonality_tolerance: f32,
    
    /// Whether to enforce strict Cartan normalization
    pub strict_normalization: bool,
}

impl Default for CartanConfig {
    fn default() -> Self {
        Self {
            diagonal_value: 2.0,
            off_diagonal_value: 0.0,
            orthogonality_tolerance: 1e-6,
            strict_normalization: true,
        }
    }
}

/// The Cartan matrix for the root system
/// 
/// This is a 32×32 matrix that encodes the desired inner product relationships
/// between root vectors in the semantic space.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct CartanMatrix {
    /// The matrix C encoding target inner products
    matrix: SMatrix<f32, ROOT_DIM, ROOT_DIM>,
    
    /// Configuration
    config: CartanConfig,
}

impl CartanMatrix {
    /// Create a new Cartan matrix with the given configuration
    pub fn new(config: CartanConfig) -> Self {
        let mut matrix = SMatrix::<f32, ROOT_DIM, ROOT_DIM>::zeros();
        
        // Set diagonal to diagonal_value
        for i in 0..ROOT_DIM {
            matrix[(i, i)] = config.diagonal_value;
        }
        
        // Set off-diagonals to off_diagonal_value
        for i in 0..ROOT_DIM {
            for j in 0..ROOT_DIM {
                if i != j {
                    matrix[(i, j)] = config.off_diagonal_value;
                }
            }
        }
        
        Self { matrix, config }
    }
    
    /// Create an identity-based Cartan matrix (orthogonal)
    pub fn identity() -> Self {
        Self::new(CartanConfig::default())
    }
    
    /// Create a Cartan matrix with specific off-diagonal structure
    /// 
    /// This allows for non-orthogonal root systems with controlled angles
    pub fn with_structure(off_diagonal_pattern: &[f32]) -> Result<Self> {
        if off_diagonal_pattern.len() != ROOT_DIM * (ROOT_DIM - 1) / 2 {
            return Err(Error::DimensionMismatch {
                expected: ROOT_DIM * (ROOT_DIM - 1) / 2,
                actual: off_diagonal_pattern.len(),
            });
        }
        
        let mut matrix = SMatrix::<f32, ROOT_DIM, ROOT_DIM>::zeros();
        
        // Set diagonal to 2.0
        for i in 0..ROOT_DIM {
            matrix[(i, i)] = 2.0;
        }
        
        // Set off-diagonals according to pattern
        let mut pattern_idx = 0;
        for i in 0..ROOT_DIM {
            for j in (i + 1)..ROOT_DIM {
                let value = off_diagonal_pattern[pattern_idx];
                matrix[(i, j)] = value;
                matrix[(j, i)] = value; // Symmetric
                pattern_idx += 1;
            }
        }
        
        Ok(Self {
            matrix,
            config: CartanConfig::default(),
        })
    }
    
    /// Get the target inner product between two root vectors
    pub fn target_inner_product(&self, i: usize, j: usize) -> f32 {
        if i < ROOT_DIM && j < ROOT_DIM {
            self.matrix[(i, j)]
        } else {
            0.0
        }
    }
    
    /// Compute the Cartan violation for a set of vectors
    /// 
    /// Returns the squared Frobenius norm of (C_actual - C_target)
    pub fn compute_violation(&self, vectors: &[RootVector]) -> f32 {
        if vectors.len() > ROOT_DIM {
            return f32::infinity();
        }
        
        let mut actual_matrix = SMatrix::<f32, ROOT_DIM, ROOT_DIM>::zeros();
        
        // Compute actual inner products
        for i in 0..vectors.len() {
            for j in 0..vectors.len() {
                actual_matrix[(i, j)] = vectors[i].dot(&vectors[j]);
            }
        }
        
        // Compute squared difference with target
        let diff = actual_matrix - self.matrix;
        diff.iter().map(|&x| x * x).sum()
    }
    
    /// Check if a set of vectors satisfies the Cartan constraints
    pub fn satisfies_constraints(&self, vectors: &[RootVector]) -> bool {
        let violation = self.compute_violation(vectors);
        violation < self.config.orthogonality_tolerance
    }
    
    /// Get the matrix as a reference
    pub fn matrix(&self) -> &SMatrix<f32, ROOT_DIM, ROOT_DIM> {
        &self.matrix
    }
    
    /// Get a specific row as a vector (for regularization)
    pub fn row(&self, i: usize) -> Option<SVector<f32, ROOT_DIM>> {
        if i < ROOT_DIM {
            Some(self.matrix.row(i).transpose())
        } else {
            None
        }
    }
    
    /// Update the Cartan matrix (for adaptive scenarios)
    pub fn update_entry(&mut self, i: usize, j: usize, value: f32) -> Result<()> {
        if i >= ROOT_DIM || j >= ROOT_DIM {
            return Err(Error::InvalidDimension { dim: i.max(j) });
        }
        
        self.matrix[(i, j)] = value;
        if i != j {
            self.matrix[(j, i)] = value; // Maintain symmetry
        }
        
        Ok(())
    }
}

/// Root system manager for the semantic space
#[derive(Debug, Clone)]
pub struct RootSystem {
    /// The current root vectors
    roots: Vec<RootVector>,
    
    /// Associated Cartan matrix
    cartan: CartanMatrix,
    
    /// Labels for semantic interpretation
    labels: Vec<Option<String>>,
}

impl RootSystem {
    /// Create a new root system
    pub fn new(cartan: CartanMatrix) -> Self {
        Self {
            roots: Vec::new(),
            cartan,
            labels: vec![None; ROOT_DIM],
        }
    }
    
    /// Add a root vector to the system
    pub fn add_root(&mut self, root: RootVector, label: Option<String>) -> Result<usize> {
        if self.roots.len() >= ROOT_DIM {
            return Err(Error::InvalidConfiguration(
                "Cannot add more than 32 root vectors".into()
            ));
        }
        
        let index = self.roots.len();
        self.roots.push(root);
        self.labels[index] = label;
        
        Ok(index)
    }
    
    /// Get a root vector by index
    pub fn get_root(&self, index: usize) -> Option<&RootVector> {
        self.roots.get(index)
    }
    
    /// Get all root vectors
    pub fn roots(&self) -> &[RootVector] {
        &self.roots
    }
    
    /// Check if the current root system satisfies Cartan constraints
    pub fn is_valid(&self) -> bool {
        self.cartan.satisfies_constraints(&self.roots)
    }
    
    /// Compute the current violation of Cartan constraints
    pub fn violation(&self) -> f32 {
        self.cartan.compute_violation(&self.roots)
    }
    
    /// Get the Cartan matrix
    pub fn cartan_matrix(&self) -> &CartanMatrix {
        &self.cartan
    }
    
    /// Set a semantic label for a root
    pub fn set_label(&mut self, index: usize, label: String) -> Result<()> {
        if index >= ROOT_DIM {
            return Err(Error::InvalidDimension { dim: index });
        }
        self.labels[index] = Some(label);
        Ok(())
    }
    
    /// Get the label for a root
    pub fn get_label(&self, index: usize) -> Option<&str> {
        self.labels.get(index)?.as_deref()
    }
}

/// Utilities for creating specific Cartan matrix types
pub mod presets {
    use super::*;
    
    /// Create an A_n type Cartan matrix (linear arrangement)
    pub fn a_type(n: usize) -> Result<CartanMatrix> {
        if n >= ROOT_DIM {
            return Err(Error::InvalidConfiguration(
                format!("A_{} type requires n < {}", n, ROOT_DIM)
            ));
        }
        
        let mut config = CartanConfig::default();
        config.off_diagonal_value = -1.0; // Adjacent roots have angle 120°
        
        let mut matrix = SMatrix::<f32, ROOT_DIM, ROOT_DIM>::zeros();
        
        // Diagonal entries
        for i in 0..ROOT_DIM {
            matrix[(i, i)] = 2.0;
        }
        
        // Off-diagonal: -1 for adjacent, 0 for non-adjacent
        for i in 0..n.min(ROOT_DIM - 1) {
            matrix[(i, i + 1)] = -1.0;
            matrix[(i + 1, i)] = -1.0;
        }
        
        Ok(CartanMatrix { matrix, config })
    }
    
    /// Create a D_n type Cartan matrix (orthogonal)
    pub fn d_type() -> CartanMatrix {
        CartanMatrix::identity()
    }
    
    /// Create an E_8 inspired structure (for the first 8 dimensions)
    pub fn e8_inspired() -> Result<CartanMatrix> {
        // Simplified E_8 structure - this is a placeholder
        // Real E_8 would require careful construction
        let mut matrix = SMatrix::<f32, ROOT_DIM, ROOT_DIM>::zeros();
        
        for i in 0..ROOT_DIM {
            matrix[(i, i)] = 2.0;
        }
        
        // Add some E_8-like connections for the first 8 roots
        let e8_connections = [
            (0, 1), (1, 2), (2, 3), (3, 4), (4, 5), (5, 6), (3, 7)
        ];
        
        for &(i, j) in &e8_connections {
            if i < ROOT_DIM && j < ROOT_DIM {
                matrix[(i, j)] = -1.0;
                matrix[(j, i)] = -1.0;
            }
        }
        
        Ok(CartanMatrix {
            matrix,
            config: CartanConfig::default(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_identity_cartan_matrix() {
        let cartan = CartanMatrix::identity();
        
        // Check diagonal values
        for i in 0..ROOT_DIM {
            assert_eq!(cartan.target_inner_product(i, i), 2.0);
        }
        
        // Check off-diagonal values
        for i in 0..ROOT_DIM {
            for j in 0..ROOT_DIM {
                if i != j {
                    assert_eq!(cartan.target_inner_product(i, j), 0.0);
                }
            }
        }
    }
    
    #[test]
    fn test_orthogonal_vectors_satisfy_constraints() {
        let cartan = CartanMatrix::identity();
        
        let mut vectors = Vec::new();
        for i in 0..3 {
            let mut v = RootVector::zeros();
            v[i] = 2.0_f32.sqrt(); // Norm sqrt(2) for Cartan normalization
            vectors.push(v);
        }
        
        assert!(cartan.satisfies_constraints(&vectors));
    }
}