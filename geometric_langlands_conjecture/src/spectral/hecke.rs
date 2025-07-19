//! Hecke operators spectral theory
//!
//! This module implements the spectral theory of Hecke operators,
//! including their eigenvalues, eigenfunctions, and L-functions.

use nalgebra::{DMatrix, DVector, Complex};
use num_complex::Complex64;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use crate::Error;

/// Hecke operator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeckeOperator {
    /// Index of the operator (usually a prime)
    pub index: usize,
    /// Level of the operator
    pub level: usize,
    /// Weight
    pub weight: i32,
    /// Matrix representation
    pub matrix: Option<DMatrix<Complex64>>,
    /// Eigenvalues
    pub eigenvalues: HashMap<String, Complex64>,
}

/// Hecke algebra
#[derive(Debug, Clone)]
pub struct HeckeAlgebra {
    /// Level
    pub level: usize,
    /// Weight
    pub weight: i32,
    /// Operators indexed by prime
    pub operators: HashMap<usize, HeckeOperator>,
    /// Commutativity relations
    pub relations: Vec<CommutationRelation>,
}

/// Commutation relation in Hecke algebra
#[derive(Debug, Clone)]
pub struct CommutationRelation {
    /// First operator index
    pub op1: usize,
    /// Second operator index
    pub op2: usize,
    /// Whether they commute
    pub commute: bool,
}

/// Hecke eigenform
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeckeEigenform {
    /// Level
    pub level: usize,
    /// Weight
    pub weight: i32,
    /// Character
    pub character: DirichletCharacter,
    /// Eigenvalues for each Hecke operator
    pub hecke_eigenvalues: HashMap<usize, Complex64>,
    /// Fourier coefficients
    pub fourier_coefficients: Vec<Complex64>,
    /// Associated L-function
    pub l_function: HeckeLFunction,
}

/// Dirichlet character
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirichletCharacter {
    /// Modulus
    pub modulus: usize,
    /// Values on generators
    pub values: HashMap<usize, Complex64>,
    /// Conductor
    pub conductor: usize,
}

/// L-function associated to Hecke eigenform
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeckeLFunction {
    /// Degree
    pub degree: usize,
    /// Conductor
    pub conductor: usize,
    /// Gamma factors at infinity
    pub gamma_factors: Vec<GammaFactor>,
    /// Euler factors
    pub euler_factors: HashMap<usize, EulerFactor>,
    /// Functional equation
    pub functional_equation: HeckeFunctionalEquation,
}

/// Gamma factor for L-function
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GammaFactor {
    /// Shift parameter
    pub shift: Complex64,
    /// Type
    pub factor_type: String,
}

/// Euler factor at a prime
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EulerFactor {
    /// Prime
    pub prime: usize,
    /// Polynomial coefficients
    pub polynomial: Vec<Complex64>,
}

/// Functional equation for Hecke L-function
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeckeFunctionalEquation {
    /// Sign (root number)
    pub sign: Complex64,
    /// Center of symmetry
    pub center: f64,
    /// Conductor
    pub conductor: usize,
}

impl HeckeOperator {
    /// Create a new Hecke operator T_n
    pub fn new(index: usize, level: usize, weight: i32) -> Self {
        Self {
            index,
            level,
            weight,
            matrix: None,
            eigenvalues: HashMap::new(),
        }
    }

    /// Compute matrix representation on space of modular forms
    pub fn compute_matrix(&mut self, dimension: usize) -> Result<(), Error> {
        // For T_p on S_k(Γ_0(N)), use double coset decomposition
        let mut matrix = DMatrix::zeros(dimension, dimension);
        
        // Simplified: for level 1 and small dimension
        if self.level == 1 && dimension > 0 {
            for i in 0..dimension {
                for j in 0..dimension {
                    matrix[(i, j)] = self.compute_matrix_entry(i, j)?;
                }
            }
        }
        
        self.matrix = Some(matrix);
        Ok(())
    }

