//! Haar measure implementation for reductive groups

use crate::{Error, Result};
use ndarray::{Array1, Array2, ArrayD};
use num_complex::Complex64;
use serde::{Deserialize, Serialize};
use std::f64::consts::PI;

/// Haar measure on a reductive group
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HaarMeasure {
    /// Dimension of the group
    dimension: usize,
    /// Normalization constant
    normalization: f64,
    /// Volume of maximal compact subgroup
    compact_volume: f64,
    /// Modular function values (for non-unimodular groups)
    modular_function: Option<Array1<f64>>,
}

impl HaarMeasure {
    /// Create a new Haar measure
    pub fn new(dimension: usize) -> Result<Self> {
        if dimension == 0 {
            return Err(Error::InvalidParameter(
                "Dimension must be positive".to_string()
            ));
        }

        // Compute normalization based on dimension
        let normalization = (2.0 * PI).powf(dimension as f64 / 2.0);
        
        // Volume of maximal compact subgroup (e.g., SO(n) in SL(n))
        let compact_volume = Self::compute_compact_volume(dimension);

        Ok(Self {
            dimension,
            normalization,
            compact_volume,
            modular_function: None,
        })
    }

    /// Create Haar measure for a non-unimodular group
    pub fn non_unimodular(dimension: usize, modular_function: Array1<f64>) -> Result<Self> {
        let mut measure = Self::new(dimension)?;
        measure.modular_function = Some(modular_function);
        Ok(measure)
    }

    /// Compute volume of maximal compact subgroup
    fn compute_compact_volume(dimension: usize) -> f64 {
        // Volume formula for SO(n) or SU(n)
        let mut volume = 1.0;
        for k in 1..=dimension {
            volume *= (2.0 * PI).powf(k as f64) / gamma(k as f64 / 2.0 + 1.0);
        }
        volume
    }

    /// Integrate a function over the group
    pub fn integrate<F>(&self, f: F, num_samples: usize) -> Result<Complex64>
    where
        F: Fn(&Array1<f64>) -> Complex64,
    {
        // Monte Carlo integration over the group manifold
        let mut sum = Complex64::new(0.0, 0.0);
        
        for _ in 0..num_samples {
            let point = self.sample_point()?;
            let value = f(&point);
            
            // Apply modular function if group is non-unimodular
            let weight = if let Some(ref modular) = self.modular_function {
                modular[0]  // Simplified - should evaluate at point
            } else {
                1.0
            };
            
            sum += value * weight;
        }
        
        Ok(sum * self.normalization / num_samples as f64)
    }

    /// Sample a random point from the group with respect to Haar measure
    pub fn sample_point(&self) -> Result<Array1<f64>> {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        
        // Sample from appropriate distribution based on group type
        let mut point = Array1::zeros(self.dimension);
        
        // For simplicity, using Gaussian distribution
        // In practice, would use group-specific sampling
        for i in 0..self.dimension {
            point[i] = rng.gen_range(-1.0..1.0);
        }
        
        Ok(point)
    }

    /// Compute the volume of a subset
    pub fn volume<F>(&self, characteristic_fn: F, num_samples: usize) -> Result<f64>
    where
        F: Fn(&Array1<f64>) -> bool,
    {
        let indicator = |x: &Array1<f64>| {
            if characteristic_fn(x) {
                Complex64::new(1.0, 0.0)
            } else {
                Complex64::new(0.0, 0.0)
            }
        };
        
        let integral = self.integrate(indicator, num_samples)?;
        Ok(integral.re)
    }

    /// Get dimension
    pub fn dimension(&self) -> usize {
        self.dimension
    }

    /// Get normalization constant
    pub fn normalization(&self) -> f64 {
        self.normalization
    }

    /// Check if the group is unimodular
    pub fn is_unimodular(&self) -> bool {
        self.modular_function.is_none()
    }

    /// Apply left translation
    pub fn left_translate(&self, g: &Array1<f64>, f: &Array1<f64>) -> Array1<f64> {
        // Simplified group multiplication
        g + f
    }

