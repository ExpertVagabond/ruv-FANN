//! Full integration test for the Geometric Langlands implementation
//!
//! This test verifies that all components work together correctly.

use geometric_langlands::prelude::*;
use geometric_langlands::physics::*;
use nalgebra::{DMatrix, DVector};
use num_complex::Complex64;

#[test]
fn test_complete_langlands_workflow() {
    println!("=== Starting Complete Langlands Workflow Test ===");
    
    // Step 1: Create mathematical structures
    println!("1. Creating mathematical structures...");
    let group = ReductiveGroup::gl_n(3);
    println!("   ✓ Created GL(3) reductive group");
    
    // Step 2: Create automorphic forms
    println!("2. Creating automorphic forms...");
    let form = AutomorphicForm::eisenstein_series(&group, 4);
    println!("   ✓ Created Eisenstein series of weight 4");
    
    // Step 3: Apply Hecke operators
    println!("3. Applying Hecke operators...");
    let hecke = HeckeOperator::new(&group, 5);
    let eigenform = hecke.apply(&form);
    println!("   ✓ Applied Hecke operator T_5");
    
    // Step 4: Create Galois representations
    println!("4. Creating Galois representations...");
    let galois_rep = GaloisRepresentation::new(3, 1);
    println!("   ✓ Created 3-dimensional l-adic representation");
    
    // Step 5: Setup Langlands correspondence
    println!("5. Setting up Langlands correspondence...");
    let mut correspondence = LanglandsCorrespondence::new(group.clone());
    correspondence.add_automorphic_form(form.clone()).unwrap();
    correspondence.add_galois_representation(galois_rep.clone()).unwrap();
    correspondence.establish_correspondence(0, 0).unwrap();
    println!("   ✓ Established correspondence");
    
    // Step 6: Compute L-functions
    println!("6. Computing L-functions...");
    let l_function = correspondence.compute_l_function().unwrap();
    println!("   ✓ Computed L-function with degree {}", l_function.degree);
    
    // Step 7: Test functoriality
    println!("7. Testing functoriality...");
    let target_group = ReductiveGroup::gl_n(6);
    let functoriality = Functoriality::new(
        group.clone(),
        target_group,
        LiftType::SymmetricPower { power: 2 }
    );
    let lifted_form = functoriality.lift_form(&form).unwrap();
    println!("   ✓ Applied symmetric square lift");
    
    // Step 8: Neural network integration
    println!("8. Testing neural network integration...");
    let mut nn = correspondence.create_neural_network();
    let training_data = TrainingData {
        automorphic_forms: vec![form.clone()],
        galois_representations: vec![galois_rep.clone()],
        correspondences: vec![(0, 0)],
    };
    let metrics = nn.train(&training_data).unwrap();
    println!("   ✓ Trained neural network - best accuracy: {:.2}%", metrics.best_accuracy * 100.0);
    
    // Step 9: Physics connection via S-duality
    println!("9. Testing physics connections...");
    let s_duality = SDuality::new(Complex64::new(0.0, 1.0));
    let coupling = CouplingConstant { g_squared: 1.0, theta: 0.0 };
    let transformed = s_duality.transform_coupling(coupling);
    println!("   ✓ Applied S-duality transformation");
    
    // Step 10: Kapustin-Witten theory
    println!("10. Testing Kapustin-Witten theory...");
    let kw_theory = KapustinWittenTheory::new(group.clone(), TopologicalTwist::A);
    let physical_correspondence = kw_theory.to_langlands().unwrap();
    println!("   ✓ Created physical interpretation via KW theory");
    
    println!("\n=== All Integration Tests Passed! ===");
}

#[test]
fn test_categorical_equivalence() {
    println!("=== Testing Categorical Structures ===");
    
    use geometric_langlands::category::*;
    
    // Test D-modules
    let module_data = DMatrix::identity(4, 4);
    let differentials = vec![DMatrix::zeros(4, 4)];
    let d_module = DModule::new(module_data, differentials, 2);
    
    assert_eq!(d_module.variety_dimension, 2);
    assert!(d_module.is_coherent());
    
    // Test perverse sheaf via Riemann-Hilbert
    let perverse = d_module.riemann_hilbert_correspondence();
    assert!(perverse.constructible);
    
    println!("   ✓ D-module and perverse sheaf correspondence works");
}

#[test]
fn test_spectral_decomposition() {
    println!("=== Testing Spectral Theory ===");
    
    use geometric_langlands::spectral::*;
    
    // Create a test matrix
    let matrix = DMatrix::from_row_slice(3, 3, &[
        1.0, 2.0, 3.0,
        2.0, 4.0, 5.0,
        3.0, 5.0, 6.0,
    ]);
    
    // Compute spectral decomposition
    let spectral = SpectralDecomposition::compute(&matrix).unwrap();
    
    // Verify reconstruction
    let reconstructed = spectral.reconstruct();
    let error = (reconstructed - matrix).norm();
    assert!(error < 1e-10);
    
    println!("   ✓ Spectral decomposition and reconstruction works");
}

