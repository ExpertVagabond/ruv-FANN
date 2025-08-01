//! Micro Core - Semantic Cartan Matrix implementation for rUv-FANN
//! 
//! This crate provides a no_std implementation of the Semantic Cartan Matrix
//! architecture, designed for integration with rUv-FANN neural networks.
//! 
//! Key features:
//! - 32-dimensional root vector space with SIMD alignment
//! - MicroNet trait for neural network agents
//! - Projection and embedding functions
//! - Optional std support for development

#![no_std]
#![forbid(unsafe_code)]
#![warn(missing_docs)]

#[cfg(feature = "std")]
extern crate std;

extern crate alloc;
use alloc::vec::Vec;
use core::mem::align_of;

pub mod types;
pub mod micronet;
pub mod projection;
pub mod integration;

// Re-export core types
pub use types::{RootVector, RootSpace, CartanMatrix};
pub use micronet::{MicroNet, AgentState, AgentType};
pub use projection::{project_to_root, embed_from_root};
pub use integration::RuvFannBridge;

/// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Default number of root dimensions (configurable)
pub const DEFAULT_ROOT_DIMS: usize = 32;

/// SIMD alignment requirement
pub const SIMD_ALIGN: usize = 16;

/// Initialize the micro_core system
/// 
/// This function performs one-time initialization of the Cartan matrix
/// and root space structures. Must be called before using other functions.
#[cfg(feature = "std")]
pub fn initialize() -> Result<(), &'static str> {
    // In no_std mode, initialization happens at compile time
    Ok(())
}

/// Check if SIMD is available and properly aligned
pub fn check_simd_support() -> bool {
    // Check for common SIMD features and alignment
    let has_simd = cfg!(any(
        target_feature = "sse2",
        target_feature = "avx",
        target_feature = "avx2",
        target_feature = "simd128"
    ));
    has_simd && align_of::<RootVector>() >= SIMD_ALIGN
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simd_alignment() {
        assert!(align_of::<RootVector>() >= SIMD_ALIGN);
    }

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }
}