    /// Apply right translation
    pub fn right_translate(&self, f: &Array1<f64>, g: &Array1<f64>) -> Array1<f64> {
        // Simplified group multiplication
        f + g
    }
}

/// Haar integral operator
#[derive(Debug, Clone)]
pub struct HaarIntegral {
    measure: HaarMeasure,
    convergence_threshold: f64,
    max_iterations: usize,
}

impl HaarIntegral {
    /// Create a new Haar integral operator
    pub fn new(measure: HaarMeasure) -> Self {
        Self {
            measure,
            convergence_threshold: 1e-10,
            max_iterations: 1000,
        }
    }

    /// Set convergence threshold
    pub fn with_threshold(mut self, threshold: f64) -> Self {
        self.convergence_threshold = threshold;
        self
    }

    /// Compute integral with adaptive sampling
    pub fn integrate_adaptive<F>(&self, f: F) -> Result<Complex64>
    where
        F: Fn(&Array1<f64>) -> Complex64,
    {
        let mut num_samples = 1000;
        let mut prev_value = Complex64::new(0.0, 0.0);
        
        for iteration in 0..self.max_iterations {
            let current_value = self.measure.integrate(&f, num_samples)?;
            
            if (current_value - prev_value).norm() < self.convergence_threshold {
                return Ok(current_value);
            }
            
            prev_value = current_value;
            num_samples *= 2;
            
            if iteration == self.max_iterations - 1 {
                return Err(Error::ConvergenceFailure { 
                    iterations: self.max_iterations 
                });
            }
        }
        
        Ok(prev_value)
    }

    /// Integrate a matrix-valued function
    pub fn integrate_matrix<F>(&self, f: F, num_samples: usize) -> Result<Array2<Complex64>>
    where
        F: Fn(&Array1<f64>) -> Array2<Complex64>,
    {
        let test_point = self.measure.sample_point()?;
        let test_value = f(&test_point);
        let shape = test_value.raw_dim();
        
        let mut result = Array2::zeros(shape);
        
        for _ in 0..num_samples {
            let point = self.measure.sample_point()?;
            let value = f(&point);
            result = result + value;
        }
        
        Ok(result * (self.measure.normalization / num_samples as f64))
    }
}

// Helper function for gamma function
fn gamma(x: f64) -> f64 {
    // Simplified gamma function approximation
    if x == 0.5 {
        PI.sqrt()
    } else if x == 1.0 {
        1.0
    } else if x == 1.5 {
        PI.sqrt() / 2.0
    } else if x == 2.0 {
        1.0
    } else if x == 2.5 {
        3.0 * PI.sqrt() / 4.0
    } else {
        // Use Stirling's approximation for general case
        (2.0 * PI * x).sqrt() * (x / std::f64::consts::E).powf(x)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_haar_measure_creation() {
        let measure = HaarMeasure::new(3).unwrap();
        assert_eq!(measure.dimension(), 3);
        assert!(measure.is_unimodular());
    }

    #[test]
    fn test_constant_function_integral() {
        let measure = HaarMeasure::new(2).unwrap();
        let constant = |_: &Array1<f64>| Complex64::new(1.0, 0.0);
        
        // Integral of constant function should give volume
        let result = measure.integrate(constant, 10000).unwrap();
        assert!(result.re > 0.0);
        assert_relative_eq!(result.im, 0.0, epsilon = 1e-10);
    }

    #[test]
    fn test_haar_integral_convergence() {
        let measure = HaarMeasure::new(2).unwrap();
        let integral = HaarIntegral::new(measure).with_threshold(1e-6);
        
        let f = |x: &Array1<f64>| Complex64::new(x[0].sin(), x[1].cos());
        let result = integral.integrate_adaptive(f);
        assert!(result.is_ok());
    }

    #[test]
    fn test_non_unimodular() {
        let modular = Array1::from_vec(vec![1.5, 2.0]);
        let measure = HaarMeasure::non_unimodular(2, modular).unwrap();
        assert!(!measure.is_unimodular());
    }
}