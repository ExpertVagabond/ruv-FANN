//! Basic example demonstrating the Geometric Langlands correspondence
//!
//! This example shows how to construct automorphic forms and their
//! corresponding Galois representations.

use geometric_langlands::prelude::*;

fn main() {
    println!("Geometric Langlands Correspondence Example");
    println!("=========================================\n");
    
    // Create a reductive group GL(3)
    println!("Creating reductive group GL(3)...");
    let g = ReductiveGroup::gl_n(3);
    println!("  Root system: {}", g.root_system);
    println!("  Dimension: {}", g.dimension);
    println!("  Rank: {}\n", g.rank);
    
    // Create Langlands correspondence
    println!("Setting up Langlands correspondence...");
    let mut correspondence = LanglandsCorrespondence::new(g.clone());
    
    // Construct an Eisenstein series (automorphic form)
    println!("Constructing Eisenstein series...");
    let form = AutomorphicForm::eisenstein_series(&g, 2);
    println!("  Weight: {}", form.weight());
    println!("  Level: {}", form.level());
    println!("  Conductor: {}\n", form.conductor());
    
    // Apply Hecke operator T_5
    println!("Applying Hecke operator T_5...");
    let hecke = HeckeOperator::new(&g, 5);
    let eigenform = hecke.apply(&form);
    let eigenvalue = hecke.eigenvalue(&form);
    println!("  Eigenvalue: {:.4}", eigenvalue);
    println!("  New conductor: {}\n", eigenform.conductor());
    
    // Add to correspondence
    correspondence.add_automorphic_form(form).unwrap();
    
    // Create corresponding Galois representation
    println!("Creating Galois representation...");
    let galois_rep = GaloisRepresentation::new(3, 1);
    println!("  Dimension: {}", galois_rep.dimension());
    println!("  Conductor: {}", galois_rep.conductor());
    println!("  Irreducible: {}\n", galois_rep.is_irreducible());
    
    correspondence.add_galois_representation(galois_rep).unwrap();
    
    // Establish correspondence
    correspondence.establish_correspondence(0, 0).unwrap();
    
    // Compute L-function
    println!("Computing L-function...");
    let l_function = correspondence.compute_l_function().unwrap();
    println!("  Degree: {}", l_function.degree);
    println!("  Conductor: {}", l_function.conductor);
    
    // Evaluate at s = 2
    let s = num_complex::Complex64::new(2.0, 0.0);
    let l_value = l_function.evaluate(s);
    println!("  L(2) = {:.6}\n", l_value);
    
    // Verify correspondence
    let verified = correspondence.verify_correspondence().unwrap();
    println!("Correspondence verified: {}\n", verified);
    
    println!("Example completed successfully!");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_correspondence() {
        let g = ReductiveGroup::gl_n(2);
        let mut correspondence = LanglandsCorrespondence::new(g.clone());
        
        let form = AutomorphicForm::eisenstein_series(&g, 1);
        correspondence.add_automorphic_form(form).unwrap();
        
        let rep = GaloisRepresentation::new(2, 1);
        correspondence.add_galois_representation(rep).unwrap();
        
        correspondence.establish_correspondence(0, 0).unwrap();
        assert!(correspondence.verify_correspondence().unwrap());
    }
}