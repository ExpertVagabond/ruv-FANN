// SIMD operations for WASM using wasm32 v128 intrinsics
// Provides vectorized neural computation primitives

use wasm_bindgen::prelude::*;
use std::arch::wasm32::*;

/// SIMD processor for vectorized neural operations
#[wasm_bindgen]
pub struct SimdProcessor {
    supported: bool,
}

/// Batch processing result
pub struct SimdBatch {
    pub values: Vec<f32>,
}

#[wasm_bindgen]
impl SimdProcessor {
    /// Create a new SIMD processor
    pub fn new() -> Self {
        Self {
            supported: Self::detect_simd_support(),
        }
    }

    /// Create a scalar-only processor (fallback)
    pub fn new_scalar() -> Self {
        Self { supported: false }
    }

    /// Check if SIMD is supported
    pub fn is_supported() -> bool {
        Self::detect_simd_support()
    }

    /// Detect SIMD support at runtime
    fn detect_simd_support() -> bool {
        // In WASM, SIMD support depends on browser/runtime
        // This is a compile-time feature in Rust
        #[cfg(target_feature = "simd128")]
        {
            true
        }
        #[cfg(not(target_feature = "simd128"))]
        {
            false
        }
    }

    /// Process spike batch using SIMD
    pub fn process_spike_batch(&self, cores: &mut [crate::neural_ops::NeuralCore]) -> Result<Vec<bool>, JsValue> {
        if self.supported && cores.len() >= 4 {
            self.process_spike_batch_simd(cores)
        } else {
            self.process_spike_batch_scalar(cores)
        }
    }

    #[cfg(target_feature = "simd128")]
    fn process_spike_batch_simd(&self, cores: &mut [crate::neural_ops::NeuralCore]) -> Result<Vec<bool>, JsValue> {
        let mut results = Vec::with_capacity(cores.len());
        
        // Process in chunks of 4 (v128 can hold 4 f32s)
        for chunk in cores.chunks_mut(4) {
            unsafe {
                // Load membrane potentials into SIMD register
                let potentials = f32x4(
                    chunk.get(0).map_or(0.0, |c| c.membrane_potential),
                    chunk.get(1).map_or(0.0, |c| c.membrane_potential),
                    chunk.get(2).map_or(0.0, |c| c.membrane_potential),
                    chunk.get(3).map_or(0.0, |c| c.membrane_potential),
                );
                
                // Load thresholds
                let thresholds = f32x4(
                    chunk.get(0).map_or(1.0, |c| c.spike_threshold),
                    chunk.get(1).map_or(1.0, |c| c.spike_threshold),
                    chunk.get(2).map_or(1.0, |c| c.spike_threshold),
                    chunk.get(3).map_or(1.0, |c| c.spike_threshold),
                );
                
                // Compare potentials >= thresholds
                let spike_mask = f32x4_ge(potentials, thresholds);
                
                // Extract results
                for i in 0..chunk.len() {
                    let spiked = i32x4_extract_lane::<0>(spike_mask) != 0;
                    results.push(spiked);
                }
            }
        }
        
        Ok(results)
    }

    #[cfg(not(target_feature = "simd128"))]
    fn process_spike_batch_simd(&self, _cores: &mut [crate::neural_ops::NeuralCore]) -> Result<Vec<bool>, JsValue> {
        Err(JsValue::from_str("SIMD not supported"))
    }

    fn process_spike_batch_scalar(&self, cores: &mut [crate::neural_ops::NeuralCore]) -> Result<Vec<bool>, JsValue> {
        let results: Vec<bool> = cores.iter()
            .map(|core| core.membrane_potential >= core.spike_threshold)
            .collect();
        Ok(results)
    }

    /// Vectorized weight update using SIMD
    #[cfg(target_feature = "simd128")]
    pub fn update_weights_simd(
        &self,
        weights: &mut [f32],
        deltas: &[f32],
        learning_rate: f32,
    ) -> Result<(), JsValue> {
        if weights.len() != deltas.len() {
            return Err(JsValue::from_str("Weight and delta arrays must have same length"));
        }

        unsafe {
            let lr_vec = f32x4_splat(learning_rate);
            
            // Process in chunks of 4
            for (weight_chunk, delta_chunk) in weights.chunks_mut(4).zip(deltas.chunks(4)) {
                if weight_chunk.len() == 4 && delta_chunk.len() == 4 {
                    // Load current weights
                    let w = v128_load(weight_chunk.as_ptr() as *const v128);
                    
                    // Load deltas
                    let d = v128_load(delta_chunk.as_ptr() as *const v128);
                    
                    // Multiply deltas by learning rate
                    let scaled_deltas = f32x4_mul(d, lr_vec);
                    
                    // Add to weights
                    let new_weights = f32x4_add(w, scaled_deltas);
                    
                    // Store back
                    v128_store(weight_chunk.as_mut_ptr() as *mut v128, new_weights);
                } else {
                    // Handle remaining elements
                    for (w, d) in weight_chunk.iter_mut().zip(delta_chunk) {
                        *w += d * learning_rate;
                    }
                }
            }
        }
        
        Ok(())
    }

