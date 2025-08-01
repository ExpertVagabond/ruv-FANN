// Performance monitoring for WASM
// Tracks execution metrics and provides optimization insights

use wasm_bindgen::prelude::*;
use web_sys::{Performance, Window};
use serde::{Serialize, Deserialize};
use std::collections::VecDeque;

/// Performance monitor for tracking simulation metrics
#[wasm_bindgen]
pub struct PerformanceMonitor {
    start_time: f64,
    frame_times: VecDeque<f64>,
    spike_counts: VecDeque<u32>,
    memory_samples: VecDeque<f32>,
    perf: Performance,
    max_samples: usize,
}

/// Performance metrics
#[wasm_bindgen]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Metrics {
    pub fps: f32,
    pub avg_frame_time_ms: f32,
    pub min_frame_time_ms: f32,
    pub max_frame_time_ms: f32,
    pub total_frames: u32,
    pub spikes_per_second: f32,
    pub memory_usage_mb: f32,
    pub simd_utilization: f32,
}

#[wasm_bindgen]
impl PerformanceMonitor {
    /// Create a new performance monitor
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        let window = web_sys::window().expect("no global window");
        let perf = window.performance().expect("performance should be available");
        
        PerformanceMonitor {
            start_time: perf.now(),
            frame_times: VecDeque::with_capacity(1000),
            spike_counts: VecDeque::with_capacity(1000),
            memory_samples: VecDeque::with_capacity(100),
            perf,
            max_samples: 1000,
        }
    }

    /// Start timing a frame
    pub fn start_frame(&mut self) {
        self.start_time = self.perf.now();
    }

    /// End timing a frame
    pub fn end_frame(&mut self) {
        let frame_time = self.perf.now() - self.start_time;
        
        self.frame_times.push_back(frame_time);
        if self.frame_times.len() > self.max_samples {
            self.frame_times.pop_front();
        }
    }

    /// Record spike count for current frame
    pub fn record_spikes(&mut self, count: u32) {
        self.spike_counts.push_back(count);
        if self.spike_counts.len() > self.max_samples {
            self.spike_counts.pop_front();
        }
    }

    /// Sample memory usage
    pub fn sample_memory(&mut self) {
        // In WASM, we estimate memory usage based on heap size
        let memory = wasm_bindgen::memory();
        let pages = memory.grow(0); // Get current page count without growing
        let memory_mb = (pages * 65536) as f32 / (1024.0 * 1024.0);
        
        self.memory_samples.push_back(memory_mb);
        if self.memory_samples.len() > 100 {
            self.memory_samples.pop_front();
        }
    }

    /// Get current metrics
    pub fn get_metrics(&self) -> Metrics {
        let total_frames = self.frame_times.len() as u32;
        
        // Calculate FPS
        let avg_frame_time = if !self.frame_times.is_empty() {
            self.frame_times.iter().sum::<f64>() / self.frame_times.len() as f64
        } else {
            16.67 // Default to 60 FPS
        };
        
        let fps = if avg_frame_time > 0.0 {
            1000.0 / avg_frame_time as f32
        } else {
            60.0
        };
        
        // Frame time statistics
        let min_frame_time = self.frame_times.iter()
            .min_by(|a, b| a.partial_cmp(b).unwrap())
            .copied()
            .unwrap_or(0.0) as f32;
            
        let max_frame_time = self.frame_times.iter()
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .copied()
            .unwrap_or(0.0) as f32;
        
        // Spike statistics
        let total_spikes: u32 = self.spike_counts.iter().sum();
        let spikes_per_frame = if total_frames > 0 {
            total_spikes as f32 / total_frames as f32
        } else {
            0.0
        };
        let spikes_per_second = spikes_per_frame * fps;
        
        // Memory usage
        let memory_usage_mb = self.memory_samples.iter()
            .copied()
            .last()
            .unwrap_or(0.0);
        
        // SIMD utilization (estimate based on performance)
        let expected_frame_time = 16.67; // 60 FPS baseline
        let simd_utilization = if avg_frame_time < expected_frame_time {
            (expected_frame_time / avg_frame_time as f32).min(1.0)
        } else {
            (avg_frame_time as f32 / expected_frame_time).recip().max(0.0)
        };
        
        Metrics {
            fps,
            avg_frame_time_ms: avg_frame_time as f32,
            min_frame_time_ms: min_frame_time,
            max_frame_time_ms: max_frame_time,
            total_frames,
            spikes_per_second,
            memory_usage_mb,
            simd_utilization,
        }
    }

    /// Reset all metrics
    pub fn reset(&mut self) {
        self.frame_times.clear();
        self.spike_counts.clear();
        self.memory_samples.clear();
        self.start_time = self.perf.now();
    }

    /// Get performance report as JSON
    pub fn get_report(&self) -> String {
        let metrics = self.get_metrics();
        serde_json::to_string_pretty(&metrics).unwrap_or_else(|_| "{}".to_string())
    }
}

