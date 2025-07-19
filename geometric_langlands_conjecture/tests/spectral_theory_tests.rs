//! Integration tests for spectral theory components

use geometric_langlands::spectral::trace_formula::*;
use geometric_langlands::spectral::eisenstein::*;
use geometric_langlands::spectral::hecke::*;
use geometric_langlands::spectral::fourier::{GroupType, GroupFourierTransform};
use nalgebra::DMatrix;
use num_complex::Complex64;
use std::collections::HashMap;

#[test]
fn test_arthur_selberg_trace_formula() {
    let test_function = TestFunction {
        function_type: TestFunctionType::Smooth,
        support: 1.0,
        fourier_transform: None,
    };
    
    let parameters = TraceParameters {
        group: "SL2".to_string(),
        level: 1,
        weight: 2,
        central_character: "trivial".to_string(),
    };

    let mut trace_formula = TraceFormula::new(test_function, parameters);
    
    // Test computation of spectral side
    let spectral_result = trace_formula.compute_spectral_side();
    assert!(spectral_result.is_ok());
    
    // Test computation of geometric side
    let geometric_result = trace_formula.compute_geometric_side();
    assert!(geometric_result.is_ok());
}

#[test]
fn test_selberg_trace_formula() {
    let mut selberg = SelbergTraceFormula::new(2.0 * std::f64::consts::PI);
    
    // Add some eigenvalues corresponding to cusp forms
    selberg.add_eigenvalue(0.25);  // λ = 1/4 corresponds to s = 1/2 ± i/2
    selberg.add_eigenvalue(1.0);   // λ = 1 corresponds to s = 1/2 ± i
    selberg.add_eigenvalue(4.0);   // λ = 4 corresponds to s = 1/2 ± 2i
    
    let trace = selberg.compute_trace();
    assert!(trace > 0.0);
}

#[test]
fn test_eisenstein_series() {
    let parabolic = ParabolicSubgroup {
        parabolic_type: ParabolicType::Minimal,
        levi: LeviComponent {
            rank: 1,
            simple_roots: vec![1],
            central_character: CentralCharacter {
                values: HashMap::new(),
                conductor: 1,
            },
        },
        unipotent_dim: 1,
    };

    let cusp_data = CuspData {
        levi_cusp_form: "trivial".to_string(),
        induced_data: InducedData {
            character: Complex64::new(1.0, 0.0),
            normalization: 1.0,
        },
    };

    let eisenstein = EisensteinSeries::new(
        parabolic,
        Complex64::new(0.5, 0.0),
        cusp_data,
    );

    // Test scattering matrix computation
    let scattering = eisenstein.scattering_matrix();
    assert!(scattering.norm() > 0.0);

    // Test constant term computation
    let constant_term = eisenstein.constant_term(2.0);
    assert!(constant_term.norm() > 0.0);

    // Test residue computation
    let residues = eisenstein.compute_residues();
    assert!(!residues.is_empty());
}

#[test]
fn test_gl_n_eisenstein_series() {
    let eisenstein = GLnEisensteinSeries::new(3, vec![2, 1]).unwrap();
    assert_eq!(eisenstein.rank, 3);
    assert_eq!(eisenstein.partition, vec![2, 1]);
    
    // Test that invalid partitions are rejected
    let invalid_eisenstein = GLnEisensteinSeries::new(3, vec![2, 2]);
    assert!(invalid_eisenstein.is_err());
}

#[test]
fn test_spectral_decomposition() {
    let mut decomp = SpectralDecomposition::new();
    
    // Add discrete spectrum
    decomp.add_discrete(Complex64::new(0.25, 0.0), 1, "cusp1".to_string());
    decomp.add_discrete(Complex64::new(1.0, 0.0), 2, "cusp2".to_string());
    
    assert_eq!(decomp.discrete_part.len(), 2);
    assert_eq!(decomp.spectral_measure(), 3.0);
    
    // Compute residual spectrum
    decomp.compute_residual_spectrum();
}

