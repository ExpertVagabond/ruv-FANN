//! # N=4 Super Yang-Mills Theory
//!
//! This module implements N=4 supersymmetric Yang-Mills theory, which provides
//! the physical foundation for S-duality and the Geometric Langlands correspondence.

use crate::core::{Field, Group, LieAlgebra};
use crate::representation::Representation;
use crate::error::{Error, Result};
use num_complex::Complex64;
use nalgebra::{DMatrix, DVector, Matrix4, Vector4};
use std::f64::consts::PI;

/// N=4 Super Yang-Mills theory
#[derive(Debug, Clone)]
pub struct N4SuperYangMills {
    /// Gauge group
    pub gauge_group: Box<dyn Group>,
    /// Lie algebra
    pub lie_algebra: Box<dyn LieAlgebra>,
    /// Coupling constant
    pub coupling: CouplingConstant,
    /// Number of supersymmetries (should be 4)
    pub n_susy: usize,
}

impl N4SuperYangMills {
    /// Create new N=4 SYM theory
    pub fn new(gauge_group: Box<dyn Group>) -> Result<Self> {
        let lie_algebra = gauge_group.lie_algebra()?;
        let coupling = CouplingConstant::new(1.0, 0.0)?; // g=1, θ=0
        
        Ok(Self {
            gauge_group,
            lie_algebra,
            coupling,
            n_susy: 4,
        })
    }
    
    /// The N=4 SYM action
    pub fn action(
        &self,
        gauge_field: &DMatrix<Complex64>,
        scalars: &[DMatrix<Complex64>; 6],
        fermions: &[DMatrix<Complex64>; 4],
    ) -> Result<f64> {
        // S = S_gauge + S_scalar + S_fermion + S_interaction
        
        let s_gauge = self.gauge_action(gauge_field)?;
        let s_scalar = self.scalar_action(scalars)?;
        let s_fermion = self.fermion_action(fermions)?;
        let s_int = self.interaction_action(gauge_field, scalars, fermions)?;
        
        Ok(s_gauge + s_scalar + s_fermion + s_int)
    }
    
    /// Yang-Mills gauge action
    fn gauge_action(&self, gauge_field: &DMatrix<Complex64>) -> Result<f64> {
        // S_gauge = (1/g²) ∫ Tr(F_μν F^μν) + (θ/8π²) ∫ Tr(F_μν *F^μν)
        
        let f_squared = self.field_strength_squared(gauge_field)?;
        let f_dual_f = self.field_strength_dual_product(gauge_field)?;
        
        let yang_mills_term = f_squared / (self.coupling.g * self.coupling.g);
        let theta_term = self.coupling.theta * f_dual_f / (8.0 * PI * PI);
        
        Ok(yang_mills_term + theta_term)
    }
    
    /// Scalar kinetic terms
    fn scalar_action(&self, scalars: &[DMatrix<Complex64>; 6]) -> Result<f64> {
        // S_scalar = (1/g²) ∫ Tr(D_μ Φ^I D^μ Φ_I)
        
        let mut action = 0.0;
        
        for scalar in scalars {
            // Simplified: just use Tr(Φ² ) for now
            action += scalar.iter()
                .map(|x| x.norm_sqr())
                .sum::<f64>();
        }
        
        Ok(action / (self.coupling.g * self.coupling.g))
    }
    
    /// Fermion kinetic terms
    fn fermion_action(&self, fermions: &[DMatrix<Complex64>; 4]) -> Result<f64> {
        // S_fermion = (1/g²) ∫ Tr(ψ^A γ^μ D_μ ψ_A)
        
        let mut action = 0.0;
        
        for fermion in fermions {
            // Simplified: just kinetic term
            action += fermion.iter()
                .map(|x| x.norm_sqr())
                .sum::<f64>();
        }
        
        Ok(action / (self.coupling.g * self.coupling.g))
    }
    
    /// Interaction terms
    fn interaction_action(
        &self,
        gauge_field: &DMatrix<Complex64>,
        scalars: &[DMatrix<Complex64>; 6],
        fermions: &[DMatrix<Complex64>; 4],
    ) -> Result<f64> {
        // Yukawa and scalar potential terms
        // These preserve N=4 supersymmetry
        
        // Simplified implementation
        Ok(0.0)
    }
    
