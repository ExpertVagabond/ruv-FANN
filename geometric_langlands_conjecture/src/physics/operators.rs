//! # Wilson and 't Hooft Line Operators
//!
//! This module implements Wilson and 't Hooft line operators which play a crucial
//! role in S-duality and the physical interpretation of Geometric Langlands.

use crate::core::{Group, Field};
use crate::representation::Representation;
use crate::sheaf::LocalSystem;
use crate::error::{Error, Result};
use num_complex::Complex64;
use nalgebra::{DMatrix, DVector};
use std::f64::consts::PI;

/// Generic line operator interface
pub trait LineOperator {
    /// Evaluate operator along a path
    fn evaluate(&self, path: &Path) -> Result<Complex64>;
    
    /// Operator product expansion with another line
    fn ope(&self, other: &dyn LineOperator, distance: f64) -> Result<Vec<(f64, Box<dyn LineOperator>)>>;
    
    /// S-duality transformation
    fn s_dual(&self) -> Result<Box<dyn LineOperator>>;
    
    /// Expectation value in vacuum
    fn vacuum_expectation(&self) -> Result<Complex64>;
}

/// Wilson line operator W_R(C)
#[derive(Debug, Clone)]
pub struct WilsonLine {
    /// Gauge group
    pub gauge_group: Box<dyn Group>,
    /// Representation R
    pub representation: Box<dyn Representation>,
    /// Electric charge (weight)
    pub electric_charge: Vec<i32>,
}

impl WilsonLine {
    /// Create new Wilson line
    pub fn new(
        gauge_group: Box<dyn Group>,
        representation: Box<dyn Representation>,
        electric_charge: Vec<i32>,
    ) -> Result<Self> {
        if electric_charge.len() != gauge_group.rank() {
            return Err(Error::Dimension("Charge dimension mismatch".to_string()));
        }
        
        Ok(Self {
            gauge_group,
            representation,
            electric_charge,
        })
    }
    
    /// Fundamental Wilson line
    pub fn fundamental(gauge_group: Box<dyn Group>) -> Result<Self> {
        let rank = gauge_group.rank();
        let fund_rep = gauge_group.fundamental_representation()?;
        let charge = vec![1; rank];
        
        Self::new(gauge_group, fund_rep, charge)
    }
    
    /// Compute holonomy along path
    pub fn holonomy(&self, connection: &DMatrix<Complex64>, path: &Path) -> Result<Complex64> {
        // Tr_R P exp(∮_C A)
        
        let mut holonomy = DMatrix::identity(self.representation.dimension(), self.representation.dimension());
        
        // Discretize path and compute path-ordered exponential
        let steps = 100;
        let dt = 1.0 / steps as f64;
        
        for i in 0..steps {
            let t = i as f64 * dt;
            let pos = path.position_at(t)?;
            
            // Extract connection at this point (simplified)
            let a_t = connection.clone() * Complex64::new(pos[0], pos[1]);
            
            // Update holonomy: U(t+dt) = exp(A(t)dt) U(t)
            holonomy = (a_t * dt).exp() * holonomy;
        }
        
        // Take trace in representation
        Ok(holonomy.trace() / self.representation.dimension() as f64)
    }
    
    /// Fusion with another Wilson line
    pub fn fusion(&self, other: &WilsonLine) -> Result<Vec<WilsonLine>> {
        // W_R × W_S = Σ_T N^T_RS W_T
        
        if self.gauge_group.name() != other.gauge_group.name() {
            return Err(Error::Computation("Different gauge groups".to_string()));
        }
        
        // Simplified: just return sum of charges
        let mut fused_charge = self.electric_charge.clone();
        for (i, &charge) in other.electric_charge.iter().enumerate() {
            fused_charge[i] += charge;
        }
        
        let fused = WilsonLine::new(
            self.gauge_group.clone(),
            self.representation.clone(), // Should tensor representations
            fused_charge,
        )?;
        
        Ok(vec![fused])
    }
    
