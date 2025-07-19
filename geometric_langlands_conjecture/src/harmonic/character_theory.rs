//! Character theory for representations

use crate::{Error, Result};
use ndarray::{Array1, Array2, ArrayD};
use num_complex::Complex64;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Character of a representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Character {
    /// Dimension of the representation
    dimension: usize,
    /// Character values on conjugacy classes
    values: HashMap<ConjugacyClass, Complex64>,
    /// Whether the character is irreducible
    is_irreducible: bool,
    /// Schur indicator (+1, 0, -1)
    schur_indicator: i8,
}

/// Conjugacy class in a group
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ConjugacyClass {
    /// Representative element (as permutation cycle type)
    cycle_type: Vec<usize>,
    /// Size of the conjugacy class
    size: usize,
    /// Order of elements in the class
    order: usize,
}

/// Character table for a group
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterTable {
    /// Dimension of the group
    group_dimension: usize,
    /// Rank of the group
    rank: usize,
    /// List of conjugacy classes
    conjugacy_classes: Vec<ConjugacyClass>,
    /// Irreducible characters
    irreducible_characters: Vec<Character>,
    /// Character inner product matrix
    inner_product_matrix: Array2<Complex64>,
}

/// Weyl character for highest weight representations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeylCharacter {
    /// Highest weight
    highest_weight: Array1<i32>,
    /// Dimension of the representation
    dimension: usize,
    /// Character polynomial (as coefficients)
    character_polynomial: Vec<Complex64>,
    /// Dominant weights appearing in the character
    weight_multiplicities: HashMap<Array1<i32>, usize>,
}

impl Character {
    /// Create a new character
    pub fn new(dimension: usize) -> Self {
        Self {
            dimension,
            values: HashMap::new(),
            is_irreducible: false,
            schur_indicator: 0,
        }
    }

    /// Create the trivial character
    pub fn trivial(group_dimension: usize) -> Self {
        let mut character = Self::new(1);
        character.is_irreducible = true;
        character.schur_indicator = 1;
        
        // Trivial character is 1 on all conjugacy classes
        // (Will be populated when conjugacy classes are known)
        character
    }

    /// Create character from representation matrix
    pub fn from_representation(representation: &Array2<Complex64>) -> Result<Self> {
        let dimension = representation.nrows();
        if dimension != representation.ncols() {
            return Err(Error::InvalidParameter(
                "Representation must be square matrix".to_string()
            ));
        }

        let mut character = Self::new(dimension);
        
        // Compute trace (character at identity)
        let trace = (0..dimension)
            .map(|i| representation[[i, i]])
            .sum::<Complex64>();
        
        // Add value for identity conjugacy class
        let identity_class = ConjugacyClass {
            cycle_type: vec![1; dimension],
            size: 1,
            order: 1,
        };
        character.values.insert(identity_class, trace);
        
        Ok(character)
    }

    /// Evaluate character at group element
    pub fn evaluate(&self, conjugacy_class: &ConjugacyClass) -> Complex64 {
        self.values.get(conjugacy_class)
            .copied()
            .unwrap_or(Complex64::new(0.0, 0.0))
    }

    /// Inner product of two characters
    pub fn inner_product(&self, other: &Character, table: &CharacterTable) -> Complex64 {
        let mut product = Complex64::new(0.0, 0.0);
        
        for class in &table.conjugacy_classes {
            let chi_self = self.evaluate(class);
            let chi_other = other.evaluate(class);
            product += chi_self * chi_other.conj() * class.size as f64;
        }
        
        product / table.group_dimension as f64
    }

    /// Decompose character into irreducibles
    pub fn decompose(&self, table: &CharacterTable) -> Result<Vec<(usize, usize)>> {
        let mut decomposition = Vec::new();
        
        for (i, irrep) in table.irreducible_characters.iter().enumerate() {
            let multiplicity = self.inner_product(irrep, table);
            
            // Multiplicity should be a non-negative integer
            let mult_int = multiplicity.re.round() as usize;
            if (multiplicity.re - mult_int as f64).abs() > 1e-6 || multiplicity.im.abs() > 1e-6 {
                return Err(Error::MathError(
                    "Character decomposition yielded non-integer multiplicity".to_string()
                ));
            }
            
            if mult_int > 0 {
                decomposition.push((i, mult_int));
            }
        }
        
        Ok(decomposition)
    }