    /// Compute field strength F_μν
    fn field_strength(&self, gauge_field: &DMatrix<Complex64>) -> Result<DMatrix<Complex64>> {
        // F_μν = ∂_μ A_ν - ∂_ν A_μ + [A_μ, A_ν]
        
        let dim = gauge_field.nrows();
        let mut f = DMatrix::zeros(dim, dim);
        
        // Simplified: just use [A_μ, A_ν] part
        if dim >= 2 {
            f = gauge_field * gauge_field - gauge_field.transpose() * gauge_field.transpose();
        }
        
        Ok(f)
    }
    
    /// Compute Tr(F²)
    fn field_strength_squared(&self, gauge_field: &DMatrix<Complex64>) -> Result<f64> {
        let f = self.field_strength(gauge_field)?;
        Ok(f.iter().map(|x| x.norm_sqr()).sum())
    }
    
    /// Compute Tr(F ∧ *F)
    fn field_strength_dual_product(&self, gauge_field: &DMatrix<Complex64>) -> Result<f64> {
        // In 4D, *F_μν = (1/2) ε_μνρσ F^ρσ
        // For topological term
        
        let f = self.field_strength(gauge_field)?;
        // Simplified: proportional to Tr(F²) in this implementation
        Ok(f.iter().map(|x| x.norm_sqr()).sum() * 0.5)
    }
    
    /// Beta function for running coupling
    pub fn beta_function(&self, energy_scale: f64) -> f64 {
        // N=4 SYM is conformal: β(g) = 0
        // This is crucial for S-duality
        0.0
    }
    
    /// Check conformal invariance
    pub fn is_conformal(&self) -> bool {
        // N=4 SYM is exactly conformal
        self.n_susy == 4 && self.beta_function(1.0).abs() < 1e-10
    }
    
    /// Compute central charge
    pub fn central_charge(&self) -> Result<f64> {
        // c = dim(G) for N=4 SYM
        Ok(self.gauge_group.dimension() as f64)
    }
    
    /// BPS states (1/2 BPS)
    pub fn bps_states(&self, charges: Vec<i32>) -> Result<Vec<BPSState>> {
        let mut states = Vec::new();
        
        for (i, &charge) in charges.iter().enumerate() {
            if charge != 0 {
                states.push(BPSState {
                    electric_charge: charge,
                    magnetic_charge: 0,
                    central_charge: charge as f64,
                    mass: (charge as f64).abs() / self.coupling.g,
                });
            }
        }
        
        Ok(states)
    }
    
    /// 't Hooft loop eigenvalues
    pub fn t_hooft_eigenvalues(&self, magnetic_charges: Vec<i32>) -> Result<Vec<Complex64>> {
        let mut eigenvalues = Vec::new();
        
        for &m in &magnetic_charges {
            // Eigenvalue depends on magnetic charge and theta angle
            let phase = self.coupling.theta * m as f64 / (2.0 * PI);
            let magnitude = (4.0 * PI / self.coupling.g).powf(m as f64);
            
            eigenvalues.push(Complex64::from_polar(magnitude, phase));
        }
        
        Ok(eigenvalues)
    }
    
    /// Wilson loop eigenvalues  
    pub fn wilson_eigenvalues(&self, electric_charges: Vec<i32>) -> Result<Vec<Complex64>> {
        let mut eigenvalues = Vec::new();
        
        for &e in &electric_charges {
            // Eigenvalue depends on electric charge
            let phase = 2.0 * PI * e as f64;
            let magnitude = (-self.coupling.g * e as f64).exp();
            
            eigenvalues.push(Complex64::from_polar(magnitude, phase));
        }
        
        Ok(eigenvalues)
    }
}

/// Gauge theory coupling constant
#[derive(Debug, Clone)]
pub struct CouplingConstant {
    /// Yang-Mills coupling g
    pub g: f64,
    /// Theta angle
    pub theta: f64,
    /// Complexified coupling τ = θ/2π + 4πi/g²
    pub tau: Complex64,
}

impl CouplingConstant {
    /// Create new coupling constant
    pub fn new(g: f64, theta: f64) -> Result<Self> {
        if g <= 0.0 {
            return Err(Error::Computation("Coupling must be positive".to_string()));
        }
        
        let tau = Complex64::new(theta / (2.0 * PI), 4.0 * PI / (g * g));
        
        Ok(Self { g, theta, tau })
    }
    
    /// S-duality transformation
    pub fn s_dual(&self) -> Result<Self> {
        // S: τ → -1/τ, g → 4π/g, θ → -θ
        
        if self.tau.norm() < 1e-10 {
            return Err(Error::Computation("Singular coupling".to_string()));
        }
        
        let new_tau = -1.0 / self.tau;
        let new_g = 4.0 * PI / self.g;
        let new_theta = -self.theta;
        
        Self::new(new_g, new_theta)
    }
    