#[test]
fn test_hecke_operators() {
    let mut t2 = HeckeOperator::new(2, 1, 12);
    
    // Test matrix computation
    let result = t2.compute_matrix(3);
    assert!(result.is_ok());
    
    // Test application to Fourier coefficients
    let coeffs = vec![
        Complex64::new(1.0, 0.0),   // a_0
        Complex64::new(1.0, 0.0),   // a_1  
        Complex64::new(-24.0, 0.0), // a_2 (Ramanujan tau)
        Complex64::new(252.0, 0.0), // a_3
    ];
    
    let applied = t2.apply(&coeffs).unwrap();
    assert_eq!(applied.len(), coeffs.len());
}

#[test]
fn test_hecke_algebra() {
    let mut algebra = HeckeAlgebra::new(1, 12);
    
    // Add some Hecke operators
    algebra.add_operator(2);
    algebra.add_operator(3);
    algebra.add_operator(5);
    
    // Compute commutation relations
    algebra.compute_relations();
    
    // T_2 and T_3 should commute (gcd(2,3) = 1)
    let relation_2_3 = algebra.relations.iter()
        .find(|r| (r.op1 == 2 && r.op2 == 3) || (r.op1 == 3 && r.op2 == 2));
    assert!(relation_2_3.unwrap().commute);
}

#[test] 
fn test_hecke_eigenform() {
    let mut eigenform = HeckeEigenform::new(1, 12);
    
    // Set Fourier coefficients for Ramanujan's Δ function
    eigenform.set_coefficient(1, Complex64::new(1.0, 0.0));
    eigenform.set_coefficient(2, Complex64::new(-24.0, 0.0));
    eigenform.set_coefficient(3, Complex64::new(252.0, 0.0));
    eigenform.set_coefficient(4, Complex64::new(-1472.0, 0.0));
    eigenform.set_coefficient(5, Complex64::new(4830.0, 0.0));
    eigenform.set_coefficient(6, Complex64::new(-6048.0, 0.0));
    
    // Test Hecke eigenvalue computation
    let eigenvalue_2 = eigenform.compute_hecke_eigenvalue(2).unwrap();
    assert_eq!(eigenvalue_2, Complex64::new(-24.0, 0.0));
    
    // Test multiplicativity (should be satisfied for Δ)
    assert!(eigenform.verify_hecke_relations());
    
    // Test L-function evaluation
    let l_value = eigenform.compute_l_value(Complex64::new(2.0, 0.0));
    assert!(l_value.norm() > 0.0);
}

#[test]
fn test_petersson_inner_product() {
    let petersson = PeterssonInnerProduct::new(12, 1);
    
    // Create two modular forms (simplified)
    let form1 = vec![
        Complex64::new(0.0, 0.0),   // a_0 = 0 for cusp forms
        Complex64::new(1.0, 0.0),   // a_1 
        Complex64::new(-24.0, 0.0), // a_2
        Complex64::new(252.0, 0.0), // a_3
    ];
    
    let form2 = vec![
        Complex64::new(0.0, 0.0),
        Complex64::new(1.0, 0.0),
        Complex64::new(24.0, 0.0),  // Different sign
        Complex64::new(-252.0, 0.0),
    ];
    
    // Compute inner product
    let inner_product = petersson.inner_product(&form1, &form2);
    assert!(inner_product.norm() > 0.0);
    
    // Test orthogonality (forms should not be orthogonal due to same coefficients)
    let are_orthogonal = petersson.are_orthogonal(&form1, &form2);
    assert!(!are_orthogonal);
}

#[test]
fn test_fourier_analysis_on_groups() {
    // Test SL(2) group Fourier transform
    let sl2_transform = GroupFourierTransform::new(GroupType::SL { rank: 2 });
    assert!(sl2_transform.haar_measure.unimodular);
    
    // Test GL(3) group Fourier transform  
    let gl3_transform = GroupFourierTransform::new(GroupType::GL { rank: 3 });
    assert!(!gl3_transform.haar_measure.unimodular); // GL(n) is not unimodular
}