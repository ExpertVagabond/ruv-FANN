// WASM bindings for the Neuro-Synaptic Simulator
// Provides JavaScript-compatible interfaces with SIMD optimizations

#![cfg(target_arch = "wasm32")]

use wasm_bindgen::prelude::*;
use serde::{Serialize, Deserialize};
use js_sys::{Array, Uint8Array, Float32Array};
use web_sys::console;

pub mod simd_ops;
pub mod memory_bridge;
pub mod neural_ops;
pub mod performance;

// Re-export main types
pub use simd_ops::{SimdProcessor, SimdBatch};
pub use memory_bridge::{WasmMemoryPool, SharedBuffer};
pub use neural_ops::{NeuralCore, SpikeEvent};
pub use performance::{PerformanceMonitor, Metrics};

/// Main WASM interface for the Neuro-Synaptic Simulator
#[wasm_bindgen]
pub struct WasmSimulator {
    cores: Vec<NeuralCore>,
    memory_pool: WasmMemoryPool,
    simd_processor: SimdProcessor,
    performance_monitor: PerformanceMonitor,
    config: SimulatorConfig,
}

#[wasm_bindgen]
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct SimulatorConfig {
    pub num_cores: u32,
    pub memory_mb: u32,
    pub enable_simd: bool,
    pub timestep_us: f32,
    pub spike_threshold: f32,
}

#[wasm_bindgen]
impl SimulatorConfig {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            num_cores: 256,
            memory_mb: 28,
            enable_simd: true,
            timestep_us: 1.0,
            spike_threshold: 1.0,
        }
    }

    #[wasm_bindgen(getter)]
    pub fn num_cores(&self) -> u32 {
        self.num_cores
    }

    #[wasm_bindgen(setter)]
    pub fn set_num_cores(&mut self, value: u32) {
        self.num_cores = value.min(256);
    }

    #[wasm_bindgen(getter)]
    pub fn memory_mb(&self) -> u32 {
        self.memory_mb
    }

    #[wasm_bindgen(setter)]
    pub fn set_memory_mb(&mut self, value: u32) {
        self.memory_mb = value.min(28);
    }
}

#[wasm_bindgen]
impl WasmSimulator {
    /// Create a new simulator instance
    #[wasm_bindgen(constructor)]
    pub fn new(config: SimulatorConfig) -> Result<WasmSimulator, JsValue> {
        console::log_1(&"Initializing WASM Neuro-Synaptic Simulator".into());
        
        // Initialize memory pool
        let memory_pool = WasmMemoryPool::new(config.memory_mb as usize * 1024 * 1024)
            .map_err(|e| JsValue::from_str(&format!("Memory init failed: {}", e)))?;
        
        // Initialize SIMD processor if supported
        let simd_processor = if config.enable_simd && SimdProcessor::is_supported() {
            console::log_1(&"SIMD support detected, enabling optimizations".into());
            SimdProcessor::new()
        } else {
            console::log_1(&"SIMD not available, using scalar operations".into());
            SimdProcessor::new_scalar()
        };
        
        // Initialize neural cores
        let mut cores = Vec::with_capacity(config.num_cores as usize);
        for i in 0..config.num_cores {
            cores.push(NeuralCore::new(i, config.spike_threshold));
        }
        
        let performance_monitor = PerformanceMonitor::new();
        
        Ok(WasmSimulator {
            cores,
            memory_pool,
            simd_processor,
            performance_monitor,
            config,
        })
    }

    /// Process a single timestep across all cores
    #[wasm_bindgen]
    pub fn step(&mut self) -> Result<(), JsValue> {
        self.performance_monitor.start_frame();
        
        // Process spikes in parallel using SIMD when available
        if self.config.enable_simd {
            self.process_spikes_simd()?;
        } else {
            self.process_spikes_scalar()?;
        }
        
        // Update weights
        self.update_weights()?;
        
        self.performance_monitor.end_frame();
        Ok(())
    }

