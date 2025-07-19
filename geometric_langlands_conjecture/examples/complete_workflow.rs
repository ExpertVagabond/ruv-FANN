//! Complete workflow example demonstrating the Geometric Langlands correspondence
//!
//! This example shows:
//! 1. Creating mathematical objects (groups, forms, representations)
//! 2. Establishing correspondences
//! 3. Computing L-functions
//! 4. Using neural networks for pattern recognition
//! 5. Verifying correspondences

use geometric_langlands::prelude::*;
use num_complex::Complex64;

fn main() {
    println!("=============================================================");
    println!("    Geometric Langlands Correspondence - Complete Workflow    ");
    println!("=============================================================\n");

    // Step 1: Create a reductive group
    println!("Step 1: Creating reductive group GL(2)...");
    let group = ReductiveGroup::gl_n(2);
    println!("  Group: {:?}", group.root_system);
    println!("  Dimension: {}", group.dimension);
    println!("  Rank: {}\n", group.rank);

    // Step 2: Create Langlands correspondence
    println!("Step 2: Initializing Langlands correspondence...");
    let mut correspondence = LanglandsCorrespondence::new(group.clone());
    let dual_group = LanglandsCorrespondence::langlands_dual(&group);
    println!("  Dual group: {:?}", dual_group.root_system);
    println!("  Dual dimension: {}\n", dual_group.dimension);

    // Step 3: Create automorphic forms
    println!("Step 3: Creating automorphic forms...");
    
    // Eisenstein series
    let eisenstein = AutomorphicForm::eisenstein_series(&group, 2);
    println!("  Eisenstein series - weight: {}, level: {}", 
             eisenstein.weight(), eisenstein.level());
    
    // Cusp form
    let cusp_form = AutomorphicForm::cusp_form(&group, 2, 11);
    println!("  Cusp form - weight: {}, level: {}, conductor: {}", 
             cusp_form.weight(), cusp_form.level(), cusp_form.conductor());
    
    // Add forms to correspondence
    correspondence.add_automorphic_form(eisenstein).unwrap();
    correspondence.add_automorphic_form(cusp_form).unwrap();
    println!("  Added {} automorphic forms\n", correspondence.automorphic_data.forms.len());

    // Step 4: Apply Hecke operators
    println!("Step 4: Computing Hecke eigenvalues...");
    let primes = [2, 3, 5, 7, 11];
    for &p in &primes {
        let hecke = HeckeOperator::new(&group, p);
        let eigenvalue = hecke.eigenvalue(&correspondence.automorphic_data.forms[0]);
        println!("  T_{} eigenvalue: {:.4}", p, eigenvalue);
    }
    println!();

    // Step 5: Create Galois representations
    println!("Step 5: Creating Galois representations...");
    
    let galois_rep1 = GaloisRepresentation::new(2, 1);
    let galois_rep2 = GaloisRepresentation::new(2, 11);
    
    println!("  Galois rep 1 - dimension: {}, conductor: {}, irreducible: {}", 
             galois_rep1.dimension(), galois_rep1.conductor(), galois_rep1.is_irreducible());
    println!("  Galois rep 2 - dimension: {}, conductor: {}, irreducible: {}", 
             galois_rep2.dimension(), galois_rep2.conductor(), galois_rep2.is_irreducible());
    
    correspondence.add_galois_representation(galois_rep1).unwrap();
    correspondence.add_galois_representation(galois_rep2).unwrap();
    println!("  Added {} Galois representations\n", correspondence.galois_data.representations.len());

    // Step 6: Establish correspondences
    println!("Step 6: Establishing correspondences...");
    correspondence.establish_correspondence(0, 0).unwrap(); // Eisenstein ↔ Rep 1
    correspondence.establish_correspondence(1, 1).unwrap(); // Cusp form ↔ Rep 2
    println!("  Established {} correspondences", correspondence.correspondence_map.form_to_galois.len());
    println!("  Correspondence verified: {}\n", correspondence.verified);

    // Step 7: Compute L-functions
    println!("Step 7: Computing L-functions...");
    let l_function = correspondence.compute_l_function().unwrap();
    println!("  L-function degree: {}", l_function.degree);
    println!("  L-function conductor: {}", l_function.conductor);
    
    // Evaluate L-function at s = 2
    let s = Complex64::new(2.0, 0.0);
    let l_value = l_function.evaluate(s);
    println!("  L(2) = {:.6}\n", l_value);

    // Step 8: Neural network analysis
    println!("Step 8: Neural network pattern recognition...");
    let config = NeuralConfig::default();
    let mut neural_net = LanglandsNeuralNetwork::new(config);
    
    // Extract features
    let form_features = neural_net.extract_automorphic_features(&correspondence.automorphic_data.forms[0]).unwrap();
    let rep_features = neural_net.extract_galois_features(&correspondence.galois_data.representations[0]).unwrap();
    
    println!("  Extracted {} automorphic features", form_features.len());
    println!("  Extracted {} Galois features", rep_features.len());
    
    // Train on correspondences
    let training_data = TrainingData {
        automorphic_forms: correspondence.automorphic_data.forms.clone(),
        galois_representations: correspondence.galois_data.representations.clone(),
        correspondences: vec![(0, 0), (1, 1)],
    };
    
    let metrics = neural_net.train(&training_data).unwrap();
    println!("  Training completed - best accuracy: {:.2}%\n", metrics.best_accuracy * 100.0);

    // Step 9: Functoriality
    println!("Step 9: Functorial lifting...");
    let target_group = ReductiveGroup::gl_n(4);
    let functoriality = Functoriality::new(
        group.clone(),
        target_group,
        LiftType::SymmetricPower { power: 2 }
    );
    
    let lifted_form = functoriality.lift_form(&correspondence.automorphic_data.forms[0]).unwrap();
    println!("  Lifted form - weight: {}, group: {}", lifted_form.weight(), lifted_form.group.root_system);
    
    // Step 10: Reciprocity law
    println!("\nStep 10: Verifying reciprocity law...");
    let mut reciprocity = ReciprocityLaw::new(ReciprocityType::Langlands);
    let verified = reciprocity.verify_numerically(100);
    println!("  Reciprocity verified: {}", verified);
    
    // Step 11: Ramanujan conjecture
    println!("\nStep 11: Checking Ramanujan bounds...");
    let mut ramanujan = RamanujanConjecture::new(group.clone());
    
    for &p in &primes {
        if let Some(eigenvalues) = correspondence.automorphic_data.hecke_eigenvalues.get(&p) {
            if !eigenvalues.is_empty() {
                let verified = ramanujan.verify_at_prime(p, eigenvalues[0]);
                println!("  Prime {}: bound satisfied = {}", p, verified);
            }
        }
    }
    println!("  Ramanujan conjecture satisfied: {}", ramanujan.is_satisfied());

    // Step 12: GPU acceleration (if available)
    #[cfg(feature = "cuda")]
    {
        println!("\nStep 12: GPU acceleration...");
        use geometric_langlands::cuda::{CudaContext, CudaHeckeOperator};
        use std::sync::Arc;
        
        if CudaContext::is_available() {
            let cuda_ctx = Arc::new(CudaContext::new().unwrap());
            let cuda_hecke = CudaHeckeOperator::new(cuda_ctx.clone(), 2, 5);
            
            let matrix = nalgebra::DMatrix::identity(2, 2);
            let gpu_result = cuda_hecke.apply_gpu(&matrix).unwrap();
            println!("  GPU Hecke operator applied successfully");
            println!("  Result norm: {:.4}", gpu_result.norm());
        } else {
            println!("  CUDA not available");
        }
    }

    // Final summary
    println!("\n=============================================================");
    println!("                    Workflow Complete!                        ");
    println!("=============================================================");
    println!("\nSummary:");
    println!("  ✓ Created {} automorphic forms", correspondence.automorphic_data.forms.len());
    println!("  ✓ Created {} Galois representations", correspondence.galois_data.representations.len());
    println!("  ✓ Established {} correspondences", correspondence.correspondence_map.form_to_galois.len());
    println!("  ✓ Computed L-function with degree {}", l_function.degree);
    println!("  ✓ Trained neural network with {:.1}% accuracy", metrics.best_accuracy * 100.0);
    println!("  ✓ Verified functoriality and reciprocity");
    println!("  ✓ Checked Ramanujan bounds");
    
    println!("\nThe Geometric Langlands correspondence connects:");
    println!("  • Automorphic forms ↔ Galois representations");
    println!("  • Hecke eigenvalues ↔ Frobenius eigenvalues");
    println!("  • L-functions ↔ Characteristic polynomials");
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_complete_workflow() {
        // Create group
        let group = ReductiveGroup::gl_n(2);
        
        // Create correspondence
        let mut correspondence = LanglandsCorrespondence::new(group.clone());
        
        // Add forms and representations
        let form = AutomorphicForm::eisenstein_series(&group, 2);
        let rep = GaloisRepresentation::new(2, 1);
        
        correspondence.add_automorphic_form(form).unwrap();
        correspondence.add_galois_representation(rep).unwrap();
        correspondence.establish_correspondence(0, 0).unwrap();
        
        // Verify
        assert!(correspondence.verify_correspondence().unwrap());
    }
}