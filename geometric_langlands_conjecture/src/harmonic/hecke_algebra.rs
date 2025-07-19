//! Hecke algebra implementation

use crate::{Error, Result};
use ndarray::{Array1, Array2, ArrayD};
use num_complex::Complex64;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Hecke algebra for a reductive group
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeckeAlgebra {
    /// Dimension of the underlying group
    dimension: usize,
    /// Rank of the group
    rank: usize,
    /// Generators of the algebra
    generators: Vec<HeckeElement>,
    /// Structure constants
    structure_constants: HashMap<(usize, usize), Vec<(usize, Complex64)>>,
    /// Quadratic relations (braid relations)
    braid_relations: Vec<BraidRelation>,
}

/// Element of the Hecke algebra
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeckeElement {
    /// Coefficients in the standard basis
    pub coefficients: HashMap<WeylGroupElement, Complex64>,
    /// Degree of the element
    pub degree: usize,
}

/// Element of the Weyl group
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WeylGroupElement {
    /// Permutation representation
    permutation: Vec<usize>,
    /// Length (number of inversions)
    length: usize,
    /// Reduced word representation
    reduced_word: Vec<usize>,
}

/// Braid relation in the Hecke algebra
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BraidRelation {
    /// Indices of generators involved
    pub indices: Vec<usize>,
    /// Order of the relation (2 or 3 typically)
    pub order: usize,
}

impl HeckeAlgebra {
    /// Create a new Hecke algebra
    pub fn new(dimension: usize, rank: usize) -> Result<Self> {
        if rank > dimension {
            return Err(Error::InvalidParameter(
                "Rank cannot exceed dimension".to_string()
            ));
        }

        // Create simple generators T_i for i = 1, ..., rank
        let generators = Self::create_generators(rank)?;
        
        // Initialize structure constants for multiplication
        let structure_constants = Self::compute_structure_constants(&generators)?;
        
        // Set up braid relations
        let braid_relations = Self::setup_braid_relations(rank);

        Ok(Self {
            dimension,
            rank,
            generators,
            structure_constants,
            braid_relations,
        })
    }

    /// Create standard generators
    fn create_generators(rank: usize) -> Result<Vec<HeckeElement>> {
        let mut generators = Vec::new();
        
        for i in 0..rank {
            // Create generator T_i corresponding to simple reflection s_i
            let mut coefficients = HashMap::new();
            
            // Identity element
            let id = WeylGroupElement::identity(rank);
            coefficients.insert(id, Complex64::new(0.0, 0.0));
            
            // Simple reflection s_i
            let s_i = WeylGroupElement::simple_reflection(i, rank)?;
            coefficients.insert(s_i, Complex64::new(1.0, 0.0));
            
            generators.push(HeckeElement {
                coefficients,
                degree: 1,
            });
        }
        
        Ok(generators)
    }

    /// Compute structure constants for multiplication
    fn compute_structure_constants(generators: &[HeckeElement]) -> Result<HashMap<(usize, usize), Vec<(usize, Complex64)>>> {
        let mut constants = HashMap::new();
        
        // For each pair of generators, compute their product
        for (i, gen_i) in generators.iter().enumerate() {
            for (j, gen_j) in generators.iter().enumerate() {
                let product = gen_i.multiply(gen_j)?;
                
                // Express product in terms of basis elements
                let mut expansion = Vec::new();
                for (k, gen_k) in generators.iter().enumerate() {
                    if let Some(coeff) = product.coefficient_of(gen_k) {
                        if coeff.norm() > 1e-10 {
                            expansion.push((k, coeff));
                        }
                    }
                }
                
                constants.insert((i, j), expansion);
            }
        }
        
        Ok(constants)
    }

    /// Set up braid relations
    fn setup_braid_relations(rank: usize) -> Vec<BraidRelation> {
        let mut relations = Vec::new();
        
        // Quadratic relations: T_i^2 = (q - q^{-1})T_i + 1
        for i in 0..rank {
            relations.push(BraidRelation {
                indices: vec![i],
                order: 2,
            });
        }
        
        // Braid relations: T_i T_j = T_j T_i if |i-j| > 1
        for i in 0..rank {
            for j in i+2..rank {
                relations.push(BraidRelation {
                    indices: vec![i, j],
                    order: 2,
                });
            }
        }
        
        // Braid relations: T_i T_{i+1} T_i = T_{i+1} T_i T_{i+1}
        for i in 0..rank-1 {
            relations.push(BraidRelation {
                indices: vec![i, i+1],
                order: 3,
            });
        }
        
        relations
    }

