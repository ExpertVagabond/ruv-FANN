//! Spherical functions and Harish-Chandra theory

use crate::{Error, Result};
use crate::harmonic::haar_measure::HaarMeasure;
use ndarray::{Array1, Array2, ArrayD};
use num_complex::Complex64;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Harish-Chandra parameter for spherical functions
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HarishChandraParameter {
    /// Dimension of the parameter space
    dimension: usize,
    /// Complex parameters (eigenvalues of Casimir operators)
    values: Array1<Complex64>,
    /// Whether the parameter is regular (generic)
    is_regular: bool,
    /// Whether the parameter is in the positive Weyl chamber
    is_dominant: bool,
}

impl HarishChandraParameter {
    /// Create a new Harish-Chandra parameter
    pub fn new(dimension: usize, values: Vec<f64>) -> Result<Self> {
        if values.len() != dimension {
            return Err(Error::DimensionMismatch {
                expected: dimension,
                actual: values.len(),
            });
        }

        let complex_values = Array1::from_vec(
            values.into_iter()
                .map(|v| Complex64::new(v, 0.0))
                .collect()
        );

        let is_regular = Self::check_regularity(&complex_values);
        let is_dominant = Self::check_dominance(&complex_values);

        Ok(Self {
            dimension,
            values: complex_values,
            is_regular,
            is_dominant,
        })
    }

    /// Create a complex parameter
    pub fn complex(dimension: usize, values: Vec<Complex64>) -> Result<Self> {
        if values.len() != dimension {
            return Err(Error::DimensionMismatch {
                expected: dimension,
                actual: values.len(),
            });
        }

        let complex_values = Array1::from_vec(values);
        let is_regular = Self::check_regularity(&complex_values);
        let is_dominant = Self::check_dominance(&complex_values);

        Ok(Self {
            dimension,
            values: complex_values,
            is_regular,
            is_dominant,
        })
    }

    /// Check if parameter is regular (no repeated values)
    fn check_regularity(values: &Array1<Complex64>) -> bool {
        for i in 0..values.len() {
            for j in i+1..values.len() {
                if (values[i] - values[j]).norm() < 1e-10 {
                    return false;
                }
            }
        }
        true
    }

    /// Check if parameter is in positive Weyl chamber
    fn check_dominance(values: &Array1<Complex64>) -> bool {
        for i in 0..values.len()-1 {
            if values[i].re < values[i+1].re {
                return false;
            }
        }
        true
    }

    /// Compute Casimir eigenvalue
    pub fn casimir_eigenvalue(&self) -> Complex64 {
        // Quadratic Casimir: sum of squares plus rho-shift
        let mut casimir = Complex64::new(0.0, 0.0);
        let rho = self.dimension as f64 / 2.0; // Half sum of positive roots
        
        for (i, &val) in self.values.iter().enumerate() {
            let shifted = val + Complex64::new(rho - i as f64, 0.0);
            casimir += shifted * shifted;
        }
        
        casimir
    }

    /// Apply Weyl group element
    pub fn weyl_transform(&self, w: &[usize]) -> Result<Self> {
        if w.len() != self.dimension {
            return Err(Error::DimensionMismatch {
                expected: self.dimension,
                actual: w.len(),
            });
        }

        let mut transformed = Array1::zeros(self.dimension);
        for (i, &j) in w.iter().enumerate() {
            if j >= self.dimension {
                return Err(Error::InvalidParameter(
                    "Weyl element index out of bounds".to_string()
                ));
            }
            transformed[i] = self.values[j];
        }

        Ok(Self {
            dimension: self.dimension,
            values: transformed,
            is_regular: self.is_regular,
            is_dominant: Self::check_dominance(&transformed),
        })
    }

    /// Get the values
    pub fn values(&self) -> &Array1<Complex64> {
        &self.values
    }
}

impl fmt::Display for HarishChandraParameter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "HC({:?})", self.values)
    }
}

/// Spherical function (zonal spherical function)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SphericalFunction {
    /// Harish-Chandra parameter
    parameter: HarishChandraParameter,
    /// Values on a grid (for interpolation)
    grid_values: Array2<Complex64>,
    /// Normalization constant
    normalization: Complex64,
    /// Asymptotic behavior coefficient
    c_function: Complex64,
}