/// Frame timer for precise measurements
#[wasm_bindgen]
pub struct FrameTimer {
    perf: Performance,
    start: f64,
    marks: Vec<(String, f64)>,
}

#[wasm_bindgen]
impl FrameTimer {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        let window = web_sys::window().expect("no global window");
        let perf = window.performance().expect("performance should be available");
        
        FrameTimer {
            perf: perf.clone(),
            start: perf.now(),
            marks: Vec::new(),
        }
    }

    /// Mark a timing point
    pub fn mark(&mut self, label: &str) {
        let elapsed = self.perf.now() - self.start;
        self.marks.push((label.to_string(), elapsed));
    }

    /// Get all marks as JSON
    pub fn get_marks(&self) -> String {
        let marks: Vec<_> = self.marks.iter()
            .map(|(label, time)| {
                serde_json::json!({
                    "label": label,
                    "time_ms": time
                })
            })
            .collect();
        
        serde_json::to_string(&marks).unwrap_or_else(|_| "[]".to_string())
    }

    /// Reset timer
    pub fn reset(&mut self) {
        self.start = self.perf.now();
        self.marks.clear();
    }
}

/// Performance optimization hints
#[wasm_bindgen]
pub struct OptimizationHints {
    hints: Vec<String>,
}

#[wasm_bindgen]
impl OptimizationHints {
    /// Analyze metrics and provide optimization hints
    pub fn analyze(metrics: &Metrics) -> OptimizationHints {
        let mut hints = Vec::new();
        
        // FPS analysis
        if metrics.fps < 30.0 {
            hints.push("Low FPS detected. Consider reducing simulation complexity.".to_string());
        }
        
        // Frame time variance
        let frame_time_variance = metrics.max_frame_time_ms - metrics.min_frame_time_ms;
        if frame_time_variance > 10.0 {
            hints.push("High frame time variance. Check for periodic heavy operations.".to_string());
        }
        
        // Memory usage
        if metrics.memory_usage_mb > 20.0 {
            hints.push("High memory usage. Consider optimizing data structures.".to_string());
        }
        
        // SIMD utilization
        if metrics.simd_utilization < 0.5 {
            hints.push("Low SIMD utilization. Ensure SIMD is enabled and batch operations.".to_string());
        }
        
        // Spike rate
        if metrics.spikes_per_second > 1_000_000.0 {
            hints.push("Very high spike rate. Consider spike compression or decimation.".to_string());
        }
        
        OptimizationHints { hints }
    }

    /// Get hints as array
    pub fn get_hints(&self) -> Vec<String> {
        self.hints.clone()
    }

    /// Get hints as JSON
    pub fn to_json(&self) -> String {
        serde_json::to_string(&self.hints).unwrap_or_else(|_| "[]".to_string())
    }
}

/// Benchmark utilities
#[wasm_bindgen]
pub fn benchmark_simd_operations(iterations: u32) -> f64 {
    let window = web_sys::window().expect("no global window");
    let perf = window.performance().expect("performance should be available");
    
    let start = perf.now();
    
    // Simulate SIMD operations
    let mut sum = 0.0f32;
    for _ in 0..iterations {
        // This would be replaced with actual SIMD ops
        let a = [1.0f32; 4];
        let b = [2.0f32; 4];
        for i in 0..4 {
            sum += a[i] * b[i];
        }
    }
    
    let elapsed = perf.now() - start;
    
    // Prevent optimization
    if sum > f32::MAX {
        web_sys::console::log_1(&"Overflow prevented".into());
    }
    
    elapsed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_calculation() {
        let monitor = PerformanceMonitor::new();
        let metrics = monitor.get_metrics();
        
        // Default values
        assert_eq!(metrics.total_frames, 0);
        assert!(metrics.fps > 0.0);
    }
}