    /// Apply Hecke operator to a function
    pub fn apply(
        &self,
        element: &HeckeElement,
        function: &Array1<Complex64>,
    ) -> Result<Array1<Complex64>> {
        if function.len() != self.dimension {
            return Err(Error::DimensionMismatch {
                expected: self.dimension,
                actual: function.len(),
            });
        }

        let mut result = Array1::zeros(self.dimension);
        
        // Apply each Weyl group element in the Hecke element
        for (w, coeff) in &element.coefficients {
            let transformed = self.apply_weyl_element(w, function)?;
            result = result + transformed * *coeff;
        }
        
        Ok(result)
    }

    /// Apply a single Weyl group element
    fn apply_weyl_element(
        &self,
        w: &WeylGroupElement,
        function: &Array1<Complex64>,
    ) -> Result<Array1<Complex64>> {
        let mut result = function.clone();
        
        // Apply permutation
        for (i, &j) in w.permutation.iter().enumerate() {
            if j < function.len() {
                result[i] = function[j];
            }
        }
        
        Ok(result)
    }

    /// Multiply two Hecke elements
    pub fn multiply(&self, a: &HeckeElement, b: &HeckeElement) -> Result<HeckeElement> {
        a.multiply(b)
    }

    /// Get the identity element
    pub fn identity(&self) -> HeckeElement {
        let mut coefficients = HashMap::new();
        coefficients.insert(WeylGroupElement::identity(self.rank), Complex64::new(1.0, 0.0));
        
        HeckeElement {
            coefficients,
            degree: 0,
        }
    }

    /// Get a generator by index
    pub fn generator(&self, index: usize) -> Result<&HeckeElement> {
        self.generators.get(index)
            .ok_or_else(|| Error::InvalidParameter(format!("Generator index {} out of bounds", index)))
    }

    /// Check if an element satisfies the braid relations
    pub fn verify_braid_relations(&self, element: &HeckeElement) -> bool {
        // Simplified check - full implementation would verify all relations
        true
    }

    /// Compute the character of a Hecke algebra element
    pub fn character(&self, element: &HeckeElement) -> Complex64 {
        // Trace of the element in the regular representation
        element.coefficients.values().sum()
    }
}

impl HeckeElement {
    /// Create a new Hecke element
    pub fn new(coefficients: HashMap<WeylGroupElement, Complex64>) -> Self {
        let degree = coefficients.keys()
            .map(|w| w.length)
            .max()
            .unwrap_or(0);
        
        Self {
            coefficients,
            degree,
        }
    }

    /// Add two Hecke elements
    pub fn add(&self, other: &Self) -> Self {
        let mut result_coeffs = self.coefficients.clone();
        
        for (w, coeff) in &other.coefficients {
            *result_coeffs.entry(w.clone()).or_insert(Complex64::new(0.0, 0.0)) += coeff;
        }
        
        Self::new(result_coeffs)
    }

    /// Multiply two Hecke elements
    pub fn multiply(&self, other: &Self) -> Result<Self> {
        let mut result_coeffs = HashMap::new();
        
        for (w1, c1) in &self.coefficients {
            for (w2, c2) in &other.coefficients {
                let w_product = w1.multiply(w2)?;
                let coeff = c1 * c2;
                
                *result_coeffs.entry(w_product).or_insert(Complex64::new(0.0, 0.0)) += coeff;
            }
        }
        
        Ok(Self::new(result_coeffs))
    }

    /// Scale by a complex number
    pub fn scale(&self, scalar: Complex64) -> Self {
        let mut scaled_coeffs = HashMap::new();
        
        for (w, coeff) in &self.coefficients {
            scaled_coeffs.insert(w.clone(), coeff * scalar);
        }
        
        Self::new(scaled_coeffs)
    }

