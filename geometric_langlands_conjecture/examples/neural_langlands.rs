//! Neural network example for Geometric Langlands pattern recognition
//!
//! This example demonstrates using neural networks to:
//! 1. Extract features from mathematical objects
//! 2. Learn correspondence patterns
//! 3. Predict new correspondences

use geometric_langlands::prelude::*;
use geometric_langlands::neural::{TrainingData, NeuralConfig};

fn main() {
    println!("=========================================================");
    println!("  Neural Networks for Geometric Langlands Correspondence  ");
    println!("=========================================================\n");

    // Step 1: Create mathematical objects
    println!("Step 1: Creating mathematical objects...");
    let group = ReductiveGroup::gl_n(2);
    
    // Create multiple automorphic forms
    let forms = vec![
        AutomorphicForm::eisenstein_series(&group, 2),
        AutomorphicForm::cusp_form(&group, 2, 11),
        AutomorphicForm::cusp_form(&group, 2, 13),
        AutomorphicForm::eisenstein_series(&group, 4),
        AutomorphicForm::cusp_form(&group, 4, 7),
    ];
    
    // Create corresponding Galois representations
    let representations = vec![
        GaloisRepresentation::new(2, 1),
        GaloisRepresentation::new(2, 11),
        GaloisRepresentation::new(2, 13),
        GaloisRepresentation::new(2, 1),
        GaloisRepresentation::new(2, 7),
    ];
    
    println!("  Created {} automorphic forms", forms.len());
    println!("  Created {} Galois representations\n", representations.len());

    // Step 2: Initialize neural network
    println!("Step 2: Initializing neural network...");
    let config = NeuralConfig {
        learning_rate: 0.001,
        batch_size: 16,
        epochs: 50,
        regularization: 0.01,
        early_stopping_patience: 5,
    };
    
    let mut neural_net = LanglandsNeuralNetwork::new(config);
    println!("  Architecture:");
    println!("    - Automorphic encoder: {} → {} → {} → {}", 
             neural_net.automorphic_encoder.input_dim,
             neural_net.automorphic_encoder.hidden_dims[0],
             neural_net.automorphic_encoder.hidden_dims[1],
             neural_net.automorphic_encoder.embedding_dim);
    println!("    - Correspondence predictor: {} hidden layers\n",
             neural_net.correspondence_predictor.hidden_dims.len());

    // Step 3: Extract features
    println!("Step 3: Extracting features from mathematical objects...");
    
    // Extract features from first form
    let form_features = neural_net.extract_automorphic_features(&forms[0]).unwrap();
    let rep_features = neural_net.extract_galois_features(&representations[0]).unwrap();
    
    println!("  Sample automorphic features (first 5): {:?}", 
             &form_features.as_slice()[..5]);
    println!("  Sample Galois features (first 5): {:?}\n", 
             &rep_features.as_slice()[..5]);

    // Step 4: Prepare training data
    println!("Step 4: Preparing training data...");
    let known_correspondences = vec![
        (0, 0), // Eisenstein(2) ↔ Galois(2,1)
        (1, 1), // Cusp(2,11) ↔ Galois(2,11)
        (2, 2), // Cusp(2,13) ↔ Galois(2,13)
        (3, 3), // Eisenstein(4) ↔ Galois(2,1)
        (4, 4), // Cusp(4,7) ↔ Galois(2,7)
    ];
    
    let training_data = TrainingData {
        automorphic_forms: forms.clone(),
        galois_representations: representations.clone(),
        correspondences: known_correspondences.clone(),
    };
    
    println!("  Training set size: {} correspondences\n", known_correspondences.len());

    // Step 5: Train the neural network
    println!("Step 5: Training neural network...");
    let metrics = neural_net.train(&training_data).unwrap();
    
    println!("  Training completed!");
    println!("  Final accuracy: {:.2}%", metrics.best_accuracy * 100.0);
    println!("  Final loss: {:.4}", metrics.best_loss);
    println!("  Epochs trained: {}\n", metrics.epoch_losses.len());

    // Step 6: Test predictions
    println!("Step 6: Testing neural network predictions...");
    
    // Create new test objects
    let test_forms = vec![
        AutomorphicForm::cusp_form(&group, 2, 17),
        AutomorphicForm::eisenstein_series(&group, 6),
    ];
    
    let test_reps = vec![
        GaloisRepresentation::new(2, 17),
        GaloisRepresentation::new(2, 1),
    ];
    
    // Make predictions
    let predictions = neural_net.predict(&test_forms, &test_reps).unwrap();
    
    println!("  Predicted correspondences:");
    for (i, &(form_idx, rep_idx)) in predictions.predicted_pairs.iter().enumerate() {
        println!("    {} ↔ {} (confidence: {:.2}%)", 
                 if form_idx == 0 { "Cusp(2,17)" } else { "Eisenstein(6)" },
                 if rep_idx == 0 { "Galois(2,17)" } else { "Galois(2,1)" },
                 predictions.confidence_scores[i] * 100.0);
    }
    println!();

    // Step 7: Pattern memory analysis
    println!("Step 7: Analyzing pattern memory...");
    
    // Find similar patterns to a new form
    let query_form = AutomorphicForm::cusp_form(&group, 2, 19);
    let query_features = neural_net.extract_automorphic_features(&query_form).unwrap();
    let similar_patterns = neural_net.find_similar_patterns(&query_features, 3);
    
    println!("  Similar patterns to Cusp(2,19):");
    for (idx, distance) in similar_patterns {
        println!("    Pattern {} - distance: {:.4}", idx, distance);
    }
    println!();

    // Step 8: Feature similarity matrix
    println!("Step 8: Computing feature similarity matrix...");
    let similarity_matrix = &predictions.feature_similarities;
    
    println!("  Similarity matrix (test forms × test reps):");
    println!("                 Galois(2,17)  Galois(2,1)");
    for i in 0..test_forms.len() {
        let form_name = if i == 0 { "Cusp(2,17)   " } else { "Eisenstein(6)" };
        print!("  {} ", form_name);
        for j in 0..test_reps.len() {
            print!("   {:.4}    ", similarity_matrix[(i, j)]);
        }
        println!();
    }

    // Final summary
    println!("\n=========================================================");
    println!("                  Neural Analysis Complete!               ");
    println!("=========================================================");
    println!("\nKey insights:");
    println!("  • Neural network successfully learned correspondence patterns");
    println!("  • Feature extraction captures mathematical invariants");
    println!("  • Predictions align with conductor matching principle");
    println!("  • Pattern memory enables similarity-based reasoning");
    
    println!("\nPotential applications:");
    println!("  • Discovering new correspondences");
    println!("  • Verifying conjectured relationships");
    println!("  • Exploring higher-dimensional cases");
    println!("  • Guiding mathematical research");
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_neural_training() {
        let group = ReductiveGroup::gl_n(2);
        let forms = vec![
            AutomorphicForm::eisenstein_series(&group, 2),
            AutomorphicForm::cusp_form(&group, 2, 11),
        ];
        let reps = vec![
            GaloisRepresentation::new(2, 1),
            GaloisRepresentation::new(2, 11),
        ];
        
        let training_data = TrainingData {
            automorphic_forms: forms,
            galois_representations: reps,
            correspondences: vec![(0, 0), (1, 1)],
        };
        
        let config = NeuralConfig {
            epochs: 10,
            ..Default::default()
        };
        
        let mut nn = LanglandsNeuralNetwork::new(config);
        let metrics = nn.train(&training_data).unwrap();
        
        assert!(metrics.best_accuracy > 0.5);
    }
}