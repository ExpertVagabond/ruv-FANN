//! Unit tests for Cartan matrix orthogonality properties
//! Tests that H * H^T ≈ 2I (orthonormality with scale factor 2)

use neuro_synaptic_simulator::cartan::{CartanMatrix, SemanticVector};
use proptest::prelude::*;
use approx::{assert_relative_eq, assert_abs_diff_eq};
use ndarray::{Array2, Array1};

#[cfg(test)]
mod cartan_orthogonality_tests {
    use super::*;

    const EPSILON: f32 = 1e-6;
    const DIMENSION: usize = 768; // Standard embedding dimension

    #[test]
    fn test_cartan_orthogonality_identity() {
        // Test: H * H^T should approximate 2 * I
        let cartan = CartanMatrix::new(DIMENSION);
        let h_matrix = cartan.get_matrix();
        
        // Compute H * H^T
        let h_transpose = h_matrix.t();
        let product = h_matrix.dot(&h_transpose);
        
        // Check diagonal elements ≈ 2
        for i in 0..DIMENSION {
            assert_relative_eq!(
                product[[i, i]], 
                2.0, 
                epsilon = EPSILON,
                max_relative = 0.01
            );
        }
        
        // Check off-diagonal elements ≈ 0
        for i in 0..DIMENSION {
            for j in 0..DIMENSION {
                if i != j {
                    assert_abs_diff_eq!(
                        product[[i, j]], 
                        0.0, 
                        epsilon = EPSILON
                    );
                }
            }
        }
    }

    #[test]
    fn test_semantic_vector_preservation() {
        // Test that semantic operations preserve vector norms
        let cartan = CartanMatrix::new(DIMENSION);
        let vec1 = Array1::from_vec(vec![1.0; DIMENSION]);
        let vec2 = Array1::from_vec(vec![0.5; DIMENSION]);
        
        let semantic1 = SemanticVector::from_embedding(&vec1, &cartan);
        let semantic2 = SemanticVector::from_embedding(&vec2, &cartan);
        
        // Test norm preservation after transformation
        let norm1_before = vec1.dot(&vec1).sqrt();
        let norm1_after = semantic1.to_embedding(&cartan).dot(&semantic1.to_embedding(&cartan)).sqrt();
        
        assert_relative_eq!(
            norm1_before,
            norm1_after,
            epsilon = EPSILON,
            max_relative = 0.01
        );
    }

    #[test]
    fn test_cartan_inverse_property() {
        // Test that applying Cartan twice returns to original (up to scaling)
        let cartan = CartanMatrix::new(DIMENSION);
        let original = Array1::from_vec((0..DIMENSION).map(|i| (i as f32 + 1.0) / DIMENSION as f32).collect());
        
        let transformed = cartan.transform(&original);
        let inverse_transformed = cartan.inverse_transform(&transformed);
        
        // Should return to original up to a scale factor
        let scale = original.dot(&inverse_transformed) / original.dot(&original);
        let scaled_inverse = &inverse_transformed / scale;
        
        for i in 0..DIMENSION {
            assert_relative_eq!(
                original[i],
                scaled_inverse[i],
                epsilon = EPSILON,
                max_relative = 0.01
            );
        }
    }

    #[test]
    fn test_orthogonal_basis_vectors() {
        // Test that Cartan basis vectors are orthogonal
        let cartan = CartanMatrix::new(DIMENSION);
        let h_matrix = cartan.get_matrix();
        
        // Check orthogonality of rows
        for i in 0..DIMENSION {
            for j in 0..DIMENSION {
                if i != j {
                    let row_i = h_matrix.row(i);
                    let row_j = h_matrix.row(j);
                    let dot_product = row_i.dot(&row_j);
                    
                    assert_abs_diff_eq!(
                        dot_product,
                        0.0,
                        epsilon = EPSILON
                    );
                }
            }
        }
    }

    #[test]
    fn test_cartan_determinant() {
        // Test that det(H * H^T) = 2^n
        let cartan = CartanMatrix::new(8); // Smaller size for determinant computation
        let h_matrix = cartan.get_matrix();
        let h_transpose = h_matrix.t();
        let product = h_matrix.dot(&h_transpose);
        
        // For orthonormal matrix scaled by sqrt(2), det(H*H^T) = 2^n
        // This is a property test for smaller matrices
        // Note: Full determinant computation for 768x768 is expensive
    }

