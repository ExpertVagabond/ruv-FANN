// Basic Example - Working Features Only
// This demonstrates the currently implemented features in geometric-langlands-wasm

import init, { 
    ReductiveGroup, 
    AutomorphicForm, 
    GaloisRepresentation,
    LanglandsCorrespondence,
    compute_l_function_value
} from '../pkg/geometric_langlands_wasm.js';

async function main() {
    // Initialize the WASM module
    await init();
    
    console.log("🌟 Geometric Langlands WASM - Basic Example");
    console.log("⚠️  Alpha version - limited features available\n");
    
    // ✅ WORKING: Create reductive groups
    console.log("1. Creating Reductive Groups:");
    const gl3 = ReductiveGroup.gl_n(3);
    console.log(`   GL(3): rank=${gl3.rank}, dimension=${gl3.dimension}`);
    
    const sl2 = ReductiveGroup.sl_n(2);
    console.log(`   SL(2): rank=${sl2.rank}, dimension=${sl2.dimension}`);
    
    const sp4 = ReductiveGroup.sp_2n(2);
    console.log(`   Sp(4): rank=${sp4.rank}, dimension=${sp4.dimension}`);
    
    // ✅ WORKING: Create automorphic forms
    console.log("\n2. Creating Automorphic Forms:");
    const cuspidal = AutomorphicForm.cuspidal_form("example_cusp", 2.5);
    console.log(`   Cuspidal form: weight=${cuspidal.weight}, level=${cuspidal.level}`);
    
    const eisenstein = AutomorphicForm.eisenstein_series(1.0, 3.0);
    console.log(`   Eisenstein series: weight=${eisenstein.weight}`);
    
    // ✅ WORKING: Create Galois representations
    console.log("\n3. Creating Galois Representations:");
    const artin = GaloisRepresentation.artin_representation(2);
    console.log(`   Artin representation: dimension=${artin.dimension}`);
    
    const weil_deligne = GaloisRepresentation.weil_deligne_representation(3, 1.5);
    console.log(`   Weil-Deligne representation: dimension=${weil_deligne.dimension}`);
    
    // ✅ WORKING: Compute L-function values
    console.log("\n4. Computing L-function values:");
    const l_value = compute_l_function_value(2.0);
    console.log(`   L(2.0) = ${l_value}`);
    
    // ✅ WORKING: Basic Langlands correspondence
    console.log("\n5. Langlands Correspondence (simplified):");
    const correspondence = new LanglandsCorrespondence();
    
    try {
        const galois_rep = correspondence.automorphic_to_galois(cuspidal);
        console.log(`   Automorphic → Galois: Success!`);
        console.log(`   Resulting dimension: ${galois_rep.dimension}`);
    } catch (e) {
        console.log(`   Note: Full correspondence not yet implemented`);
    }
    
    // ⚠️ NOT YET IMPLEMENTED features (for clarity):
    console.log("\n⚠️  Features in development:");
    console.log("   - Advanced geometric structures");
    console.log("   - Complete spectral decomposition");
    console.log("   - Full categorical equivalences");
    console.log("   - GPU acceleration");
    console.log("   - Complex visualizations");
    
    console.log("\n✅ Demo completed successfully!");
}

// Run the example
main().catch(console.error);