    /// Check if character is irreducible
    pub fn check_irreducibility(&self, table: &CharacterTable) -> bool {
        let norm_squared = self.inner_product(self, table);
        (norm_squared.re - 1.0).abs() < 1e-6 && norm_squared.im.abs() < 1e-6
    }

    /// Compute Schur indicator
    pub fn compute_schur_indicator(&self, table: &CharacterTable) -> i8 {
        // Frobenius-Schur indicator: 1/|G| * sum over g of chi(g^2)
        let mut indicator = Complex64::new(0.0, 0.0);
        
        for class in &table.conjugacy_classes {
            // For simplicity, approximate chi(g^2)
            let chi_g = self.evaluate(class);
            let chi_g2 = if class.order % 2 == 0 {
                chi_g  // Simplified
            } else {
                chi_g * chi_g
            };
            
            indicator += chi_g2 * class.size as f64;
        }
        
        indicator /= table.group_dimension as f64;
        
        // Should be real and one of {-1, 0, 1}
        if indicator.im.abs() > 1e-6 {
            return 0;
        }
        
        if indicator.re > 0.5 {
            1
        } else if indicator.re < -0.5 {
            -1
        } else {
            0
        }
    }

    /// Get dimension
    pub fn dimension(&self) -> usize {
        self.dimension
    }

    /// Check if character is real
    pub fn is_real(&self) -> bool {
        self.values.values()
            .all(|&v| v.im.abs() < 1e-10)
    }
}

impl CharacterTable {
    /// Create a new character table
    pub fn new(group_dimension: usize, rank: usize) -> Result<Self> {
        let conjugacy_classes = Self::generate_conjugacy_classes(rank)?;
        let num_classes = conjugacy_classes.len();
        
        // Number of irreducible representations equals number of conjugacy classes
        let irreducible_characters = Vec::with_capacity(num_classes);
        let inner_product_matrix = Array2::zeros((num_classes, num_classes));
        
        Ok(Self {
            group_dimension,
            rank,
            conjugacy_classes,
            irreducible_characters,
            inner_product_matrix,
        })
    }

    /// Generate conjugacy classes (simplified for symmetric group)
    fn generate_conjugacy_classes(rank: usize) -> Result<Vec<ConjugacyClass>> {
        let mut classes = Vec::new();
        
        // Generate partitions of rank (cycle types)
        let partitions = Self::generate_partitions(rank);
        
        for partition in partitions {
            let size = Self::conjugacy_class_size(rank, &partition);
            let order = partition.iter().copied().reduce(lcm).unwrap_or(1);
            
            classes.push(ConjugacyClass {
                cycle_type: partition,
                size,
                order,
            });
        }
        
        Ok(classes)
    }

    /// Generate all partitions of n
    fn generate_partitions(n: usize) -> Vec<Vec<usize>> {
        if n == 0 {
            return vec![vec![]];
        }
        
        let mut partitions = Vec::new();
        
        for first in (1..=n).rev() {
            let sub_partitions = Self::generate_partitions(n - first);
            for mut sub in sub_partitions {
                if sub.is_empty() || sub[0] <= first {
                    sub.insert(0, first);
                    partitions.push(sub);
                }
            }
        }
        
        partitions
    }

    /// Compute size of conjugacy class from cycle type
    fn conjugacy_class_size(n: usize, cycle_type: &[usize]) -> usize {
        let mut size = factorial(n);
        let mut cycle_counts = HashMap::new();
        
        for &cycle_len in cycle_type {
            *cycle_counts.entry(cycle_len).or_insert(0) += 1;
        }
        
        for (&cycle_len, &count) in &cycle_counts {
            size /= cycle_len.pow(count as u32) * factorial(count);
        }
        
        size
    }

    /// Compute character from representation
    pub fn compute_character(&self, representation: &Array2<Complex64>) -> Result<Character> {
        let mut character = Character::from_representation(representation)?;
        
        // Compute character values on all conjugacy classes
        for class in &self.conjugacy_classes {
            // This would require actual group element representatives
            // For now, using simplified computation
            let value = self.compute_character_value(representation, class)?;
            character.values.insert(class.clone(), value);
        }
        
        character.is_irreducible = character.check_irreducibility(self);
        character.schur_indicator = character.compute_schur_indicator(self);
        
        Ok(character)
    }

