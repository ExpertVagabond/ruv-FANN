// Neural operations for WASM - Core neural computation primitives
// Implements spiking neural network operations optimized for WASM

use wasm_bindgen::prelude::*;
use serde::{Serialize, Deserialize};

/// Neural core representing a single processing unit
#[wasm_bindgen]
pub struct NeuralCore {
    pub id: u32,
    pub membrane_potential: f32,
    pub spike_threshold: f32,
    weights: Vec<f32>,
    last_spike_time: f32,
    refractory_period: f32,
    current_time: f32,
    spike_count: u32,
}

/// Spike event for inter-core communication
#[wasm_bindgen]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SpikeEvent {
    pub source_core: u32,
    pub target_core: u32,
    pub timestamp: f32,
    pub weight: f32,
}

#[wasm_bindgen]
impl NeuralCore {
    /// Create a new neural core
    #[wasm_bindgen(constructor)]
    pub fn new(id: u32, spike_threshold: f32) -> Self {
        NeuralCore {
            id,
            membrane_potential: 0.0,
            spike_threshold,
            weights: Vec::new(),
            last_spike_time: -1000.0,
            refractory_period: 2.0, // 2ms refractory period
            current_time: 0.0,
            spike_count: 0,
        }
    }

    /// Load synaptic weights
    pub fn load_weights(&mut self, weights: &[f32]) {
        self.weights = weights.to_vec();
    }

    /// Process a single timestep
    pub fn process_timestep(&mut self) {
        // Check if in refractory period
        if self.current_time - self.last_spike_time < self.refractory_period {
            self.membrane_potential = 0.0;
            self.current_time += 1.0;
            return;
        }

        // Leak current (exponential decay)
        self.membrane_potential *= 0.95;

        // Add random input current (simplified)
        let input_current = (self.id as f32 * 0.1).sin() * 0.5 + 0.5;
        self.membrane_potential += input_current;

        // Check for spike
        if self.membrane_potential >= self.spike_threshold {
            self.fire_spike();
        }

        self.current_time += 1.0;
    }

    /// Fire a spike
    pub fn fire_spike(&mut self) {
        self.spike_count += 1;
        self.last_spike_time = self.current_time;
        self.membrane_potential = 0.0; // Reset potential
    }

    /// Update weights based on spike timing (STDP)
    pub fn update_weights(&mut self) {
        // Simplified STDP (Spike-Timing-Dependent Plasticity)
        let time_diff = self.current_time - self.last_spike_time;
        
        if time_diff < 20.0 {
            // Recent spike - potentiate weights
            for w in &mut self.weights {
                *w *= 1.01;
                *w = w.min(10.0); // Cap weights
            }
        } else if time_diff < 50.0 {
            // Older spike - depress weights
            for w in &mut self.weights {
                *w *= 0.99;
                *w = w.max(0.001); // Minimum weight
            }
        }
    }

    /// Get current output value
    pub fn get_output(&self) -> f32 {
        if self.current_time - self.last_spike_time < 1.0 {
            1.0 // Recently spiked
        } else {
            self.membrane_potential / self.spike_threshold
        }
    }

    /// Reset core to initial state
    pub fn reset(&mut self) {
        self.membrane_potential = 0.0;
        self.last_spike_time = -1000.0;
        self.current_time = 0.0;
        self.spike_count = 0;
        // Keep weights
    }

    /// Get spike count
    #[wasm_bindgen(getter)]
    pub fn spike_count(&self) -> u32 {
        self.spike_count
    }

    /// Get current time
    #[wasm_bindgen(getter)]
    pub fn current_time(&self) -> f32 {
        self.current_time
    }

    /// Process incoming spike
    pub fn receive_spike(&mut self, weight: f32) {
        if self.current_time - self.last_spike_time >= self.refractory_period {
            self.membrane_potential += weight;
        }
    }
}

/// Synaptic connection between cores
#[wasm_bindgen]
pub struct Synapse {
    source: u32,
    target: u32,
    weight: f32,
    delay: f32,
    plasticity_enabled: bool,
}

#[wasm_bindgen]
impl Synapse {
    #[wasm_bindgen(constructor)]
    pub fn new(source: u32, target: u32, weight: f32, delay: f32) -> Self {
        Synapse {
            source,
            target,
            weight,
            delay,
            plasticity_enabled: true,
        }
    }