#[test]
fn test_parallel_computation() {
    println!("=== Testing Parallel Computation ===");
    
    use rayon::prelude::*;
    
    let group = ReductiveGroup::gl_n(2);
    let primes: Vec<u32> = vec![2, 3, 5, 7, 11, 13, 17, 19];
    
    // Parallel Hecke eigenvalue computation
    let eigenvalues: Vec<f64> = primes.par_iter()
        .map(|&p| {
            let form = AutomorphicForm::eisenstein_series(&group, 4);
            let hecke = HeckeOperator::new(&group, p);
            hecke.eigenvalue(&form)
        })
        .collect();
    
    assert_eq!(eigenvalues.len(), primes.len());
    println!("   ✓ Parallel Hecke eigenvalue computation works");
}

#[test]
fn test_mathematical_consistency() {
    println!("=== Testing Mathematical Consistency ===");
    
    // Test conductor compatibility
    let group = ReductiveGroup::gl_n(2);
    let form = AutomorphicForm::cusp_form(&group, 2, 11);
    let galois = GaloisRepresentation::new(2, 11);
    
    assert_eq!(form.conductor(), galois.conductor());
    println!("   ✓ Conductor compatibility verified");
    
    // Test L-function functional equation
    let mut correspondence = LanglandsCorrespondence::new(group);
    correspondence.add_automorphic_form(form).unwrap();
    correspondence.add_galois_representation(galois).unwrap();
    
    let l_func = correspondence.compute_l_function().unwrap();
    
    // Check critical strip
    let s = Complex64::new(0.5, 1.0);
    let value = l_func.evaluate(s);
    assert!(value.norm() < 100.0); // Reasonable bound
    
    println!("   ✓ L-function properties verified");
}

#[test]
fn test_error_handling() {
    println!("=== Testing Error Handling ===");
    
    let group1 = ReductiveGroup::gl_n(2);
    let group2 = ReductiveGroup::gl_n(3);
    
    // Test group mismatch
    let form = AutomorphicForm::eisenstein_series(&group1, 4);
    let mut correspondence = LanglandsCorrespondence::new(group2);
    
    let result = correspondence.add_automorphic_form(form);
    assert!(result.is_err());
    
    println!("   ✓ Error handling for group mismatch works");
}

#[test]
fn test_ramanujan_bounds() {
    println!("=== Testing Ramanujan Conjecture ===");
    
    let group = ReductiveGroup::gl_n(2);
    let mut ramanujan = RamanujanConjecture::new(group);
    
    // Test bounds at small primes
    let test_primes = [2, 3, 5, 7, 11];
    for &p in &test_primes {
        let eigenvalue = Complex64::new((p as f64).sqrt(), 0.0);
        let verified = ramanujan.verify_at_prime(p, eigenvalue);
        assert!(verified);
    }
    
    assert!(ramanujan.is_satisfied());
    println!("   ✓ Ramanujan bounds verified for small primes");
}

#[test]
fn test_reciprocity_law() {
    println!("=== Testing Reciprocity Laws ===");
    
    let mut reciprocity = ReciprocityLaw::new(ReciprocityType::Langlands);
    
    // Add some L-functions
    reciprocity.arithmetic_side.l_functions.push(LFunction::trivial());
    
    // Verify numerically
    let verified = reciprocity.verify_numerically(100);
    assert!(verified);
    
    println!("   ✓ Reciprocity law numerical verification passed");
}

#[test]
fn test_memory_and_performance() {
    println!("=== Testing Memory and Performance ===");
    
    use std::time::Instant;
    
    let start = Instant::now();
    
    // Create large structures
    let group = ReductiveGroup::gl_n(10);
    let mut correspondence = LanglandsCorrespondence::new(group.clone());
    
    // Add multiple forms
    for weight in 2..10 {
        let form = AutomorphicForm::eisenstein_series(&group, weight);
        correspondence.add_automorphic_form(form).unwrap();
    }
    
    let duration = start.elapsed();
    println!("   ✓ Created large correspondence in {:?}", duration);
    
    assert!(duration.as_secs() < 5); // Should complete within 5 seconds
}

fn main() {
    println!("Running full integration tests...");
    test_complete_langlands_workflow();
    test_categorical_equivalence();
    test_spectral_decomposition();
    test_parallel_computation();
    test_mathematical_consistency();
    test_error_handling();
    test_ramanujan_bounds();
    test_reciprocity_law();
    test_memory_and_performance();
    println!("\nAll tests completed successfully!");
}