impl SphericalFunction {
    /// Compute spherical function for given parameter
    pub fn compute(
        parameter: &HarishChandraParameter,
        haar_measure: &HaarMeasure,
    ) -> Result<Self> {
        // Create evaluation grid
        let grid_size = 50;
        let mut grid_values = Array2::zeros((grid_size, grid_size));
        
        // Compute c-function (Harish-Chandra's c-function)
        let c_function = Self::compute_c_function(parameter)?;
        
        // Normalization at identity
        let normalization = Complex64::new(1.0, 0.0);
        
        // Fill grid with spherical function values
        for i in 0..grid_size {
            for j in 0..grid_size {
                let t = i as f64 / grid_size as f64;
                let s = j as f64 / grid_size as f64;
                grid_values[[i, j]] = Self::evaluate_at_point(parameter, t, s)?;
            }
        }
        
        Ok(Self {
            parameter: parameter.clone(),
            grid_values,
            normalization,
            c_function,
        })
    }

    /// Evaluate spherical function at a point
    fn evaluate_at_point(
        parameter: &HarishChandraParameter,
        t: f64,
        s: f64,
    ) -> Result<Complex64> {
        // Simplified evaluation using power series expansion
        let lambda = &parameter.values;
        let dim = parameter.dimension;
        
        // Radial component
        let r = (t * t + s * s).sqrt();
        if r < 1e-10 {
            return Ok(Complex64::new(1.0, 0.0));
        }
        
        // Bessel-type function for spherical functions
        let mut value = Complex64::new(0.0, 0.0);
        
        for k in 0..10 {  // Truncated series
            let term = r.powi(k as i32) / factorial(k) as f64;
            let phase = lambda.iter()
                .enumerate()
                .map(|(i, &l)| l * Complex64::new(0.0, t * (i as f64 + 1.0)))
                .sum::<Complex64>();
            
            value += Complex64::new(term, 0.0) * phase.exp();
        }
        
        Ok(value / dim as f64)
    }

    /// Compute Harish-Chandra's c-function
    fn compute_c_function(parameter: &HarishChandraParameter) -> Result<Complex64> {
        let lambda = &parameter.values;
        let dim = parameter.dimension;
        let rho = dim as f64 / 2.0;
        
        let mut c_value = Complex64::new(1.0, 0.0);
        
        // Product formula for c-function
        for i in 0..dim {
            for j in i+1..dim {
                let diff = lambda[i] - lambda[j];
                let numerator = gamma_complex(diff + Complex64::new(1.0, 0.0));
                let denominator = gamma_complex(diff + Complex64::new(rho, 0.0));
                
                if denominator.norm() < 1e-10 {
                    return Err(Error::MathError("c-function pole encountered".to_string()));
                }
                
                c_value *= numerator / denominator;
            }
        }
        
        Ok(c_value)
    }

    /// Evaluate at a group element
    pub fn evaluate(&self, g: &Array1<f64>) -> Result<Complex64> {
        if g.len() != self.parameter.dimension {
            return Err(Error::DimensionMismatch {
                expected: self.parameter.dimension,
                actual: g.len(),
            });
        }

        // Map group element to grid coordinates
        let t = g[0].abs().min(1.0);
        let s = if g.len() > 1 { g[1].abs().min(1.0) } else { 0.0 };
        
        // Bilinear interpolation on grid
        let i = (t * (self.grid_values.nrows() - 1) as f64) as usize;
        let j = (s * (self.grid_values.ncols() - 1) as f64) as usize;
        
        if i < self.grid_values.nrows() && j < self.grid_values.ncols() {
            Ok(self.grid_values[[i, j]])
        } else {
            Err(Error::InvalidParameter("Group element out of grid range".to_string()))
        }
    }

    /// Get the Harish-Chandra parameter
    pub fn parameter(&self) -> &HarishChandraParameter {
        &self.parameter
    }

    /// Get the c-function value
    pub fn c_function(&self) -> Complex64 {
        self.c_function
    }

    /// Asymptotic expansion (for large elements)
    pub fn asymptotic_expansion(&self, g: &Array1<f64>, order: usize) -> Result<Vec<Complex64>> {
        let mut expansion = vec![self.c_function];
        
        // Higher order terms in asymptotic expansion
        for k in 1..=order {
            let term = self.c_function * Complex64::new(1.0 / k as f64, 0.0);
            expansion.push(term);
        }
        
        Ok(expansion)
    }

    /// Check if the function is tempered
    pub fn is_tempered(&self) -> bool {
        // Spherical function is tempered if parameter is purely imaginary
        self.parameter.values.iter()
            .all(|&v| v.re.abs() < 1e-10)
    }