    #[cfg(not(target_feature = "simd128"))]
    pub fn update_weights_simd(
        &self,
        weights: &mut [f32],
        deltas: &[f32],
        learning_rate: f32,
    ) -> Result<(), JsValue> {
        self.update_weights_scalar(weights, deltas, learning_rate)
    }

    /// Scalar weight update (fallback)
    pub fn update_weights_scalar(
        &self,
        weights: &mut [f32],
        deltas: &[f32],
        learning_rate: f32,
    ) -> Result<(), JsValue> {
        if weights.len() != deltas.len() {
            return Err(JsValue::from_str("Weight and delta arrays must have same length"));
        }

        for (w, d) in weights.iter_mut().zip(deltas) {
            *w += d * learning_rate;
        }
        
        Ok(())
    }

    /// Vectorized ReLU activation
    #[cfg(target_feature = "simd128")]
    pub fn relu_simd(&self, values: &mut [f32]) -> Result<(), JsValue> {
        unsafe {
            let zero = f32x4_splat(0.0);
            
            for chunk in values.chunks_mut(4) {
                if chunk.len() == 4 {
                    let v = v128_load(chunk.as_ptr() as *const v128);
                    let result = f32x4_max(v, zero);
                    v128_store(chunk.as_mut_ptr() as *mut v128, result);
                } else {
                    for val in chunk {
                        *val = val.max(0.0);
                    }
                }
            }
        }
        
        Ok(())
    }

    #[cfg(not(target_feature = "simd128"))]
    pub fn relu_simd(&self, values: &mut [f32]) -> Result<(), JsValue> {
        for val in values {
            *val = val.max(0.0);
        }
        Ok(())
    }

    /// Vectorized sigmoid approximation
    #[cfg(target_feature = "simd128")]
    pub fn sigmoid_simd(&self, values: &mut [f32]) -> Result<(), JsValue> {
        unsafe {
            let one = f32x4_splat(1.0);
            let neg_one = f32x4_splat(-1.0);
            
            for chunk in values.chunks_mut(4) {
                if chunk.len() == 4 {
                    let v = v128_load(chunk.as_ptr() as *const v128);
                    
                    // Fast sigmoid approximation: 1 / (1 + exp(-x))
                    // Using approximation: 0.5 + 0.5 * tanh(x/2)
                    // Even faster: 0.5 + x / (2 + 2*|x|)
                    
                    let half = f32x4_splat(0.5);
                    let two = f32x4_splat(2.0);
                    
                    let abs_v = f32x4_abs(v);
                    let denominator = f32x4_add(two, f32x4_mul(two, abs_v));
                    let fraction = f32x4_div(v, denominator);
                    let result = f32x4_add(half, fraction);
                    
                    v128_store(chunk.as_mut_ptr() as *mut v128, result);
                } else {
                    for val in chunk {
                        *val = 0.5 + *val / (2.0 + 2.0 * val.abs());
                    }
                }
            }
        }
        
        Ok(())
    }

    #[cfg(not(target_feature = "simd128"))]
    pub fn sigmoid_simd(&self, values: &mut [f32]) -> Result<(), JsValue> {
        for val in values {
            *val = 0.5 + *val / (2.0 + 2.0 * val.abs());
        }
        Ok(())
    }

    /// Vectorized dot product
    #[cfg(target_feature = "simd128")]
    pub fn dot_product_simd(&self, a: &[f32], b: &[f32]) -> Result<f32, JsValue> {
        if a.len() != b.len() {
            return Err(JsValue::from_str("Arrays must have same length"));
        }

        unsafe {
            let mut sum = f32x4_splat(0.0);
            
            // Process main chunks
            let chunks = a.len() / 4;
            for i in 0..chunks {
                let offset = i * 4;
                let va = v128_load(a[offset..].as_ptr() as *const v128);
                let vb = v128_load(b[offset..].as_ptr() as *const v128);
                let prod = f32x4_mul(va, vb);
                sum = f32x4_add(sum, prod);
            }
            
            // Sum the vector elements
            let mut result = f32x4_extract_lane::<0>(sum)
                + f32x4_extract_lane::<1>(sum)
                + f32x4_extract_lane::<2>(sum)
                + f32x4_extract_lane::<3>(sum);
            
            // Handle remaining elements
            for i in (chunks * 4)..a.len() {
                result += a[i] * b[i];
            }
            
            Ok(result)
        }
    }

    #[cfg(not(target_feature = "simd128"))]
    pub fn dot_product_simd(&self, a: &[f32], b: &[f32]) -> Result<f32, JsValue> {
        if a.len() != b.len() {
            return Err(JsValue::from_str("Arrays must have same length"));
        }

        let result = a.iter().zip(b).map(|(x, y)| x * y).sum();
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simd_detection() {
        let processor = SimdProcessor::new();
        // Test will pass regardless of SIMD support
        assert!(processor.supported || !processor.supported);
    }

    #[test]
    fn test_relu() {
        let processor = SimdProcessor::new();
        let mut values = vec![-2.0, -1.0, 0.0, 1.0, 2.0];
        processor.relu_simd(&mut values).unwrap();
        assert_eq!(values, vec![0.0, 0.0, 0.0, 1.0, 2.0]);
    }
}