    /// Apply STDP rule
    pub fn apply_stdp(&mut self, pre_spike_time: f32, post_spike_time: f32) {
        if !self.plasticity_enabled {
            return;
        }

        let dt = post_spike_time - pre_spike_time;
        
        // STDP window
        let tau_plus = 20.0;
        let tau_minus = 20.0;
        let a_plus = 0.01;
        let a_minus = 0.01;

        let dw = if dt > 0.0 {
            // Post after pre - potentiation
            a_plus * (-dt / tau_plus).exp()
        } else {
            // Pre after post - depression
            -a_minus * (dt / tau_minus).exp()
        };

        self.weight += dw;
        self.weight = self.weight.clamp(0.001, 10.0);
    }

    #[wasm_bindgen(getter)]
    pub fn weight(&self) -> f32 {
        self.weight
    }

    #[wasm_bindgen(setter)]
    pub fn set_weight(&mut self, value: f32) {
        self.weight = value.clamp(0.001, 10.0);
    }
}

/// Neural network layer
#[wasm_bindgen]
pub struct NeuralLayer {
    cores: Vec<NeuralCore>,
    layer_type: LayerType,
}

#[wasm_bindgen]
#[derive(Clone, Copy)]
pub enum LayerType {
    Input,
    Hidden,
    Output,
}

#[wasm_bindgen]
impl NeuralLayer {
    #[wasm_bindgen(constructor)]
    pub fn new(size: u32, layer_type: LayerType, spike_threshold: f32) -> Self {
        let cores = (0..size)
            .map(|i| NeuralCore::new(i, spike_threshold))
            .collect();

        NeuralLayer { cores, layer_type }
    }

    /// Process all cores in the layer
    pub fn process(&mut self) {
        for core in &mut self.cores {
            core.process_timestep();
        }
    }

    /// Get layer outputs
    pub fn get_outputs(&self) -> Vec<f32> {
        self.cores.iter().map(|c| c.get_output()).collect()
    }

    /// Reset all cores
    pub fn reset(&mut self) {
        for core in &mut self.cores {
            core.reset();
        }
    }

    /// Get spike statistics
    pub fn get_spike_stats(&self) -> SpikeStats {
        let total_spikes: u32 = self.cores.iter().map(|c| c.spike_count).sum();
        let avg_spikes = total_spikes as f32 / self.cores.len() as f32;
        
        SpikeStats {
            total_spikes,
            avg_spikes_per_core: avg_spikes,
            active_cores: self.cores.iter().filter(|c| c.spike_count > 0).count() as u32,
        }
    }
}

/// Spike statistics
#[wasm_bindgen]
#[derive(Serialize, Deserialize)]
pub struct SpikeStats {
    pub total_spikes: u32,
    pub avg_spikes_per_core: f32,
    pub active_cores: u32,
}

/// Create a spike raster plot data
#[wasm_bindgen]
pub fn create_spike_raster(cores: &[NeuralCore], time_window: f32) -> Vec<u32> {
    let mut raster = Vec::new();
    
    for core in cores {
        if core.current_time - core.last_spike_time < time_window {
            raster.push(core.id);
        }
    }
    
    raster
}

/// Calculate network synchrony
#[wasm_bindgen]
pub fn calculate_synchrony(cores: &[NeuralCore], window_ms: f32) -> f32 {
    if cores.is_empty() {
        return 0.0;
    }

    let current_time = cores[0].current_time;
    let active_count = cores.iter()
        .filter(|c| (current_time - c.last_spike_time).abs() < window_ms)
        .count();
    
    active_count as f32 / cores.len() as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_neural_core() {
        let mut core = NeuralCore::new(0, 1.0);
        assert_eq!(core.membrane_potential, 0.0);
        
        // Process several timesteps
        for _ in 0..10 {
            core.process_timestep();
        }
        
        // Should have accumulated some potential
        assert!(core.membrane_potential > 0.0);
    }

    #[test]
    fn test_spike_generation() {
        let mut core = NeuralCore::new(0, 0.5);
        core.membrane_potential = 0.6; // Above threshold
        
        let initial_count = core.spike_count;
        core.process_timestep();
        
        assert_eq!(core.spike_count, initial_count + 1);
        assert_eq!(core.membrane_potential, 0.0); // Reset after spike
    }
}