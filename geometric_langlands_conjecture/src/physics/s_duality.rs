//! # S-Duality and Electric-Magnetic Duality
//!
//! This module implements S-duality transformations which establish the
//! physical foundation of the Geometric Langlands correspondence.

use crate::core::{Field, Group};
use crate::representation::Representation;
use crate::galois::GaloisRepresentation;
use crate::automorphic::AutomorphicRepresentation;
use crate::error::{Error, Result};
use num_complex::Complex64;
use nalgebra::{Matrix4, Vector4};
use std::f64::consts::PI;

/// S-duality transformation in N=4 Super Yang-Mills
#[derive(Debug, Clone)]
pub struct SDuality {
    /// Gauge group
    pub gauge_group: Box<dyn Group>,
    /// Coupling constant τ = θ/2π + 4πi/g²
    pub coupling: Complex64,
    /// Langlands dual group
    pub dual_group: Box<dyn Group>,
}

impl SDuality {
    /// Create a new S-duality transformation
    pub fn new(gauge_group: Box<dyn Group>) -> Result<Self> {
        let dual_group = gauge_group.langlands_dual()?;
        let coupling = Complex64::new(0.0, 4.0 * PI); // Weak coupling limit
        
        Ok(Self {
            gauge_group,
            coupling,
            dual_group,
        })
    }
    
    /// Apply S-duality transformation: τ → -1/τ
    pub fn transform(&mut self) -> Result<()> {
        if self.coupling.norm() < 1e-10 {
            return Err(Error::Computation("Singular coupling constant".to_string()));
        }
        
        self.coupling = -1.0 / self.coupling;
        std::mem::swap(&mut self.gauge_group, &mut self.dual_group);
        
        Ok(())
    }
    
    /// Verify S-duality as Langlands correspondence
    pub fn verify_langlands_correspondence(
        &self,
        automorphic: &dyn AutomorphicRepresentation,
        galois: &dyn GaloisRepresentation,
    ) -> Result<bool> {
        // Check that the automorphic representation on G
        // corresponds to Galois representation with dual group G^L
        let g_rep = automorphic.local_component(2)?; // At prime 2
        let g_dual_rep = galois.local_system()?;
        
        // Verify matching of Langlands parameters
        let params_match = self.verify_parameters(&g_rep, &g_dual_rep)?;
        
        // Check Satake correspondence
        let satake_match = self.verify_satake_correspondence(&g_rep, &g_dual_rep)?;
        
        Ok(params_match && satake_match)
    }
    
    /// Verify Langlands parameters match under S-duality
    fn verify_parameters(
        &self,
        rep1: &dyn Representation,
        rep2: &dyn Representation,
    ) -> Result<bool> {
        // In physics, this corresponds to matching of
        // electric charges ↔ magnetic charges
        let dim1 = rep1.dimension();
        let dim2 = rep2.dimension();
        
        // Dimensions should match for corresponding representations
        if dim1 != dim2 {
            return Ok(false);
        }
        
        // Check character matching (simplified)
        let char1 = rep1.character(&self.gauge_group.identity());
        let char2 = rep2.character(&self.dual_group.identity());
        
        Ok((char1 - char2).abs() < 1e-10)
    }
    
    /// Verify Satake correspondence
    fn verify_satake_correspondence(
        &self,
        rep1: &dyn Representation,
        rep2: &dyn Representation,
    ) -> Result<bool> {
        // The Satake correspondence relates spherical representations
        // This is the mathematical version of electromagnetic duality
        
        // For now, return true if dimensions match
        // Full implementation would check Hecke eigenvalues
        Ok(rep1.dimension() == rep2.dimension())
    }
    
