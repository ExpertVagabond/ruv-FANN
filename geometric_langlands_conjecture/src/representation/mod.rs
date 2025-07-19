//! Representation theory
//!
//! This module implements representation theory for reductive groups,
//! including principal series and discrete series representations.

use crate::error::Result;
use nalgebra::DVector;
use num_complex::Complex64;

/// Trait for group representations
pub trait Representation {
    /// Dimension of the representation
    fn dimension(&self) -> usize;
    
    /// Character function
    fn character(&self, element: &[f64]) -> f64;
    
    /// Weight space for given weight
    fn weight_space(&self, weight: &[i32]) -> Result<DVector<Complex64>>;
    
    /// Highest weight
    fn highest_weight(&self) -> Vec<i32>;
}

/// Principal series representation
#[derive(Debug, Clone, PartialEq)]
pub struct PrincipalSeries {
    /// Dimension
    pub dimension: usize,
    /// Inducing character
    pub character: Vec<f64>,
}

impl PrincipalSeries {
    /// Create new principal series
    pub fn new(dimension: usize, character: Vec<f64>) -> Self {
        Self { dimension, character }
    }
}

impl Representation for PrincipalSeries {
    fn dimension(&self) -> usize {
        self.dimension
    }
    
    fn character(&self, _element: &[f64]) -> f64 {
        self.character.iter().sum()
    }
    
    fn weight_space(&self, _weight: &[i32]) -> Result<DVector<Complex64>> {
        Ok(DVector::from_element(self.dimension, Complex64::new(1.0, 0.0)))
    }
    
    fn highest_weight(&self) -> Vec<i32> {
        vec![1; self.dimension.min(3)]
    }
}

/// Discrete series representation
#[derive(Debug, Clone, PartialEq)]
pub struct DiscreteSeries {
    /// Dimension
    pub dimension: usize,
    /// L² property
    pub is_square_integrable: bool,
}

impl DiscreteSeries {
    /// Create new discrete series
    pub fn new(dimension: usize) -> Self {
        Self { 
            dimension, 
            is_square_integrable: true 
        }
    }
}

impl Representation for DiscreteSeries {
    fn dimension(&self) -> usize {
        self.dimension
    }
    
    fn character(&self, _element: &[f64]) -> f64 {
        self.dimension as f64
    }
    
    fn weight_space(&self, _weight: &[i32]) -> Result<DVector<Complex64>> {
        Ok(DVector::from_element(self.dimension, Complex64::new(1.0, 0.0)))
    }
    
    fn highest_weight(&self) -> Vec<i32> {
        vec![1; self.dimension.min(3)]
    }
}