    /// Compute character value on conjugacy class
    fn compute_character_value(
        &self,
        representation: &Array2<Complex64>,
        class: &ConjugacyClass,
    ) -> Result<Complex64> {
        // Simplified: would need actual group element
        // For permutation representations, can compute from cycle type
        let dim = representation.nrows();
        let mut trace = Complex64::new(0.0, 0.0);
        
        // Count fixed points for permutation character
        for &cycle_len in &class.cycle_type {
            if cycle_len == 1 {
                trace += Complex64::new(1.0, 0.0);
            }
        }
        
        Ok(trace)
    }

    /// Add irreducible character
    pub fn add_irreducible(&mut self, character: Character) -> Result<()> {
        if !character.is_irreducible {
            return Err(Error::InvalidParameter(
                "Character must be irreducible".to_string()
            ));
        }
        
        self.irreducible_characters.push(character);
        self.update_inner_products()?;
        
        Ok(())
    }

    /// Update inner product matrix
    fn update_inner_products(&mut self) -> Result<()> {
        let n = self.irreducible_characters.len();
        self.inner_product_matrix = Array2::zeros((n, n));
        
        for i in 0..n {
            for j in 0..n {
                let inner = self.irreducible_characters[i]
                    .inner_product(&self.irreducible_characters[j], self);
                self.inner_product_matrix[[i, j]] = inner;
            }
        }
        
        Ok(())
    }

    /// Check orthogonality relations
    pub fn verify_orthogonality(&self) -> bool {
        let n = self.irreducible_characters.len();
        
        for i in 0..n {
            for j in 0..n {
                let expected = if i == j { 
                    Complex64::new(1.0, 0.0) 
                } else { 
                    Complex64::new(0.0, 0.0) 
                };
                
                let actual = self.inner_product_matrix[[i, j]];
                if (actual - expected).norm() > 1e-6 {
                    return false;
                }
            }
        }
        
        true
    }

    /// Get conjugacy classes
    pub fn conjugacy_classes(&self) -> &[ConjugacyClass] {
        &self.conjugacy_classes
    }

    /// Get irreducible characters
    pub fn irreducibles(&self) -> &[Character] {
        &self.irreducible_characters
    }
}

impl WeylCharacter {
    /// Create Weyl character from highest weight
    pub fn new(highest_weight: Array1<i32>) -> Result<Self> {
        let dimension = Self::compute_dimension(&highest_weight)?;
        let weight_multiplicities = Self::compute_weight_multiplicities(&highest_weight)?;
        let character_polynomial = Self::compute_character_polynomial(&highest_weight)?;
        
        Ok(Self {
            highest_weight,
            dimension,
            character_polynomial,
            weight_multiplicities,
        })
    }

    /// Compute dimension using Weyl dimension formula
    fn compute_dimension(highest_weight: &Array1<i32>) -> Result<usize> {
        let n = highest_weight.len();
        let rho: Array1<i32> = Array1::from_vec((1..=n).rev().map(|i| i as i32).collect());
        
        let mut numerator = 1i64;
        let mut denominator = 1i64;
        
        for i in 0..n {
            for j in i+1..n {
                let lambda_diff = highest_weight[i] - highest_weight[j];
                let rho_diff = rho[i] - rho[j];
                
                numerator *= (lambda_diff + rho_diff) as i64;
                denominator *= rho_diff as i64;
            }
        }
        
        if denominator == 0 {
            return Err(Error::MathError("Division by zero in dimension formula".to_string()));
        }
        
        Ok((numerator / denominator) as usize)
    }

    /// Compute weight multiplicities using Freudenthal's formula
    fn compute_weight_multiplicities(highest_weight: &Array1<i32>) -> Result<HashMap<Array1<i32>, usize>> {
        let mut multiplicities = HashMap::new();
        
        // Highest weight has multiplicity 1
        multiplicities.insert(highest_weight.clone(), 1);
        
        // Simplified: would use Freudenthal recursion formula
        // For now, just include the highest weight
        
        Ok(multiplicities)
    }