    /// Compute a single matrix entry
    fn compute_matrix_entry(&self, i: usize, j: usize) -> Result<Complex64, Error> {
        // Simplified computation using Hecke relations
        let p = self.index as f64;
        let k = self.weight as f64;
        
        if i == j {
            // Diagonal entries involve traces
            if i == 0 {
                // T_p on constant term
                Ok(Complex64::new(p.powf(k - 1.0) + 1.0, 0.0))
            } else {
                // General diagonal entry
                Ok(Complex64::new(1.0, 0.0))
            }
        } else if i == j + self.index || j == i + self.index {
            // Off-diagonal entries from Hecke relations
            Ok(Complex64::new(p.powf((k - 1.0) / 2.0), 0.0))
        } else {
            Ok(Complex64::new(0.0, 0.0))
        }
    }

    /// Apply operator to a modular form (given by Fourier coefficients)
    pub fn apply(&self, coefficients: &[Complex64]) -> Result<Vec<Complex64>, Error> {
        let mut result = vec![Complex64::new(0.0, 0.0); coefficients.len()];
        let p = self.index;
        let k = self.weight;
        
        for n in 0..coefficients.len() {
            // T_p(a_n) = a_{np} + p^{k-1} * a_{n/p} if p|n
            let np = n * p;
            if np < coefficients.len() {
                result[n] += coefficients[np];
            }
            
            if n > 0 && n % p == 0 {
                let n_div_p = n / p;
                result[n] += Complex64::new((p as f64).powi(k - 1), 0.0) * coefficients[n_div_p];
            }
        }
        
        Ok(result)
    }

    /// Compute eigenvalues on a basis
    pub fn compute_eigenvalues(&mut self, basis: &[Vec<Complex64>]) -> Result<(), Error> {
        self.eigenvalues.clear();
        
        for (i, form) in basis.iter().enumerate() {
            let applied = self.apply(form)?;
            
            // Check if it's an eigenform
            if let Some(eigenvalue) = self.is_eigenform(&applied, form) {
                self.eigenvalues.insert(format!("form_{}", i), eigenvalue);
            }
        }
        
        Ok(())
    }

    /// Check if a form is an eigenform and return eigenvalue
    fn is_eigenform(&self, applied: &[Complex64], original: &[Complex64]) -> Option<Complex64> {
        if applied.len() != original.len() || applied.is_empty() {
            return None;
        }
        
        // Find first non-zero coefficient
        for i in 0..original.len() {
            if original[i].norm() > 1e-10 {
                let eigenvalue = applied[i] / original[i];
                
                // Verify it's an eigenvalue for all coefficients
                let mut is_eigen = true;
                for j in 0..original.len() {
                    if original[j].norm() > 1e-10 {
                        let ratio = applied[j] / original[j];
                        if (ratio - eigenvalue).norm() > 1e-10 {
                            is_eigen = false;
                            break;
                        }
                    }
                }
                
                if is_eigen {
                    return Some(eigenvalue);
                }
                break;
            }
        }
        
        None
    }
}

impl HeckeAlgebra {
    /// Create new Hecke algebra
    pub fn new(level: usize, weight: i32) -> Self {
        Self {
            level,
            weight,
            operators: HashMap::new(),
            relations: Vec::new(),
        }
    }

    /// Add Hecke operator T_p
    pub fn add_operator(&mut self, prime: usize) {
        let operator = HeckeOperator::new(prime, self.level, self.weight);
        self.operators.insert(prime, operator);
    }

    /// Compute commutation relations
    pub fn compute_relations(&mut self) {
        self.relations.clear();
        
        let primes: Vec<usize> = self.operators.keys().cloned().collect();
        
        for i in 0..primes.len() {
            for j in i+1..primes.len() {
                let p = primes[i];
                let q = primes[j];
                
                // T_p and T_q commute if gcd(p,q) = 1
                let commute = gcd(p, q) == 1;
                
                self.relations.push(CommutationRelation {
                    op1: p,
                    op2: q,
                    commute,
                });
            }
        }
    }

    /// Get operator T_n for any n (using multiplicativity)
    pub fn get_operator(&self, n: usize) -> Result<HeckeOperator, Error> {
        // Factor n into prime powers
        let factorization = factor(n);
        
        if factorization.len() == 1 && factorization[0].1 == 1 {
            // n is prime
            if let Some(op) = self.operators.get(&n) {
                return Ok(op.clone());
            }
        }
        
        // Use multiplicativity: T_{mn} = T_m T_n if gcd(m,n) = 1
        // For prime powers: T_{p^k} computed recursively
        
        // Placeholder for general n
        Ok(HeckeOperator::new(n, self.level, self.weight))
    }
}