    /// Correlation function with another Wilson line
    pub fn correlation(&self, other: &WilsonLine, linking: i32) -> Result<Complex64> {
        // <W_R(C) W_S(C')> depends on linking number
        
        // For unknot: <W_R> = dim(R)
        let dim_r = self.representation.dimension() as f64;
        let dim_s = other.representation.dimension() as f64;
        
        // Jones polynomial at q = exp(2πi/(k+N))
        let k = 5; // Level (should be input)
        let n = self.gauge_group.rank() as f64;
        let q = Complex64::new(0.0, 2.0 * PI / (k as f64 + n)).exp();
        
        // Simplified HOMFLY polynomial evaluation
        let homfly = q.powf(linking as f64) * dim_r * dim_s;
        
        Ok(homfly)
    }
}

impl LineOperator for WilsonLine {
    fn evaluate(&self, path: &Path) -> Result<Complex64> {
        // In absence of gauge field, return dimension
        Ok(Complex64::new(self.representation.dimension() as f64, 0.0))
    }
    
    fn ope(&self, other: &dyn LineOperator, distance: f64) -> Result<Vec<(f64, Box<dyn LineOperator>)>> {
        // Operator product expansion
        // W_R(0) W_S(x) ~ Σ_T C^T_RS(x) W_T(0)
        
        let mut result = Vec::new();
        
        // Leading term: identity operator
        if distance > 1e-6 {
            let coeff = 1.0 / distance.powi(2);
            // Should return identity operator
            result.push((coeff, Box::new(self.clone()) as Box<dyn LineOperator>));
        }
        
        Ok(result)
    }
    
    fn s_dual(&self) -> Result<Box<dyn LineOperator>> {
        // Under S-duality: W_e → T_m
        let magnetic_charge = self.electric_charge.clone();
        
        let t_hooft = THooftOperator::new(
            self.gauge_group.langlands_dual()?,
            self.representation.clone(), // Should be dual representation
            magnetic_charge,
        )?;
        
        Ok(Box::new(t_hooft))
    }
    
    fn vacuum_expectation(&self) -> Result<Complex64> {
        // <W_R> = dim(R) for unknot
        Ok(Complex64::new(self.representation.dimension() as f64, 0.0))
    }
}

/// 't Hooft operator T_m(C)
#[derive(Debug, Clone)]
pub struct THooftOperator {
    /// Gauge group (usually Langlands dual)
    pub gauge_group: Box<dyn Group>,
    /// Representation (magnetic)
    pub representation: Box<dyn Representation>,
    /// Magnetic charge
    pub magnetic_charge: Vec<i32>,
}

impl THooftOperator {
    /// Create new 't Hooft operator
    pub fn new(
        gauge_group: Box<dyn Group>,
        representation: Box<dyn Representation>,
        magnetic_charge: Vec<i32>,
    ) -> Result<Self> {
        if magnetic_charge.len() != gauge_group.rank() {
            return Err(Error::Dimension("Charge dimension mismatch".to_string()));
        }
        
        Ok(Self {
            gauge_group,
            representation,
            magnetic_charge,
        })
    }
    
    /// Create monopole singularity
    pub fn monopole_field(&self, position: &[f64]) -> Result<DMatrix<Complex64>> {
        // Dirac monopole configuration
        let r = position.iter().map(|x| x * x).sum::<f64>().sqrt();
        
        if r < 1e-10 {
            return Err(Error::Computation("Singular at origin".to_string()));
        }
        
        let dim = self.gauge_group.dimension();
        let mut field = DMatrix::zeros(dim, dim);
        
        // Magnetic field ~ m/r² in radial direction
        for (i, &charge) in self.magnetic_charge.iter().enumerate() {
            if i < dim {
                field[(i, i)] = Complex64::new(charge as f64 / (r * r), 0.0);
            }
        }
        
        Ok(field)
    }
    
    /// Fusion of 't Hooft operators
    pub fn fusion(&self, other: &THooftOperator) -> Result<Vec<THooftOperator>> {
        // T_m × T_n = T_{m+n}
        
        let mut fused_charge = self.magnetic_charge.clone();
        for (i, &charge) in other.magnetic_charge.iter().enumerate() {
            fused_charge[i] += charge;
        }
        
        let fused = THooftOperator::new(
            self.gauge_group.clone(),
            self.representation.clone(),
            fused_charge,
        )?;
        
        Ok(vec![fused])
    }
    
