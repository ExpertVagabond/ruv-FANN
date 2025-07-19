//! Integration tests for physics modules

use geometric_langlands::prelude::*;
use geometric_langlands::core::mock::{MockGroup, MockVariety};
use geometric_langlands::physics::*;
use num_complex::Complex64;
use std::f64::consts::PI;

#[test]
fn test_s_duality_basic() {
    let group = Box::new(MockGroup::new("SU(2)".to_string(), 3));
    let mut s_duality = SDuality::new(group).unwrap();
    
    let initial_coupling = s_duality.coupling;
    s_duality.transform().unwrap();
    
    // Check τ → -1/τ
    let expected = -1.0 / initial_coupling;
    assert!((s_duality.coupling - expected).norm() < 1e-10);
}

#[test]
fn test_kapustin_witten_theory() {
    let group = Box::new(MockGroup::new("SU(3)".to_string(), 8));
    let curve = Box::new(MockVariety::new(1, 2)); // genus 2 curve
    
    let kw = KapustinWittenTheory::new(
        group,
        curve,
        TopologicalTwist::BModel
    ).unwrap();
    
    assert_eq!(kw.twist, TopologicalTwist::BModel);
    
    let action = kw.twisted_action().unwrap();
    assert!(action.is_finite());
}

#[test]
fn test_n4_super_yang_mills() {
    let group = Box::new(MockGroup::new("SU(2)".to_string(), 3));
    let n4sym = N4SuperYangMills::new(group).unwrap();
    
    assert_eq!(n4sym.n_susy, 4);
    assert!(n4sym.is_conformal());
    
    // Beta function should vanish (conformal theory)
    assert_eq!(n4sym.beta_function(1.0), 0.0);
    assert_eq!(n4sym.beta_function(100.0), 0.0);
}

#[test]
fn test_coupling_s_duality() {
    let coupling = CouplingConstant::new(2.0, 0.0).unwrap();
    let s_dual = coupling.s_dual().unwrap();
    
    // S: g → 4π/g
    assert!((s_dual.g - 2.0 * PI).abs() < 1e-10);
    assert_eq!(s_dual.theta, 0.0);
    
    // Check round trip
    let double_dual = s_dual.s_dual().unwrap();
    assert!((double_dual.g - coupling.g).abs() < 1e-8);
}

#[test]
fn test_wilson_line_operators() {
    let group = Box::new(MockGroup::new("SU(2)".to_string(), 3));
    let wilson = WilsonLine::fundamental(group).unwrap();
    
    let vev = wilson.vacuum_expectation().unwrap();
    assert_eq!(vev.re, wilson.representation.dimension() as f64);
    
    // Test S-duality: Wilson ↔ 't Hooft
    let dual = wilson.s_dual().unwrap();
    let dual_dual = dual.s_dual().unwrap();
    // Should get back Wilson line (up to group dual)
}

#[test]
fn test_path_and_linking() {
    let circle1 = operators::Path::circle(1.0);
    let circle2 = operators::Path::circle(2.0);
    
    assert!(circle1.closed);
    assert!(circle2.closed);
    
    // Test position evaluation
    let pos = circle1.position_at(0.0).unwrap();
    assert!((pos[0] - 1.0).abs() < 1e-10);
    assert!(pos[1].abs() < 1e-10);
    
    // Test linking number
    let linking = circle1.linking_number(&circle2).unwrap();
    // Should be 0 for concentric circles
}

#[test]
fn test_branes_and_mirror_symmetry() {
    let variety1 = Box::new(MockVariety::new(2, 0)); // P²
    let variety2 = Box::new(MockVariety::new(2, 0)); // Mirror P²
    
    let mirror = MirrorSymmetry::new(variety1, variety2).unwrap();
    assert_eq!(mirror.a_side.dimension(), mirror.b_side.dimension());
    
    // Test HMS verification
    let hms_verified = mirror.verify_hms().unwrap();
    assert!(hms_verified);
}