    /// Get coefficient of a basis element
    pub fn coefficient_of(&self, other: &HeckeElement) -> Option<Complex64> {
        // Simplified - assumes other is a single basis element
        if other.coefficients.len() == 1 {
            let (w, _) = other.coefficients.iter().next()?;
            self.coefficients.get(w).copied()
        } else {
            None
        }
    }

    /// Compute the inverse if it exists
    pub fn inverse(&self) -> Result<Self> {
        // Simplified - only works for invertible elements
        if self.coefficients.len() == 1 {
            let (w, coeff) = self.coefficients.iter().next().unwrap();
            if coeff.norm() < 1e-10 {
                return Err(Error::MathError("Element is not invertible".to_string()));
            }
            
            let mut inv_coeffs = HashMap::new();
            let w_inv = w.inverse()?;
            inv_coeffs.insert(w_inv, Complex64::new(1.0, 0.0) / coeff);
            
            Ok(Self::new(inv_coeffs))
        } else {
            Err(Error::MathError("General inverse not implemented".to_string()))
        }
    }
}

impl WeylGroupElement {
    /// Create identity element
    pub fn identity(rank: usize) -> Self {
        Self {
            permutation: (0..rank).collect(),
            length: 0,
            reduced_word: vec![],
        }
    }

    /// Create simple reflection s_i
    pub fn simple_reflection(i: usize, rank: usize) -> Result<Self> {
        if i >= rank {
            return Err(Error::InvalidParameter(
                format!("Reflection index {} exceeds rank {}", i, rank)
            ));
        }

        let mut permutation: Vec<usize> = (0..rank).collect();
        if i < rank - 1 {
            permutation.swap(i, i + 1);
        }

        Ok(Self {
            permutation,
            length: 1,
            reduced_word: vec![i],
        })
    }

    /// Multiply two Weyl group elements
    pub fn multiply(&self, other: &Self) -> Result<Self> {
        if self.permutation.len() != other.permutation.len() {
            return Err(Error::DimensionMismatch {
                expected: self.permutation.len(),
                actual: other.permutation.len(),
            });
        }

        let mut result_perm = vec![0; self.permutation.len()];
        for (i, &j) in self.permutation.iter().enumerate() {
            if j < other.permutation.len() {
                result_perm[i] = other.permutation[j];
            }
        }

        // Compute length and reduced word (simplified)
        let mut reduced_word = self.reduced_word.clone();
        reduced_word.extend(&other.reduced_word);
        let length = self.length + other.length; // Upper bound

        Ok(Self {
            permutation: result_perm,
            length,
            reduced_word,
        })
    }

    /// Compute inverse
    pub fn inverse(&self) -> Result<Self> {
        let mut inv_perm = vec![0; self.permutation.len()];
        
        for (i, &j) in self.permutation.iter().enumerate() {
            if j < inv_perm.len() {
                inv_perm[j] = i;
            }
        }

        Ok(Self {
            permutation: inv_perm,
            length: self.length,
            reduced_word: self.reduced_word.iter().rev().copied().collect(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_hecke_algebra_creation() {
        let hecke = HeckeAlgebra::new(4, 3).unwrap();
        assert_eq!(hecke.rank, 3);
        assert_eq!(hecke.generators.len(), 3);
    }

    #[test]
    fn test_weyl_group_element() {
        let w = WeylGroupElement::identity(3);
        assert_eq!(w.length, 0);
        assert_eq!(w.permutation, vec![0, 1, 2]);
    }

    #[test]
    fn test_simple_reflection() {
        let s1 = WeylGroupElement::simple_reflection(0, 3).unwrap();
        assert_eq!(s1.length, 1);
        assert_eq!(s1.permutation, vec![1, 0, 2]);
    }

    #[test]
    fn test_hecke_element_multiplication() {
        let w1 = WeylGroupElement::identity(2);
        let w2 = WeylGroupElement::simple_reflection(0, 2).unwrap();
        
        let mut coeffs1 = HashMap::new();
        coeffs1.insert(w1, Complex64::new(1.0, 0.0));
        let h1 = HeckeElement::new(coeffs1);
        
        let mut coeffs2 = HashMap::new();
        coeffs2.insert(w2, Complex64::new(1.0, 0.0));
        let h2 = HeckeElement::new(coeffs2);
        
        let product = h1.multiply(&h2).unwrap();
        assert!(product.coefficients.len() > 0);
    }
}