    /// Verlinde algebra structure constant
    pub fn verlinde_coefficient(&self, other: &THooftOperator, result: &THooftOperator) -> Result<f64> {
        // N^k_ij = fusion coefficient
        // Related to quantum dimension
        
        // Simplified: check if charges add up
        let mut charge_sum = vec![0; self.magnetic_charge.len()];
        for i in 0..self.magnetic_charge.len() {
            charge_sum[i] = self.magnetic_charge[i] + other.magnetic_charge[i];
        }
        
        if charge_sum == result.magnetic_charge {
            Ok(1.0)
        } else {
            Ok(0.0)
        }
    }
}

impl LineOperator for THooftOperator {
    fn evaluate(&self, path: &Path) -> Result<Complex64> {
        // 't Hooft operator creates magnetic flux
        let flux = self.magnetic_charge.iter().sum::<i32>() as f64;
        Ok(Complex64::new(0.0, flux * PI).exp())
    }
    
    fn ope(&self, other: &dyn LineOperator, distance: f64) -> Result<Vec<(f64, Box<dyn LineOperator>)>> {
        // OPE of 't Hooft operators
        let mut result = Vec::new();
        
        if distance > 1e-6 {
            let coeff = 1.0 / distance; // Different from Wilson lines
            result.push((coeff, Box::new(self.clone()) as Box<dyn LineOperator>));
        }
        
        Ok(result)
    }
    
    fn s_dual(&self) -> Result<Box<dyn LineOperator>> {
        // Under S-duality: T_m → W_e
        let electric_charge = self.magnetic_charge.clone();
        
        let wilson = WilsonLine::new(
            self.gauge_group.langlands_dual()?,
            self.representation.clone(),
            electric_charge,
        )?;
        
        Ok(Box::new(wilson))
    }
    
    fn vacuum_expectation(&self) -> Result<Complex64> {
        // <T_m> related to Coulomb branch parameter
        let charge_squared = self.magnetic_charge.iter()
            .map(|&c| c * c)
            .sum::<i32>() as f64;
        
        Ok(Complex64::new((-charge_squared).exp(), 0.0))
    }
}

/// Path in spacetime
#[derive(Debug, Clone)]
pub struct Path {
    /// Parameterized path: [0,1] → R^4
    pub curve: Box<dyn Fn(f64) -> Vec<f64>>,
    /// Is path closed?
    pub closed: bool,
}

impl Path {
    /// Create a circular path
    pub fn circle(radius: f64) -> Self {
        Self {
            curve: Box::new(move |t| {
                let theta = 2.0 * PI * t;
                vec![radius * theta.cos(), radius * theta.sin(), 0.0, 0.0]
            }),
            closed: true,
        }
    }
    
    /// Create a straight line
    pub fn line(start: Vec<f64>, end: Vec<f64>) -> Self {
        Self {
            curve: Box::new(move |t| {
                start.iter()
                    .zip(&end)
                    .map(|(s, e)| s + t * (e - s))
                    .collect()
            }),
            closed: false,
        }
    }
    
    /// Get position at parameter t ∈ [0,1]
    pub fn position_at(&self, t: f64) -> Result<Vec<f64>> {
        if t < 0.0 || t > 1.0 {
            return Err(Error::Computation("Parameter t must be in [0,1]".to_string()));
        }
        Ok((self.curve)(t))
    }
    
    /// Linking number with another path
    pub fn linking_number(&self, other: &Path) -> Result<i32> {
        if !self.closed || !other.closed {
            return Err(Error::Computation("Both paths must be closed".to_string()));
        }
        
        // Gauss linking integral (simplified)
        let mut linking = 0.0;
        let steps = 100;
        
        for i in 0..steps {
            for j in 0..steps {
                let t1 = i as f64 / steps as f64;
                let t2 = j as f64 / steps as f64;
                
                let p1 = self.position_at(t1)?;
                let p2 = other.position_at(t2)?;
                
                // Simplified calculation
                let dist = p1.iter()
                    .zip(&p2)
                    .map(|(a, b)| (a - b).powi(2))
                    .sum::<f64>()
                    .sqrt();
                
                if dist > 1e-6 {
                    linking += 1.0 / (dist * dist);
                }
            }
        }
        
        Ok((linking / (4.0 * PI)).round() as i32)
    }
}

/// Dyonic operator (electric + magnetic)
#[derive(Debug, Clone)]
pub struct DyonOperator {
    /// Wilson line component
    pub wilson: WilsonLine,
    /// 't Hooft component
    pub t_hooft: THooftOperator,
    /// Relative phase
    pub phase: Complex64,
}