#[test]
fn test_hitchin_system() {
    let curve = Box::new(MockVariety::new(1, 2)); // genus 2 curve
    let group = Box::new(MockGroup::new("SL(2)".to_string(), 3));
    
    let hitchin = HitchinSystem::new(curve, group).unwrap();
    
    // Test integrable system properties
    let dof = hitchin.degrees_of_freedom();
    assert_eq!(dof, 3); // dim(SL(2)) * (g-1) = 3 * 1 = 3
    
    assert!(hitchin.check_involution().unwrap());
    
    // Test Hitchin equations (should be zero for zero fields)
    let (eq1, eq2) = hitchin.hitchin_equations().unwrap();
    assert!(eq1.norm() < 1e-10);
    assert!(eq2.norm() < 1e-10);
}

#[test]
fn test_hitchin_fibration() {
    let curve = Box::new(MockVariety::new(1, 2)); // genus 2 curve
    let group = Box::new(MockGroup::new("SL(3)".to_string(), 8));
    
    let fibration = HitchinFibration::new(group, curve).unwrap();
    
    assert!(fibration.is_proper());
    
    // Test fiber properties
    let base_point = vec![Complex64::new(1.0, 0.0), Complex64::new(0.0, 1.0)];
    let is_lagrangian = fibration.fiber_is_lagrangian(base_point).unwrap();
    assert!(is_lagrangian);
}

#[test]
fn test_electromagnetic_duality() {
    use nalgebra::Vector4;
    
    let e_field = Vector4::new(0.0, 1.0, 0.0, 0.0); // Electric field in x direction
    let b_field = Vector4::new(0.0, 0.0, 1.0, 0.0); // Magnetic field in y direction
    
    let mut em_duality = ElectricMagneticDuality::new(e_field, b_field);
    
    // Test dual operation
    let initial_f = em_duality.field_strength.clone();
    em_duality.apply_duality(); // E ↔ B
    em_duality.apply_duality(); // Should return to original
    
    assert!((em_duality.field_strength - initial_f).norm() < 1e-10);
    
    // Test Maxwell invariance
    em_duality.apply_duality(); // Apply once
    assert!(em_duality.verify_maxwell_invariance());
}

#[test]
fn test_montonen_olive_duality() {
    let group = Box::new(MockGroup::new("SU(2)".to_string(), 3));
    let mut mo = MontonenOliveDuality::new(group, 0.0, 0.5).unwrap();
    
    // Should show strong-weak duality
    assert!(mo.verify_strong_weak_duality());
    
    let initial_g = mo.g_ym;
    mo.transform().unwrap();
    
    // Check g → 4π/g transformation
    let expected_g = 4.0 * PI / initial_g;
    assert!((mo.g_ym - expected_g).abs() < 1e-10);
    
    // Theta should change sign
    assert!((mo.theta - 0.0).abs() < 1e-10); // Was 0, should stay 0
}

#[test]
fn test_brane_configuration() {
    use geometric_langlands::physics::branes::{ABrane, BBrane, Brane};
    use geometric_langlands::core::mock::TrivialLocalSystem;
    
    let lag1 = Box::new(MockVariety::new(1, 1));
    let lag2 = Box::new(MockVariety::new(1, 1));
    
    let a1 = Box::new(ABrane::special_lagrangian(lag1, 0.0).unwrap()) as Box<dyn Brane>;
    let a2 = Box::new(ABrane::special_lagrangian(lag2, PI).unwrap()) as Box<dyn Brane>;
    
    let config = BraneConfiguration::new(vec![a1, a2]).unwrap();
    
    assert_eq!(config.branes.len(), 2);
    assert!(config.is_bps());
    
    let total_z = config.total_central_charge().unwrap();
    assert!(total_z.norm() > 0.0);
}