    /// T-duality transformation  
    pub fn t_dual(&self) -> Result<Self> {
        // T: τ → τ + 1, θ → θ + 2π
        
        let new_theta = self.theta + 2.0 * PI;
        Self::new(self.g, new_theta)
    }
    
    /// Check weak coupling
    pub fn is_weak_coupling(&self) -> bool {
        self.g < 1.0
    }
    
    /// Check strong coupling
    pub fn is_strong_coupling(&self) -> bool {
        self.g > 1.0
    }
}

/// Generic gauge theory interface
pub trait GaugeTheory {
    /// Gauge group
    fn gauge_group(&self) -> &dyn Group;
    
    /// Coupling constant
    fn coupling(&self) -> &CouplingConstant;
    
    /// Compute beta function
    fn beta_function(&self, scale: f64) -> f64;
    
    /// Check if theory is asymptotically free
    fn is_asymptotically_free(&self) -> bool {
        self.beta_function(1.0) < 0.0
    }
    
    /// Check if theory is conformal
    fn is_conformal(&self) -> bool {
        self.beta_function(1.0).abs() < 1e-10
    }
}

impl GaugeTheory for N4SuperYangMills {
    fn gauge_group(&self) -> &dyn Group {
        &*self.gauge_group
    }
    
    fn coupling(&self) -> &CouplingConstant {
        &self.coupling
    }
    
    fn beta_function(&self, scale: f64) -> f64 {
        self.beta_function(scale)
    }
}

/// BPS state in N=4 SYM
#[derive(Debug, Clone)]
pub struct BPSState {
    /// Electric charge
    pub electric_charge: i32,
    /// Magnetic charge  
    pub magnetic_charge: i32,
    /// Central charge Z
    pub central_charge: f64,
    /// BPS mass
    pub mass: f64,
}

impl BPSState {
    /// Check if state is BPS
    pub fn is_bps(&self) -> bool {
        // BPS bound: M = |Z|
        (self.mass - self.central_charge.abs()).abs() < 1e-10
    }
    
    /// Check if state is 1/2 BPS
    pub fn is_half_bps(&self) -> bool {
        // Either purely electric or purely magnetic
        self.electric_charge == 0 || self.magnetic_charge == 0
    }
    
    /// Check if state is 1/4 BPS
    pub fn is_quarter_bps(&self) -> bool {
        // Both electric and magnetic charges
        self.electric_charge != 0 && self.magnetic_charge != 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::mock::MockGroup;

    #[test]
    fn test_n4_sym_creation() {
        let group = Box::new(MockGroup::new("SU(2)".to_string(), 3));
        let n4sym = N4SuperYangMills::new(group).unwrap();
        
        assert_eq!(n4sym.n_susy, 4);
        assert!(n4sym.is_conformal());
    }
    
    #[test]
    fn test_coupling_s_duality() {
        let coupling = CouplingConstant::new(2.0, 0.0).unwrap();
        let s_dual = coupling.s_dual().unwrap();
        
        assert!((s_dual.g - 2.0 * PI).abs() < 1e-10);
        assert_eq!(s_dual.theta, 0.0);
    }
    
    #[test]
    fn test_coupling_t_duality() {
        let coupling = CouplingConstant::new(1.0, PI).unwrap();
        let t_dual = coupling.t_dual().unwrap();
        
        assert_eq!(t_dual.g, 1.0);
        assert!((t_dual.theta - 3.0 * PI).abs() < 1e-10);
    }
    
    #[test]
    fn test_bps_states() {
        let group = Box::new(MockGroup::new("SU(2)".to_string(), 3));
        let n4sym = N4SuperYangMills::new(group).unwrap();
        
        let states = n4sym.bps_states(vec![1, 0, -1]).unwrap();
        
        assert_eq!(states.len(), 2);
        assert!(states[0].is_bps());
        assert!(states[0].is_half_bps());
    }
    
    #[test]
    fn test_conformal_invariance() {
        let group = Box::new(MockGroup::new("SU(3)".to_string(), 8));
        let n4sym = N4SuperYangMills::new(group).unwrap();
        
        // Beta function should vanish at all scales
        assert_eq!(n4sym.beta_function(1.0), 0.0);
        assert_eq!(n4sym.beta_function(100.0), 0.0);
        assert_eq!(n4sym.beta_function(0.01), 0.0);
    }
}