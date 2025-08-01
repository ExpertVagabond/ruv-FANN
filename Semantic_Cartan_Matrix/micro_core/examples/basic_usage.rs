//! Basic usage example of the micro_core crate

use micro_core::{
    RuvFannBridge, RootVector, RootSpace,
    project_to_root, embed_from_root,
    MicroNet, AgentType,
};

fn main() {
    println!("Micro Core - Semantic Cartan Matrix Example\n");
    
    // Initialize the rUv-FANN bridge
    let mut bridge = RuvFannBridge::new();
    println!("Initialized bridge with {} root dimensions", 32);
    
    // Simulate some token embeddings (normally from a neural network)
    let token_dim = 768; // Common transformer dimension
    let tokens = vec![
        vec![0.1; token_dim],  // Token 1
        vec![0.2; token_dim],  // Token 2
        vec![0.3; token_dim],  // Token 3
    ];
    
    // Project tokens to root space
    println!("\nProjecting {} tokens to root space:", tokens.len());
    let root_vectors: Vec<RootVector> = tokens.iter()
        .map(|token| project_to_root(token, &bridge.root_space))
        .collect();
    
    for (i, root_vec) in root_vectors.iter().enumerate() {
        println!("  Token {} -> Root magnitude: {:.4}", i, root_vec.magnitude());
    }
    
    // Create agents for processing
    println!("\nCreating specialized agents:");
    let mut routing_agent = bridge.create_routing_agent(1);
    let mut reasoning_agent = bridge.create_reasoning_agent(2);
    
    println!("  Agent 1: {} (rank-1: {})", 
        routing_agent.agent_type(), 
        routing_agent.is_routing_head()
    );
    println!("  Agent 2: {} (rank-1: {})", 
        reasoning_agent.agent_type(), 
        reasoning_agent.is_routing_head()
    );
    
    // Process through agents
    println!("\nProcessing through agents:");
    for (i, root_vec) in root_vectors.iter().enumerate() {
        let routing_output = routing_agent.forward(root_vec);
        let reasoning_output = reasoning_agent.forward(root_vec);
        
        println!("  Token {}:", i);
        println!("    Routing output magnitude: {:.4}", routing_output.magnitude());
        println!("    Reasoning output magnitude: {:.4}", reasoning_output.magnitude());
    }
    
    // Check agent compatibility
    let compatibility = routing_agent.compatibility(reasoning_agent.as_ref());
    println!("\nAgent compatibility: {:.2}%", compatibility * 100.0);
    
    // Simulate training epochs with regularization
    println!("\nSimulating training with Cartan regularization:");
    for epoch in 0..10 {
        bridge.step_regularization(epoch);
        let metrics = bridge.export_metrics();
        
        println!("  Epoch {}: λ={:.4}, Cartan loss={:.4}, Orthogonality={:.6}", 
            epoch, 
            metrics.lambda, 
            metrics.cartan_loss,
            metrics.basis_orthogonality
        );
    }
    
    // Demonstrate embedding back to high dimensions
    println!("\nEmbedding back to token space:");
    let reconstructed = embed_from_root(&root_vectors[0], &bridge.root_space, token_dim);
    let reconstruction_error: f32 = tokens[0].iter()
        .zip(reconstructed.iter())
        .map(|(a, b)| (a - b).powi(2))
        .sum::<f32>()
        .sqrt();
    
    println!("  Reconstruction error: {:.6}", reconstruction_error);
    
    // Export final metrics
    let final_metrics = bridge.export_metrics();
    println!("\nFinal metrics:");
    println!("  Cartan loss: {:.4}", final_metrics.cartan_loss);
    println!("  Lambda: {:.4}", final_metrics.lambda);
    println!("  Basis orthogonality: {:.6}", final_metrics.basis_orthogonality);
    println!("  Root magnitudes: {:?}", 
        &final_metrics.root_magnitudes[0..5] // Show first 5
    );
}