    /// Process multiple timesteps
    #[wasm_bindgen]
    pub fn run(&mut self, timesteps: u32) -> Result<JsValue, JsValue> {
        console::log_1(&format!("Running {} timesteps", timesteps).into());
        
        for t in 0..timesteps {
            self.step()?;
            
            // Report progress every 1000 steps
            if t % 1000 == 0 && t > 0 {
                let progress = (t as f64 / timesteps as f64) * 100.0;
                console::log_1(&format!("Progress: {:.1}%", progress).into());
            }
        }
        
        // Return performance metrics
        let metrics = self.performance_monitor.get_metrics();
        Ok(serde_wasm_bindgen::to_value(&metrics)?)
    }

    /// Load neural network weights from typed array
    #[wasm_bindgen]
    pub fn load_weights(&mut self, weights: Float32Array) -> Result<(), JsValue> {
        let weights_vec: Vec<f32> = weights.to_vec();
        console::log_1(&format!("Loading {} weights", weights_vec.len()).into());
        
        // Distribute weights across cores
        let weights_per_core = weights_vec.len() / self.cores.len();
        for (i, core) in self.cores.iter_mut().enumerate() {
            let start = i * weights_per_core;
            let end = ((i + 1) * weights_per_core).min(weights_vec.len());
            core.load_weights(&weights_vec[start..end]);
        }
        
        Ok(())
    }

    /// Get output spikes from all cores
    #[wasm_bindgen]
    pub fn get_outputs(&self) -> Result<Float32Array, JsValue> {
        let mut outputs = Vec::with_capacity(self.cores.len());
        
        for core in &self.cores {
            outputs.push(core.get_output());
        }
        
        Ok(Float32Array::from(&outputs[..]))
    }

    /// Reset all cores to initial state
    #[wasm_bindgen]
    pub fn reset(&mut self) {
        console::log_1(&"Resetting simulator".into());
        
        for core in &mut self.cores {
            core.reset();
        }
        
        self.performance_monitor.reset();
    }

    /// Get current performance metrics
    #[wasm_bindgen]
    pub fn get_metrics(&self) -> Result<JsValue, JsValue> {
        let metrics = self.performance_monitor.get_metrics();
        Ok(serde_wasm_bindgen::to_value(&metrics)?)
    }

    /// Check if SIMD is supported and enabled
    #[wasm_bindgen]
    pub fn is_simd_enabled(&self) -> bool {
        self.config.enable_simd && SimdProcessor::is_supported()
    }

    /// Get memory usage statistics
    #[wasm_bindgen]
    pub fn get_memory_stats(&self) -> Result<JsValue, JsValue> {
        let stats = self.memory_pool.get_stats();
        Ok(serde_wasm_bindgen::to_value(&stats)?)
    }
}

// Private implementation methods
impl WasmSimulator {
    fn process_spikes_simd(&mut self) -> Result<(), JsValue> {
        // Batch process spikes using SIMD operations
        let batch_size = 16; // Process 16 neurons at once with SIMD
        
        for chunk in self.cores.chunks_mut(batch_size) {
            let spike_batch = self.simd_processor.process_spike_batch(chunk)?;
            
            // Apply results back to cores
            for (core, spike) in chunk.iter_mut().zip(spike_batch.iter()) {
                if *spike {
                    core.fire_spike();
                }
            }
        }
        
        Ok(())
    }

    fn process_spikes_scalar(&mut self) -> Result<(), JsValue> {
        // Process spikes one by one (fallback for non-SIMD)
        for core in &mut self.cores {
            core.process_timestep();
        }
        
        Ok(())
    }

    fn update_weights(&mut self) -> Result<(), JsValue> {
        // Update synaptic weights based on spike timing
        for core in &mut self.cores {
            core.update_weights();
        }
        
        Ok(())
    }
}

// Utility functions exposed to JavaScript
#[wasm_bindgen]
pub fn version() -> String {
    "0.1.0-wasm".to_string()
}

#[wasm_bindgen]
pub fn check_simd_support() -> bool {
    SimdProcessor::is_supported()
}

#[wasm_bindgen]
pub fn get_max_cores() -> u32 {
    256
}

#[wasm_bindgen]
pub fn get_max_memory_mb() -> u32 {
    28
}

// Initialize WASM module
#[wasm_bindgen(start)]
pub fn init() {
    // Set panic hook for better error messages
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
    
    console::log_1(&"WASM Neuro-Synaptic Simulator loaded successfully".into());
    console::log_1(&format!("SIMD support: {}", check_simd_support()).into());
}