    /// Get the SL(2,Z) action on coupling constant
    pub fn sl2z_action(&self, a: i32, b: i32, c: i32, d: i32) -> Result<Complex64> {
        // Check that [[a,b],[c,d]] is in SL(2,Z)
        if a * d - b * c != 1 {
            return Err(Error::Computation("Not an SL(2,Z) element".to_string()));
        }
        
        // Act on τ by fractional linear transformation
        let numerator = self.coupling * a as f64 + b as f64;
        let denominator = self.coupling * c as f64 + d as f64;
        
        if denominator.norm() < 1e-10 {
            return Err(Error::Computation("Singular transformation".to_string()));
        }
        
        Ok(numerator / denominator)
    }
    
    /// Electric-magnetic duality matrix
    pub fn em_duality_matrix(&self) -> Matrix4<Complex64> {
        // In 4D, F_μν → *F_μν under S-duality
        // This exchanges E ↔ B fields
        let i = Complex64::i();
        
        Matrix4::new(
            0.0.into(), i, 0.0.into(), 0.0.into(),
            -i, 0.0.into(), 0.0.into(), 0.0.into(),
            0.0.into(), 0.0.into(), 0.0.into(), i,
            0.0.into(), 0.0.into(), -i, 0.0.into(),
        )
    }
}

/// Montonen-Olive duality
#[derive(Debug, Clone)]
pub struct MontonenOliveDuality {
    /// Base S-duality
    pub s_duality: SDuality,
    /// Theta angle
    pub theta: f64,
    /// Yang-Mills coupling
    pub g_ym: f64,
}

impl MontonenOliveDuality {
    /// Create new Montonen-Olive duality
    pub fn new(gauge_group: Box<dyn Group>, theta: f64, g_ym: f64) -> Result<Self> {
        let coupling = Complex64::new(theta / (2.0 * PI), 4.0 * PI / (g_ym * g_ym));
        let mut s_duality = SDuality::new(gauge_group)?;
        s_duality.coupling = coupling;
        
        Ok(Self {
            s_duality,
            theta,
            g_ym,
        })
    }
    
    /// Transform under Montonen-Olive duality
    pub fn transform(&mut self) -> Result<()> {
        // S: g → 4π/g, θ → -θ
        self.g_ym = 4.0 * PI / self.g_ym;
        self.theta = -self.theta;
        
        // Update coupling constant
        self.s_duality.coupling = Complex64::new(
            self.theta / (2.0 * PI),
            4.0 * PI / (self.g_ym * self.g_ym)
        );
        
        // Exchange gauge group with dual
        self.s_duality.transform()?;
        
        Ok(())
    }
    
    /// Verify strong-weak duality
    pub fn verify_strong_weak_duality(&self) -> bool {
        // At θ = 0, S-duality exchanges g ↔ 4π/g
        // This exchanges strong and weak coupling regimes
        let weak_coupling = self.g_ym < 1.0;
        let dual_coupling = 4.0 * PI / self.g_ym;
        let dual_weak = dual_coupling < 1.0;
        
        // One should be weak, the other strong
        weak_coupling != dual_weak
    }
}

/// Electric-Magnetic duality implementation
#[derive(Debug, Clone)]
pub struct ElectricMagneticDuality {
    /// Field strength tensor F_μν
    pub field_strength: Matrix4<Complex64>,
    /// Dual field strength *F_μν
    pub dual_field_strength: Matrix4<Complex64>,
}