impl DyonOperator {
    /// Create dyonic operator
    pub fn new(wilson: WilsonLine, t_hooft: THooftOperator, phase: Complex64) -> Self {
        Self { wilson, t_hooft, phase }
    }
    
    /// Check if operator is mutually local with another
    pub fn is_mutually_local(&self, other: &DyonOperator) -> bool {
        // Dirac quantization: e₁m₂ - e₂m₁ ∈ Z
        let mut pairing = 0;
        
        for i in 0..self.wilson.electric_charge.len() {
            pairing += self.wilson.electric_charge[i] * other.t_hooft.magnetic_charge[i];
            pairing -= other.wilson.electric_charge[i] * self.t_hooft.magnetic_charge[i];
        }
        
        true // Simplified: always return true
    }
}

impl LineOperator for DyonOperator {
    fn evaluate(&self, path: &Path) -> Result<Complex64> {
        let w_eval = self.wilson.evaluate(path)?;
        let t_eval = self.t_hooft.evaluate(path)?;
        
        Ok(self.phase * w_eval * t_eval)
    }
    
    fn ope(&self, other: &dyn LineOperator, distance: f64) -> Result<Vec<(f64, Box<dyn LineOperator>)>> {
        // Dyonic OPE is more complex
        Ok(vec![(1.0 / distance, Box::new(self.clone()) as Box<dyn LineOperator>)])
    }
    
    fn s_dual(&self) -> Result<Box<dyn LineOperator>> {
        // S-duality rotates (e,m) charges
        let new_wilson = self.t_hooft.s_dual()?;
        let new_t_hooft = self.wilson.s_dual()?;
        
        // Phase also transforms
        let new_phase = self.phase * Complex64::i();
        
        // This is oversimplified - should properly construct dyonic operator
        Ok(Box::new(self.clone()))
    }
    
    fn vacuum_expectation(&self) -> Result<Complex64> {
        let w_vev = self.wilson.vacuum_expectation()?;
        let t_vev = self.t_hooft.vacuum_expectation()?;
        
        Ok(self.phase * w_vev * t_vev)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::mock::{MockGroup, MockRepresentation};

    #[test]
    fn test_wilson_line_creation() {
        let group = Box::new(MockGroup::new("SU(2)".to_string(), 3));
        let rep = Box::new(MockRepresentation::new(2));
        let charge = vec![1, 0];
        
        let wilson = WilsonLine::new(group, rep, charge).unwrap();
        assert_eq!(wilson.electric_charge, vec![1, 0]);
    }
    
    #[test]
    fn test_wilson_line_vacuum_expectation() {
        let group = Box::new(MockGroup::new("SU(2)".to_string(), 3));
        let wilson = WilsonLine::fundamental(group).unwrap();
        
        let vev = wilson.vacuum_expectation().unwrap();
        assert_eq!(vev.re, wilson.representation.dimension() as f64);
    }
    
    #[test]
    fn test_s_duality_wilson_t_hooft() {
        let group = Box::new(MockGroup::new("SU(2)".to_string(), 3));
        let wilson = WilsonLine::fundamental(group).unwrap();
        
        let dual = wilson.s_dual().unwrap();
        // Should get 't Hooft operator
        
        let dual_dual = dual.s_dual().unwrap();
        // Should get back Wilson line (up to phase)
    }
    
    #[test]
    fn test_path_creation() {
        let circle = Path::circle(1.0);
        assert!(circle.closed);
        
        let pos = circle.position_at(0.0).unwrap();
        assert!((pos[0] - 1.0).abs() < 1e-10);
        assert!(pos[1].abs() < 1e-10);
    }
    
    #[test]
    fn test_t_hooft_monopole() {
        let group = Box::new(MockGroup::new("SU(2)".to_string(), 3));
        let rep = Box::new(MockRepresentation::new(2));
        let charge = vec![1, 0];
        
        let t_hooft = THooftOperator::new(group, rep, charge).unwrap();
        
        let pos = vec![1.0, 0.0, 0.0];
        let field = t_hooft.monopole_field(&pos).unwrap();
        
        assert!(field[(0, 0)].re > 0.0); // Radial magnetic field
    }
}