impl HeckeEigenform {
    /// Create new Hecke eigenform
    pub fn new(level: usize, weight: i32) -> Self {
        Self {
            level,
            weight,
            character: DirichletCharacter::trivial(level),
            hecke_eigenvalues: HashMap::new(),
            fourier_coefficients: Vec::new(),
            l_function: HeckeLFunction::new(level),
        }
    }

    /// Set Fourier coefficient a_n
    pub fn set_coefficient(&mut self, n: usize, value: Complex64) {
        if n >= self.fourier_coefficients.len() {
            self.fourier_coefficients.resize(n + 1, Complex64::new(0.0, 0.0));
        }
        self.fourier_coefficients[n] = value;
    }

    /// Get Fourier coefficient a_n
    pub fn get_coefficient(&self, n: usize) -> Complex64 {
        if n < self.fourier_coefficients.len() {
            self.fourier_coefficients[n]
        } else {
            Complex64::new(0.0, 0.0)
        }
    }

    /// Compute Hecke eigenvalue for T_p using Fourier coefficients
    pub fn compute_hecke_eigenvalue(&mut self, p: usize) -> Result<Complex64, Error> {
        if p >= self.fourier_coefficients.len() {
            return Err(Error::Other("Not enough Fourier coefficients".to_string()));
        }
        
        // For normalized eigenform: a_p = eigenvalue of T_p
        let eigenvalue = self.fourier_coefficients[p];
        self.hecke_eigenvalues.insert(p, eigenvalue);
        
        Ok(eigenvalue)
    }

    /// Verify Hecke relations
    pub fn verify_hecke_relations(&self) -> bool {
        // Check multiplicativity: a_{mn} = a_m * a_n for gcd(m,n) = 1
        for m in 2..10 {
            for n in 2..10 {
                if gcd(m, n) == 1 && m * n < self.fourier_coefficients.len() {
                    let a_mn = self.get_coefficient(m * n);
                    let a_m = self.get_coefficient(m);
                    let a_n = self.get_coefficient(n);
                    
                    if (a_mn - a_m * a_n).norm() > 1e-10 {
                        return false;
                    }
                }
            }
        }
        
        true
    }

    /// Compute L-function values
    pub fn compute_l_value(&self, s: Complex64) -> Complex64 {
        let mut sum = Complex64::new(0.0, 0.0);
        
        // L(f, s) = Σ a_n / n^s
        for n in 1..self.fourier_coefficients.len() {
            let a_n = self.get_coefficient(n);
            let n_s = Complex64::new(n as f64, 0.0).powc(s);
            sum += a_n / n_s;
        }
        
        sum
    }
}

impl DirichletCharacter {
    /// Create trivial character mod n
    pub fn trivial(modulus: usize) -> Self {
        Self {
            modulus,
            values: HashMap::new(),
            conductor: 1,
        }
    }

    /// Evaluate character at n
    pub fn evaluate(&self, n: usize) -> Complex64 {
        if gcd(n, self.modulus) > 1 {
            Complex64::new(0.0, 0.0)
        } else if let Some(&value) = self.values.get(&(n % self.modulus)) {
            value
        } else {
            Complex64::new(1.0, 0.0) // Trivial character
        }
    }
}

impl HeckeLFunction {
    /// Create new L-function
    pub fn new(conductor: usize) -> Self {
        Self {
            degree: 2,
            conductor,
            gamma_factors: vec![
                GammaFactor {
                    shift: Complex64::new(0.0, 0.0),
                    factor_type: "Real".to_string(),
                }
            ],
            euler_factors: HashMap::new(),
            functional_equation: HeckeFunctionalEquation {
                sign: Complex64::new(1.0, 0.0),
                center: 0.5,
                conductor,
            },
        }
    }

    /// Add Euler factor at prime p
    pub fn add_euler_factor(&mut self, p: usize, ap: Complex64) {
        // For GL(2): L_p(s) = 1 / (1 - a_p p^{-s} + p^{k-1-2s})
        let euler = EulerFactor {
            prime: p,
            polynomial: vec![
                Complex64::new(1.0, 0.0),
                -ap,
                Complex64::new(p as f64, 0.0).powi(self.degree as i32 - 1),
            ],
        };
        self.euler_factors.insert(p, euler);
    }

