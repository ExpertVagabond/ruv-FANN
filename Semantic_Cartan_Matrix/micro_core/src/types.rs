//! Core types for the Semantic Cartan Matrix implementation

use core::ops::{Index, IndexMut};
use core::fmt;
use alloc::vec::Vec;

/// A 32-dimensional root vector with SIMD alignment
/// 
/// This is the fundamental type representing vectors in the root space.
/// Aligned to 16 bytes for SIMD operations.
#[repr(align(16))]
#[derive(Clone, Copy)]
pub struct RootVector {
    /// The vector components
    pub data: [f32; 32],
}

impl RootVector {
    /// Create a new zero vector
    pub const fn zero() -> Self {
        Self { data: [0.0; 32] }
    }

    /// Create a new vector from an array
    pub const fn from_array(data: [f32; 32]) -> Self {
        Self { data }
    }

    /// Dot product with another vector
    pub fn dot(&self, other: &Self) -> f32 {
        #[cfg(feature = "simd")]
        {
            // SIMD implementation for faster computation
            self.dot_simd(other)
        }
        #[cfg(not(feature = "simd"))]
        {
            self.data.iter()
                .zip(other.data.iter())
                .map(|(a, b)| a * b)
                .sum()
        }
    }

    /// SIMD-accelerated dot product
    #[cfg(feature = "simd")]
    fn dot_simd(&self, other: &Self) -> f32 {
        // This would use platform-specific SIMD intrinsics
        // For now, fallback to scalar implementation
        self.data.iter()
            .zip(other.data.iter())
            .map(|(a, b)| a * b)
            .sum()
    }

    /// Normalize the vector to unit length
    pub fn normalize(&mut self) {
        let mag = self.magnitude();
        if mag > 0.0 {
            for i in 0..32 {
                self.data[i] /= mag;
            }
        }
    }

    /// Get the magnitude (length) of the vector
    pub fn magnitude(&self) -> f32 {
        libm::sqrtf(self.dot(self))
    }

    /// Scale the vector by a scalar
    pub fn scale(&mut self, scalar: f32) {
        for i in 0..32 {
            self.data[i] *= scalar;
        }
    }

    /// Get a slice view of the data
    pub fn as_slice(&self) -> &[f32] {
        &self.data[..]
    }

    /// Get a mutable slice view of the data
    pub fn as_mut_slice(&mut self) -> &mut [f32] {
        &mut self.data[..]
    }

    /// Create zero vector (alias for compatibility)
    pub fn zeros() -> Self {
        Self::zero()
    }
}

impl Default for RootVector {
    fn default() -> Self {
        Self::zero()
    }
}

impl Index<usize> for RootVector {
    type Output = f32;

    fn index(&self, index: usize) -> &Self::Output {
        &self.data[index]
    }
}

impl IndexMut<usize> for RootVector {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.data[index]
    }
}

impl fmt::Debug for RootVector {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "RootVector[")?;
        for (i, val) in self.data.iter().enumerate() {
            if i > 0 { write!(f, ", ")?; }
            write!(f, "{:.3}", val)?;
        }
        write!(f, "]")
    }
}

/// The 32-dimensional root space containing orthonormal basis vectors
pub struct RootSpace {
    /// The orthonormal basis vectors (rows of matrix H)
    pub basis: Vec<RootVector>,
    /// The Cartan matrix representation
    pub cartan: CartanMatrix,
}

impl RootSpace {
    /// Create a new root space with default initialization
    pub fn new() -> Self {
        let mut basis = Vec::with_capacity(32);
        
        // Initialize with identity-like basis
        for i in 0..32 {
            let mut vec = RootVector::zero();
            vec.data[i] = libm::sqrtf(2.0); // Cartan convention: ⟨αᵢ, αᵢ⟩ = 2
            basis.push(vec);
        }

        let cartan = CartanMatrix::default();
        
        Self { basis, cartan }
    }

    /// Initialize from a pre-computed basis
    pub fn from_basis(basis: Vec<RootVector>) -> Result<Self, &'static str> {
        if basis.len() != 32 {
            return Err("Root space must have exactly 32 basis vectors");
        }

        // Verify orthonormality with Cartan scaling
        for i in 0..32 {
            let self_dot = basis[i].dot(&basis[i]);
            if libm::fabsf(self_dot - 2.0) > 0.01 {
                return Err("Basis vectors must have Cartan norm sqrt(2)");
            }
            
            for j in (i+1)..32 {
                let cross_dot = basis[i].dot(&basis[j]);
                if libm::fabsf(cross_dot) > 0.01 {
                    return Err("Basis vectors must be orthogonal");
                }
            }
        }

        let cartan = CartanMatrix::from_basis(&basis);
        Ok(Self { basis, cartan })
    }

    /// Project a high-dimensional vector to root space
    pub fn project(&self, input: &[f32]) -> RootVector {
        let mut result = RootVector::zero();
        
        for i in 0..32 {
            let mut sum = 0.0f32;
            let basis_vec = &self.basis[i];
            
            // Compute dot product with i-th basis vector
            for (j, &val) in input.iter().enumerate().take(32) {
                sum += val * basis_vec.data[j];
            }
            
            result.data[i] = sum;
        }
        
        result
    }
}

/// The Cartan matrix representation
/// 
/// A 32x32 matrix with 2's on the diagonal and specific off-diagonal
/// values encoding the angle relationships between root vectors.
pub struct CartanMatrix {
    /// The matrix data in row-major order
    pub data: [[f32; 32]; 32],
}

impl CartanMatrix {
    /// Create a default Cartan matrix (diagonal)
    pub fn default() -> Self {
        let mut data = [[0.0; 32]; 32];
        
        // Initialize diagonal elements to 2
        for i in 0..32 {
            data[i][i] = 2.0;
        }
        
        Self { data }
    }

    /// Create from a basis of root vectors
    pub fn from_basis(basis: &[RootVector]) -> Self {
        let mut data = [[0.0; 32]; 32];
        
        // Compute Cartan matrix entries: C_ij = 2⟨αᵢ, αⱼ⟩/⟨αⱼ, αⱼ⟩
        for i in 0..32 {
            for j in 0..32 {
                let dot_ij = basis[i].dot(&basis[j]);
                let dot_jj = basis[j].dot(&basis[j]);
                data[i][j] = 2.0 * dot_ij / dot_jj;
            }
        }
        
        Self { data }
    }

    /// Compute the Frobenius norm difference from a target matrix
    pub fn frobenius_distance(&self, target: &Self) -> f32 {
        let mut sum = 0.0f32;
        
        for i in 0..32 {
            for j in 0..32 {
                let diff = self.data[i][j] - target.data[i][j];
                sum += diff * diff;
            }
        }
        
        libm::sqrtf(sum)
    }
}

impl Default for RootSpace {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_root_vector_creation() {
        let vec = RootVector::zero();
        assert_eq!(vec.data[0], 0.0);
        assert_eq!(vec.data[31], 0.0);
    }

    #[test]
    fn test_root_vector_dot_product() {
        let mut v1 = RootVector::zero();
        let mut v2 = RootVector::zero();
        
        v1.data[0] = 1.0;
        v2.data[0] = 2.0;
        
        assert_eq!(v1.dot(&v2), 2.0);
    }

    #[test]
    fn test_root_space_initialization() {
        let space = RootSpace::new();
        assert_eq!(space.basis.len(), 32);
        
        // Check Cartan normalization
        for vec in &space.basis {
            let norm_squared = vec.dot(vec);
            assert!((norm_squared - 2.0).abs() < 0.001);
        }
    }
}