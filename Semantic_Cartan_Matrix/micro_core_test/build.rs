//! Build script for micro_core

fn main() {
    // Print feature configuration for debugging
    println!("cargo:warning=Building micro_core with:");
    println!("cargo:warning=  - Target: {}", std::env::var("TARGET").unwrap_or_default());
    println!("cargo:warning=  - Profile: {}", std::env::var("PROFILE").unwrap_or_default());
}