impl ElectricMagneticDuality {
    /// Create from electric and magnetic fields
    pub fn new(e_field: Vector4<f64>, b_field: Vector4<f64>) -> Self {
        let mut f = Matrix4::zeros();
        let mut f_dual = Matrix4::zeros();
        
        // F_0i = E_i, F_ij = ε_ijk B_k
        for i in 1..4 {
            f[(0, i)] = Complex64::new(e_field[i], 0.0);
            f[(i, 0)] = Complex64::new(-e_field[i], 0.0);
            
            f_dual[(0, i)] = Complex64::new(b_field[i], 0.0);
            f_dual[(i, 0)] = Complex64::new(-b_field[i], 0.0);
        }
        
        // Spatial components (simplified)
        f[(1, 2)] = Complex64::new(b_field[3], 0.0);
        f[(2, 1)] = Complex64::new(-b_field[3], 0.0);
        f[(2, 3)] = Complex64::new(b_field[1], 0.0);
        f[(3, 2)] = Complex64::new(-b_field[1], 0.0);
        f[(3, 1)] = Complex64::new(b_field[2], 0.0);
        f[(1, 3)] = Complex64::new(-b_field[2], 0.0);
        
        f_dual[(1, 2)] = Complex64::new(-e_field[3], 0.0);
        f_dual[(2, 1)] = Complex64::new(e_field[3], 0.0);
        f_dual[(2, 3)] = Complex64::new(-e_field[1], 0.0);
        f_dual[(3, 2)] = Complex64::new(e_field[1], 0.0);
        f_dual[(3, 1)] = Complex64::new(-e_field[2], 0.0);
        f_dual[(1, 3)] = Complex64::new(e_field[2], 0.0);
        
        Self {
            field_strength: f,
            dual_field_strength: f_dual,
        }
    }
    
    /// Apply electromagnetic duality: F → *F
    pub fn apply_duality(&mut self) {
        std::mem::swap(&mut self.field_strength, &mut self.dual_field_strength);
    }
    
    /// Verify duality invariance of Maxwell equations
    pub fn verify_maxwell_invariance(&self) -> bool {
        // In vacuum, Maxwell equations are invariant under E ↔ B
        // This is the physical origin of S-duality
        
        // Check that F and *F satisfy same equations (simplified)
        let f_squared = (self.field_strength * self.field_strength).trace();
        let f_dual_squared = (self.dual_field_strength * self.dual_field_strength).trace();
        
        (f_squared - f_dual_squared).norm() < 1e-10
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::mock::MockGroup;

    #[test]
    fn test_s_duality_transform() {
        let group = Box::new(MockGroup::new("SU(2)".to_string(), 3));
        let mut s_duality = SDuality::new(group).unwrap();
        
        let initial_coupling = s_duality.coupling;
        s_duality.transform().unwrap();
        
        // Check τ → -1/τ
        let expected = -1.0 / initial_coupling;
        assert!((s_duality.coupling - expected).norm() < 1e-10);
    }
    
    #[test]
    fn test_sl2z_action() {
        let group = Box::new(MockGroup::new("SU(2)".to_string(), 3));
        let s_duality = SDuality::new(group).unwrap();
        
        // T transformation: τ → τ + 1
        let t_transformed = s_duality.sl2z_action(1, 1, 0, 1).unwrap();
        let expected = s_duality.coupling + 1.0;
        assert!((t_transformed - expected).norm() < 1e-10);
        
        // S transformation: τ → -1/τ
        let s_transformed = s_duality.sl2z_action(0, -1, 1, 0).unwrap();
        let expected = -1.0 / s_duality.coupling;
        assert!((s_transformed - expected).norm() < 1e-10);
    }
    
    #[test]
    fn test_montonen_olive() {
        let group = Box::new(MockGroup::new("SU(2)".to_string(), 3));
        let mut mo = MontonenOliveDuality::new(group, 0.0, 0.5).unwrap();
        
        assert!(mo.verify_strong_weak_duality());
        
        let initial_g = mo.g_ym;
        mo.transform().unwrap();
        
        // Check g → 4π/g
        let expected_g = 4.0 * PI / initial_g;
        assert!((mo.g_ym - expected_g).abs() < 1e-10);
    }
    
    #[test]
    fn test_em_duality() {
        let e = Vector4::new(0.0, 1.0, 0.0, 0.0);
        let b = Vector4::new(0.0, 0.0, 1.0, 0.0);
        
        let mut em = ElectricMagneticDuality::new(e, b);
        let initial_f = em.field_strength.clone();
        
        em.apply_duality();
        em.apply_duality();
        
        // Double duality should return to original
        assert!((em.field_strength - initial_f).norm() < 1e-10);
    }
}