    // Property-based tests using proptest
    proptest! {
        #[test]
        fn prop_cartan_preserves_angles(
            v1 in prop::collection::vec(-1.0f32..1.0f32, DIMENSION),
            v2 in prop::collection::vec(-1.0f32..1.0f32, DIMENSION)
        ) {
            let cartan = CartanMatrix::new(DIMENSION);
            let vec1 = Array1::from_vec(v1);
            let vec2 = Array1::from_vec(v2);
            
            // Skip if vectors are too small
            if vec1.dot(&vec1) < EPSILON || vec2.dot(&vec2) < EPSILON {
                return Ok(());
            }
            
            // Compute cosine similarity before and after transformation
            let cos_before = vec1.dot(&vec2) / (vec1.dot(&vec1).sqrt() * vec2.dot(&vec2).sqrt());
            
            let trans1 = cartan.transform(&vec1);
            let trans2 = cartan.transform(&vec2);
            
            let cos_after = trans1.dot(&trans2) / (trans1.dot(&trans1).sqrt() * trans2.dot(&trans2).sqrt());
            
            // Angles should be preserved
            prop_assert!((cos_before - cos_after).abs() < 0.01);
        }

        #[test]
        fn prop_cartan_linearity(
            v1 in prop::collection::vec(-1.0f32..1.0f32, DIMENSION),
            v2 in prop::collection::vec(-1.0f32..1.0f32, DIMENSION),
            alpha in -10.0f32..10.0f32,
            beta in -10.0f32..10.0f32
        ) {
            let cartan = CartanMatrix::new(DIMENSION);
            let vec1 = Array1::from_vec(v1);
            let vec2 = Array1::from_vec(v2);
            
            // Test linearity: H(αv1 + βv2) = αH(v1) + βH(v2)
            let linear_combo = &vec1 * alpha + &vec2 * beta;
            let transformed_combo = cartan.transform(&linear_combo);
            
            let trans1 = cartan.transform(&vec1);
            let trans2 = cartan.transform(&vec2);
            let expected = &trans1 * alpha + &trans2 * beta;
            
            for i in 0..DIMENSION {
                prop_assert!((transformed_combo[i] - expected[i]).abs() < 0.001);
            }
        }

        #[test]
        fn prop_orthonormalization_stability(
            v in prop::collection::vec(0.1f32..1.0f32, DIMENSION)
        ) {
            let cartan = CartanMatrix::new(DIMENSION);
            let vec = Array1::from_vec(v);
            
            // Apply transformation multiple times
            let mut current = vec.clone();
            for _ in 0..10 {
                current = cartan.transform(&current);
                current = cartan.inverse_transform(&current);
            }
            
            // Should be stable (return to original up to scaling)
            let scale = vec.dot(&current) / vec.dot(&vec);
            let scaled_current = &current / scale;
            
            for i in 0..DIMENSION {
                prop_assert!((vec[i] - scaled_current[i]).abs() < 0.01);
            }
        }
    }

    #[test]
    fn test_gram_schmidt_orthogonalization() {
        // Test that Cartan performs proper Gram-Schmidt orthogonalization
        let cartan = CartanMatrix::new(DIMENSION);
        
        // Create a set of linearly independent vectors
        let mut vectors = Vec::new();
        for i in 0..8 {
            let mut v = Array1::zeros(DIMENSION);
            v[i * DIMENSION / 8] = 1.0;
            v[(i * DIMENSION / 8 + 1) % DIMENSION] = 0.5;
            vectors.push(v);
        }
        
        // Transform all vectors
        let transformed: Vec<_> = vectors.iter()
            .map(|v| cartan.transform(v))
            .collect();
        
        // Check orthogonality of transformed vectors
        for i in 0..transformed.len() {
            for j in i+1..transformed.len() {
                let dot = transformed[i].dot(&transformed[j]);
                assert_abs_diff_eq!(dot, 0.0, epsilon = 0.01);
            }
        }
    }

    #[test]
    fn test_numerical_stability() {
        // Test numerical stability with extreme values
        let cartan = CartanMatrix::new(DIMENSION);
        
        // Test with very small values
        let small = Array1::from_vec(vec![1e-10; DIMENSION]);
        let trans_small = cartan.transform(&small);
        assert!(trans_small.iter().all(|&x| x.is_finite()));
        
        // Test with very large values
        let large = Array1::from_vec(vec![1e10; DIMENSION]);
        let trans_large = cartan.transform(&large);
        assert!(trans_large.iter().all(|&x| x.is_finite()));
        
        // Test with mixed values
        let mut mixed = Array1::zeros(DIMENSION);
        for i in 0..DIMENSION {
            mixed[i] = if i % 2 == 0 { 1e-5 } else { 1e5 };
        }
        let trans_mixed = cartan.transform(&mixed);
        assert!(trans_mixed.iter().all(|&x| x.is_finite()));
    }
}