    /// Evaluate L-function using Euler product
    pub fn evaluate(&self, s: Complex64) -> Complex64 {
        let mut product = Complex64::new(1.0, 0.0);
        
        for euler in self.euler_factors.values() {
            let factor = self.evaluate_euler_factor(euler, s);
            product *= factor;
        }
        
        product
    }

    /// Evaluate single Euler factor
    fn evaluate_euler_factor(&self, euler: &EulerFactor, s: Complex64) -> Complex64 {
        let p = Complex64::new(euler.prime as f64, 0.0);
        let p_s = p.powc(-s);
        
        // Evaluate polynomial at p^{-s}
        let mut value = Complex64::new(0.0, 0.0);
        let mut power = Complex64::new(1.0, 0.0);
        
        for coeff in &euler.polynomial {
            value += coeff * power;
            power *= p_s;
        }
        
        if value.norm() > 1e-10 {
            Complex64::new(1.0, 0.0) / value
        } else {
            Complex64::new(1.0, 0.0)
        }
    }
}

/// Helper function: compute gcd
fn gcd(a: usize, b: usize) -> usize {
    if b == 0 {
        a
    } else {
        gcd(b, a % b)
    }
}

/// Helper function: factor an integer
fn factor(n: usize) -> Vec<(usize, usize)> {
    let mut factors = Vec::new();
    let mut m = n;
    let mut p = 2;
    
    while p * p <= m {
        let mut e = 0;
        while m % p == 0 {
            m /= p;
            e += 1;
        }
        if e > 0 {
            factors.push((p, e));
        }
        p += if p == 2 { 1 } else { 2 };
    }
    
    if m > 1 {
        factors.push((m, 1));
    }
    
    factors
}

/// Petersson inner product for Hecke eigenforms
pub struct PeterssonInnerProduct {
    /// Weight
    pub weight: i32,
    /// Level
    pub level: usize,
}

impl PeterssonInnerProduct {
    /// Create new Petersson inner product
    pub fn new(weight: i32, level: usize) -> Self {
        Self { weight, level }
    }

    /// Compute inner product of two modular forms
    pub fn inner_product(&self, f: &[Complex64], g: &[Complex64]) -> Complex64 {
        // <f, g> = ∫∫_{F} f(z) conj(g(z)) y^k dxdy/y^2
        // where F is fundamental domain
        
        // Simplified: use Parseval for Fourier coefficients
        let mut sum = Complex64::new(0.0, 0.0);
        
        let max_n = f.len().min(g.len());
        for n in 1..max_n {
            sum += f[n] * g[n].conj() / Complex64::new(n as f64, 0.0).powi(self.weight - 1);
        }
        
        // Normalize by volume of fundamental domain
        let volume = if self.level == 1 {
            std::f64::consts::PI / 3.0
        } else {
            std::f64::consts::PI * self.level as f64 / 3.0
        };
        
        sum * Complex64::new(volume, 0.0)
    }

    /// Check if two forms are orthogonal
    pub fn are_orthogonal(&self, f: &[Complex64], g: &[Complex64]) -> bool {
        self.inner_product(f, g).norm() < 1e-10
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hecke_operator() {
        let mut t2 = HeckeOperator::new(2, 1, 12);
        t2.compute_matrix(5).unwrap();
        assert_eq!(t2.index, 2);
    }

    #[test]
    fn test_hecke_eigenform() {
        let mut eigenform = HeckeEigenform::new(1, 12);
        
        // Set some Fourier coefficients (Ramanujan's tau function)
        eigenform.set_coefficient(1, Complex64::new(1.0, 0.0));
        eigenform.set_coefficient(2, Complex64::new(-24.0, 0.0));
        eigenform.set_coefficient(3, Complex64::new(252.0, 0.0));
        
        let eigenvalue = eigenform.compute_hecke_eigenvalue(2).unwrap();
        assert_eq!(eigenvalue, Complex64::new(-24.0, 0.0));
    }

    #[test]
    fn test_gcd() {
        assert_eq!(gcd(12, 18), 6);
        assert_eq!(gcd(17, 19), 1);
    }
}