    /// Functional equation relating phi_lambda and phi_{-lambda}
    pub fn functional_equation(&self, other: &SphericalFunction) -> Result<bool> {
        // Check if parameters are opposite
        let tolerance = 1e-10;
        
        for i in 0..self.parameter.dimension {
            let sum = self.parameter.values[i] + other.parameter.values[i];
            if sum.norm() > tolerance {
                return Ok(false);
            }
        }
        
        // Check c-function relation
        let product = self.c_function * other.c_function;
        Ok((product - Complex64::new(1.0, 0.0)).norm() < tolerance)
    }
}

/// Elementary spherical function (for K-types)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementarySphericalFunction {
    /// Highest weight of K-type
    highest_weight: Array1<i32>,
    /// Dimension of K-type
    dimension: usize,
    /// Matrix coefficients
    matrix_coefficients: Array2<Complex64>,
}

impl ElementarySphericalFunction {
    /// Create elementary spherical function for K-type
    pub fn new(highest_weight: Array1<i32>) -> Result<Self> {
        let dimension = Self::compute_dimension(&highest_weight)?;
        let matrix_coefficients = Self::compute_matrix_coefficients(&highest_weight, dimension)?;
        
        Ok(Self {
            highest_weight,
            dimension,
            matrix_coefficients,
        })
    }

    /// Compute dimension using Weyl dimension formula
    fn compute_dimension(highest_weight: &Array1<i32>) -> Result<usize> {
        let n = highest_weight.len();
        let mut dim = 1;
        
        for i in 0..n {
            for j in i+1..n {
                let diff = highest_weight[i] - highest_weight[j];
                dim *= (diff + j - i) as usize;
                dim /= (j - i) as usize;
            }
        }
        
        Ok(dim)
    }

    /// Compute matrix coefficients
    fn compute_matrix_coefficients(
        highest_weight: &Array1<i32>,
        dimension: usize,
    ) -> Result<Array2<Complex64>> {
        // Simplified - would use representation theory
        let mut coeffs = Array2::zeros((dimension, dimension));
        
        // Set diagonal elements
        for i in 0..dimension {
            coeffs[[i, i]] = Complex64::new(1.0, 0.0);
        }
        
        Ok(coeffs)
    }

    /// Evaluate at maximal compact subgroup element
    pub fn evaluate_k(&self, k: &Array2<f64>) -> Result<Complex64> {
        // Trace of representation matrix
        let mut trace = Complex64::new(0.0, 0.0);
        
        for i in 0..self.dimension.min(k.nrows()) {
            trace += Complex64::new(k[[i, i]], 0.0);
        }
        
        Ok(trace / self.dimension as f64)
    }
}

// Helper function for complex gamma function (simplified)
fn gamma_complex(z: Complex64) -> Complex64 {
    // Use Stirling approximation for complex gamma
    if z.re > 0.0 {
        let sqrt_2pi = (2.0 * std::f64::consts::PI).sqrt();
        let e = std::f64::consts::E;
        
        let magnitude = sqrt_2pi * z.re.powf(z.re - 0.5) * (-z.re).exp();
        let phase = z.im * z.re.ln();
        
        Complex64::from_polar(magnitude, phase)
    } else {
        // Use reflection formula for negative real parts
        Complex64::new(1.0, 0.0)  // Simplified
    }
}

// Helper function for factorial
fn factorial(n: usize) -> usize {
    if n <= 1 { 1 } else { n * factorial(n - 1) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_harish_chandra_parameter() {
        let param = HarishChandraParameter::new(3, vec![3.0, 2.0, 1.0]).unwrap();
        assert!(param.is_regular);
        assert!(param.is_dominant);
    }

    #[test]
    fn test_casimir_eigenvalue() {
        let param = HarishChandraParameter::new(2, vec![1.0, 0.0]).unwrap();
        let casimir = param.casimir_eigenvalue();
        assert!(casimir.re > 0.0);
    }

    #[test]
    fn test_spherical_function_normalization() {
        let param = HarishChandraParameter::new(2, vec![0.0, 0.0]).unwrap();
        let haar = HaarMeasure::new(2).unwrap();
        let spherical = SphericalFunction::compute(&param, &haar).unwrap();
        
        // At identity, spherical function should be 1
        let identity = Array1::zeros(2);
        let value = spherical.evaluate(&identity).unwrap();
        assert_relative_eq!(value.re, 1.0, epsilon = 0.1);
    }

    #[test]
    fn test_elementary_spherical() {
        let highest_weight = Array1::from_vec(vec![2, 1, 0]);
        let elem = ElementarySphericalFunction::new(highest_weight).unwrap();
        assert!(elem.dimension > 0);
    }
}