//! Tests for WebGPU compute backend

#[cfg(test)]
mod webgpu_tests {
    use crate::webgpu::{BackendSelector, ComputeProfile, MatrixSize, OperationType};

    #[test]
    fn test_backend_selector_creation() {
        let selector = BackendSelector::<f32>::new();
        let capabilities = selector.capabilities();

        // Should have at least one backend (CPU fallback)
        assert!(!capabilities.is_empty());

        // All backends should support f32
        for cap in &capabilities {
            assert!(cap.supports_f32);
        }
    }

    #[test]
    fn test_compute_profile_selection() {
        let selector = BackendSelector::<f32>::new();

        let profiles = vec![
            ComputeProfile {
                matrix_size: MatrixSize::Small,
                batch_size: 1,
                operation_type: OperationType::ForwardPass,
            },
            ComputeProfile {
                matrix_size: MatrixSize::Large,
                batch_size: 32,
                operation_type: OperationType::Inference,
            },
        ];

        for profile in profiles {
            let backend = selector.select_backend(&profile);
            assert!(backend.is_some(), "Should always find a backend");
        }
    }
}