    /// Compute character polynomial
    fn compute_character_polynomial(highest_weight: &Array1<i32>) -> Result<Vec<Complex64>> {
        // Simplified: actual computation uses Weyl character formula
        let dim = Self::compute_dimension(highest_weight)?;
        let mut coeffs = vec![Complex64::new(0.0, 0.0); dim];
        coeffs[0] = Complex64::new(1.0, 0.0);
        
        Ok(coeffs)
    }

    /// Evaluate character at element
    pub fn evaluate(&self, element: &Array1<Complex64>) -> Result<Complex64> {
        if element.len() != self.highest_weight.len() {
            return Err(Error::DimensionMismatch {
                expected: self.highest_weight.len(),
                actual: element.len(),
            });
        }

        // Use Weyl character formula
        let mut value = Complex64::new(0.0, 0.0);
        
        for (weight, &mult) in &self.weight_multiplicities {
            let mut term = Complex64::new(mult as f64, 0.0);
            
            for i in 0..element.len() {
                term *= element[i].powi(weight[i]);
            }
            
            value += term;
        }
        
        Ok(value)
    }

    /// Get dimension
    pub fn dimension(&self) -> usize {
        self.dimension
    }

    /// Get highest weight
    pub fn highest_weight(&self) -> &Array1<i32> {
        &self.highest_weight
    }

    /// Check if weight is dominant
    pub fn is_dominant(weight: &Array1<i32>) -> bool {
        for i in 0..weight.len()-1 {
            if weight[i] < weight[i+1] {
                return false;
            }
        }
        true
    }

    /// Apply Weyl group element to character
    pub fn weyl_transform(&self, w: &[usize]) -> Result<Self> {
        let mut transformed_weight = Array1::zeros(self.highest_weight.len());
        
        for (i, &j) in w.iter().enumerate() {
            if j >= self.highest_weight.len() {
                return Err(Error::InvalidParameter(
                    "Weyl group element out of bounds".to_string()
                ));
            }
            transformed_weight[i] = self.highest_weight[j];
        }
        
        // Apply sign from Weyl group element
        let sign = Self::weyl_sign(w);
        if sign < 0 {
            transformed_weight = -transformed_weight;
        }
        
        Self::new(transformed_weight)
    }

    /// Compute sign of Weyl group element
    fn weyl_sign(w: &[usize]) -> i32 {
        let mut inversions = 0;
        
        for i in 0..w.len() {
            for j in i+1..w.len() {
                if w[i] > w[j] {
                    inversions += 1;
                }
            }
        }
        
        if inversions % 2 == 0 { 1 } else { -1 }
    }
}

// Helper functions
fn factorial(n: usize) -> usize {
    if n <= 1 { 1 } else { n * factorial(n - 1) }
}

fn gcd(a: usize, b: usize) -> usize {
    if b == 0 { a } else { gcd(b, a % b) }
}

fn lcm(a: usize, b: usize) -> usize {
    a * b / gcd(a, b)
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_character_creation() {
        let char = Character::new(3);
        assert_eq!(char.dimension(), 3);
        assert!(!char.is_irreducible);
    }

    #[test]
    fn test_trivial_character() {
        let char = Character::trivial(6);
        assert_eq!(char.dimension(), 1);
        assert!(char.is_irreducible);
        assert_eq!(char.schur_indicator, 1);
    }

    #[test]
    fn test_conjugacy_classes() {
        let table = CharacterTable::new(6, 3).unwrap();
        assert!(!table.conjugacy_classes.is_empty());
        
        // Check identity class exists
        let has_identity = table.conjugacy_classes.iter()
            .any(|c| c.order == 1 && c.size == 1);
        assert!(has_identity);
    }

    #[test]
    fn test_weyl_character_dimension() {
        // Standard representation of SL(3)
        let highest_weight = Array1::from_vec(vec![1, 0, -1]);
        let weyl_char = WeylCharacter::new(highest_weight).unwrap();
        assert_eq!(weyl_char.dimension(), 3);
    }

    #[test]
    fn test_character_from_representation() {
        let rep = Array2::eye(3);
        let char = Character::from_representation(&rep).unwrap();
        assert_eq!(char.dimension(), 3);
        
        // Character at identity should be trace = 3
        let id_class = ConjugacyClass {
            cycle_type: vec![1, 1, 1],
            size: 1,
            order: 1,
        };
        assert_relative_eq!(char.evaluate(&id_class).re, 3.0, epsilon = 1e-10);
    }
}