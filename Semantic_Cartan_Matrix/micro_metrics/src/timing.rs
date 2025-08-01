//! High-precision timing utilities for performance measurement

use alloc::{vec::Vec, string::String};

#[cfg(feature = "serde")]
use serde::{Serialize, Deserialize};

/// Platform-specific timing implementation
#[cfg(not(target_arch = "wasm32"))]
mod native_timing {
    use std::time::Instant;
    
    pub fn now() -> u64 {
        // Use nanoseconds since epoch for cross-platform consistency
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64
    }
    
    pub fn elapsed_since(start: u64) -> u64 {
        now().saturating_sub(start)
    }
}

#[cfg(target_arch = "wasm32")]
mod wasm_timing {
    #[cfg(feature = "wasm")]
    use web_sys::window;
    
    pub fn now() -> u64 {
        #[cfg(feature = "wasm")]
        {
            window()
                .and_then(|w| w.performance())
                .map(|p| (p.now() * 1_000_000.0) as u64) // Convert ms to ns
                .unwrap_or(0)
        }
        #[cfg(not(feature = "wasm"))]
        {
            0 // Fallback for no-std WASM
        }
    }
    
    pub fn elapsed_since(start: u64) -> u64 {
        now().saturating_sub(start)
    }
}

#[cfg(not(target_arch = "wasm32"))]
use native_timing::*;

#[cfg(target_arch = "wasm32")]
use wasm_timing::*;

/// High-precision timer for measuring execution time
#[derive(Debug, Clone)]
pub struct Timer {
    /// Start time in nanoseconds
    start_time: u64,
    
    /// Label for the timer
    label: String,
}

impl Timer {
    /// Create and start a new timer
    pub fn start(label: String) -> Self {
        Self {
            start_time: now(),
            label,
        }
    }
    
    /// Get elapsed time in nanoseconds
    pub fn elapsed_ns(&self) -> u64 {
        elapsed_since(self.start_time)
    }
    
    /// Get elapsed time in microseconds
    pub fn elapsed_us(&self) -> f64 {
        self.elapsed_ns() as f64 / 1_000.0
    }
    
    /// Get elapsed time in milliseconds
    pub fn elapsed_ms(&self) -> f64 {
        self.elapsed_ns() as f64 / 1_000_000.0
    }
    
    /// Get elapsed time in seconds
    pub fn elapsed_s(&self) -> f64 {
        self.elapsed_ns() as f64 / 1_000_000_000.0
    }
    
    /// Stop the timer and return timing info
    pub fn stop(self) -> TimingInfo {
        let elapsed = self.elapsed_ns();
        TimingInfo {
            label: self.label,
            elapsed_ns: elapsed,
        }
    }
    
    /// Get the timer label
    pub fn label(&self) -> &str {
        &self.label
    }
}

/// Information about a completed timing measurement
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct TimingInfo {
    /// Label for the measurement
    pub label: String,
    
    /// Elapsed time in nanoseconds
    pub elapsed_ns: u64,
}

impl TimingInfo {
    /// Get elapsed time in microseconds
    pub fn elapsed_us(&self) -> f64 {
        self.elapsed_ns as f64 / 1_000.0
    }
    
    /// Get elapsed time in milliseconds
    pub fn elapsed_ms(&self) -> f64 {
        self.elapsed_ns as f64 / 1_000_000.0
    }
    
    /// Get elapsed time in seconds
    pub fn elapsed_s(&self) -> f64 {
        self.elapsed_ns as f64 / 1_000_000_000.0
    }
}

/// Collection of timing measurements
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct TimingReport {
    /// Individual timing measurements
    pub measurements: Vec<TimingInfo>,
    
    /// Total elapsed time
    pub total_ns: u64,
}

impl TimingReport {
    /// Create a new timing report
    pub fn new() -> Self {
        Self {
            measurements: Vec::new(),
            total_ns: 0,
        }
    }
    
    /// Add a timing measurement
    pub fn add_measurement(&mut self, timing: TimingInfo) {
        self.total_ns += timing.elapsed_ns;
        self.measurements.push(timing);
    }
    
    /// Get total elapsed time in milliseconds
    pub fn total_ms(&self) -> f64 {
        self.total_ns as f64 / 1_000_000.0
    }
    
    /// Get average time per measurement
    pub fn average_ms(&self) -> f64 {
        if self.measurements.is_empty() {
            0.0
        } else {
            self.total_ms() / self.measurements.len() as f64
        }
    }
    
    /// Find the slowest measurement
    pub fn slowest(&self) -> Option<&TimingInfo> {
        self.measurements.iter()
            .max_by_key(|t| t.elapsed_ns)
    }
    
    /// Find the fastest measurement
    pub fn fastest(&self) -> Option<&TimingInfo> {
        self.measurements.iter()
            .min_by_key(|t| t.elapsed_ns)
    }
}

impl Default for TimingReport {
    fn default() -> Self {
        Self::new()
    }
}

/// Macro for easy timing of code blocks
#[macro_export]
macro_rules! time_block {
    ($label:expr, $block:block) => {{
        let timer = $crate::timing::Timer::start($label.to_string());
        let result = $block;
        let timing = timer.stop();
        (result, timing)
    }};
}