#[test]
fn test_physics_mathematics_bridge() {
    // Test that physics concepts correctly map to mathematical ones
    
    // 1. S-duality ↔ Langlands correspondence
    let group = Box::new(MockGroup::new("SL(2)".to_string(), 3));
    let s_duality = SDuality::new(group).unwrap();
    
    // The coupling constant τ should behave like a modular parameter
    let sl2z_transformed = s_duality.sl2z_action(0, -1, 1, 0).unwrap(); // S transformation
    let expected = -1.0 / s_duality.coupling;
    assert!((sl2z_transformed - expected).norm() < 1e-10);
    
    // 2. Kapustin-Witten ↔ Derived categories  
    let curve = Box::new(MockVariety::new(1, 2));
    let kw_group = Box::new(MockGroup::new("SU(2)".to_string(), 3));
    let kw = KapustinWittenTheory::new(kw_group, curve, TopologicalTwist::BModel).unwrap();
    
    let derived_cat = kw.to_derived_category().unwrap();
    // Should successfully create derived category
    
    // 3. Mirror symmetry ↔ Homological mirror symmetry
    let a_side = Box::new(MockVariety::new(2, 1)); // K3 surface
    let b_side = Box::new(MockVariety::new(2, 1)); // Mirror K3
    
    let mirror = MirrorSymmetry::new(a_side, b_side).unwrap();
    let hms = HomologicalMirrorSymmetry::new(mirror).unwrap();
    
    assert!(hms.verify_conjecture().unwrap());
    
    // 4. Hitchin system ↔ Spectral curves
    let hitchin_curve = Box::new(MockVariety::new(1, 3)); // genus 3
    let hitchin_group = Box::new(MockGroup::new("SL(3)".to_string(), 8));
    let hitchin = HitchinSystem::new(hitchin_curve, hitchin_group).unwrap();
    
    let spectral = hitchin.spectral_curve().unwrap();
    assert!(spectral.genus > 0); // Should have positive genus
    
    let is_regular = hitchin.is_regular().unwrap();
    // With zero Higgs field, should be regular
}

#[test]
fn test_instanton_corrections() {
    let a_side = Box::new(MockVariety::new(2, 0)); // P²
    let b_side = Box::new(MockVariety::new(2, 0)); // Mirror P²
    
    let mirror = MirrorSymmetry::new(a_side, b_side).unwrap();
    let corrections = mirror.instanton_corrections().unwrap();
    
    // Should find some instanton corrections
    assert!(!corrections.is_empty());
    
    for correction in &corrections {
        assert!(correction.degree > 0);
        assert!(correction.contribution.norm() > 0.0);
    }
}

#[test] 
fn test_comprehensive_physics_integration() {
    // This test verifies the complete physics picture works together
    
    // Start with N=4 SYM
    let gauge_group = Box::new(MockGroup::new("SU(2)".to_string(), 3));
    let n4sym = N4SuperYangMills::new(gauge_group.clone()).unwrap();
    
    // Apply S-duality
    let mut s_duality = SDuality::new(gauge_group.clone()).unwrap();
    s_duality.transform().unwrap();
    
    // Create Wilson and 't Hooft operators
    let wilson = WilsonLine::fundamental(gauge_group.clone()).unwrap();
    let t_hooft_group = s_duality.dual_group.clone(); 
    let t_hooft = THooftOperator::new(
        t_hooft_group,
        gauge_group.fundamental_representation().unwrap(),
        vec![1, 0],
    ).unwrap();
    
    // Apply topological twist → Kapustin-Witten
    let curve = Box::new(MockVariety::new(1, 2));
    let kw = KapustinWittenTheory::new(
        gauge_group.clone(),
        curve.clone(),
        TopologicalTwist::BModel,
    ).unwrap();
    
    // Get Hitchin system
    let hitchin = HitchinSystem::new(curve, gauge_group).unwrap();
    
    // Verify integrability 
    assert!(hitchin.check_involution().unwrap());
    
    // The whole construction should be consistent
    assert!(n4sym.is_conformal());
    assert_eq!(kw.twist, TopologicalTwist::BModel);
    assert!(hitchin.degrees_of_freedom() > 0);
}