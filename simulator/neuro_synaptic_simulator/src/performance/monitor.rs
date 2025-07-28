use crate::timing::{TimingModel, PowerModel, EnergyConsumption};
use std::sync::{Arc, Mutex};
use std::time::Instant;
use serde::{Serialize, Deserialize};

/// Performance metrics collected during simulation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// Total simulated time in microseconds
    pub simulated_time_us: f64,
    
    /// Wall clock time for simulation in seconds
    pub wall_time_seconds: f64,
    
    /// Total instructions executed across all cores
    pub total_instructions: u64,
    
    /// Total cycles executed across all cores
    pub total_cycles: u64,
    
    /// Average instructions per cycle
    pub average_ipc: f64,
    
    /// Peak IPC achieved
    pub peak_ipc: f64,
    
    /// Energy consumption statistics
    pub energy_stats: EnergyStatistics,
    
    /// Per-core metrics
    pub core_metrics: Vec<CoreMetrics>,
    
    /// Memory bandwidth utilization
    pub memory_bandwidth_gbps: f64,
    
    /// Cache statistics
    pub cache_stats: CacheStatistics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnergyStatistics {
    pub total_energy_joules: f64,
    pub average_power_watts: f64,
    pub peak_power_watts: f64,
    pub energy_efficiency_gops_per_watt: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoreMetrics {
    pub core_id: usize,
    pub cycles_executed: u64,
    pub instructions_executed: u64,
    pub ipc: f64,
    pub utilization_percent: f64,
    pub power_state_distribution: PowerStateDistribution,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PowerStateDistribution {
    pub off_percent: f64,
    pub deep_sleep_percent: f64,
    pub idle_percent: f64,
    pub active_low_percent: f64,
    pub active_medium_percent: f64,
    pub active_high_percent: f64,
    pub turbo_percent: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStatistics {
    pub total_accesses: u64,
    pub total_hits: u64,
    pub total_misses: u64,
    pub hit_rate: f64,
    pub miss_penalty_cycles: u64,
}

/// Performance monitor for tracking simulation metrics
pub struct PerformanceMonitor {
    timing_model: Arc<TimingModel>,
    power_model: Arc<PowerModel>,
    start_time: Instant,
    peak_ipc: Arc<Mutex<f64>>,
    memory_accesses: Arc<Mutex<u64>>,
}

impl PerformanceMonitor {
    pub fn new(timing_model: Arc<TimingModel>, power_model: Arc<PowerModel>) -> Self {
        Self {
            timing_model,
            power_model,
            start_time: Instant::now(),
            peak_ipc: Arc::new(Mutex::new(0.0)),
            memory_accesses: Arc::new(Mutex::new(0)),
        }
    }
    
    /// Update peak IPC if current is higher
    pub fn update_peak_ipc(&self, current_ipc: f64) {
        let mut peak = self.peak_ipc.lock().unwrap();
        if current_ipc > *peak {
            *peak = current_ipc;
        }
    }
    
    /// Record memory access
    pub fn record_memory_access(&self, bytes: u64) {
        let mut accesses = self.memory_accesses.lock().unwrap();
        *accesses += bytes;
    }
    
    /// Collect all performance metrics
    pub fn collect_metrics(&self) -> PerformanceMetrics {
        let wall_time = self.start_time.elapsed().as_secs_f64();
        let simulated_time_us = self.timing_model.get_total_execution_time_us();
        let current_timestamp_ns = (simulated_time_us * 1000.0) as u64;
        
        // Collect timing metrics
        let total_cycles = self.timing_model.get_total_cycles();
        let average_ipc = self.timing_model.get_average_ipc();
        let peak_ipc = *self.peak_ipc.lock().unwrap();
        
        // Collect per-core metrics
        let mut core_metrics = Vec::new();
        let mut total_instructions = 0u64;
        let mut total_cache_hits = 0u64;
        let mut total_cache_misses = 0u64;
        
        for i in 0..256 { // Assuming 256 cores
            if let Some(counter) = self.timing_model.get_core_counters(i) {
                let cycles = counter.cycles_executed.load(std::sync::atomic::Ordering::Relaxed);
                let instructions = counter.instructions_executed.load(std::sync::atomic::Ordering::Relaxed);
                let cache_hits = counter.cache_hits.load(std::sync::atomic::Ordering::Relaxed);
                let cache_misses = counter.cache_misses.load(std::sync::atomic::Ordering::Relaxed);
                
                total_instructions += instructions;
                total_cache_hits += cache_hits;
                total_cache_misses += cache_misses;
                
                let ipc = if cycles > 0 {
                    (instructions as f64) / (cycles as f64)
                } else {
                    0.0
                };
                
                let utilization = if simulated_time_us > 0.0 && cycles > 0 {
                    let core_time_us = cycles as f64 / self.timing_model.get_clock_freq_ghz();
                    (core_time_us / simulated_time_us) * 100.0
                } else {
                    0.0
                };
                
                core_metrics.push(CoreMetrics {
                    core_id: i,
                    cycles_executed: cycles,
                    instructions_executed: instructions,
                    ipc,
                    utilization_percent: utilization.min(100.0),
                    power_state_distribution: PowerStateDistribution {
                        off_percent: 0.0,
                        deep_sleep_percent: 0.0,
                        idle_percent: 100.0 - utilization.min(100.0),
                        active_low_percent: 0.0,
                        active_medium_percent: 0.0,
                        active_high_percent: utilization.min(100.0),
                        turbo_percent: 0.0,
                    },
                });
            }
        }
        
        // Collect energy metrics
        let energy_consumption = self.power_model.get_energy_stats(current_timestamp_ns);
        let gops = (total_instructions as f64) / 1e9; // Giga-operations
        let energy_efficiency = if energy_consumption.total_joules > 0.0 {
            gops / energy_consumption.total_joules
        } else {
            0.0
        };
        
        let energy_stats = EnergyStatistics {
            total_energy_joules: energy_consumption.total_joules,
            average_power_watts: energy_consumption.average_watts,
            peak_power_watts: energy_consumption.peak_watts,
            energy_efficiency_gops_per_watt: energy_efficiency,
        };
        
        // Calculate memory bandwidth
        let memory_bytes = *self.memory_accesses.lock().unwrap();
        let memory_bandwidth_gbps = if wall_time > 0.0 {
            (memory_bytes as f64) / (wall_time * 1e9)
        } else {
            0.0
        };
        
        // Cache statistics
        let total_cache_accesses = total_cache_hits + total_cache_misses;
        let cache_hit_rate = if total_cache_accesses > 0 {
            (total_cache_hits as f64) / (total_cache_accesses as f64)
        } else {
            0.0
        };
        
        let cache_stats = CacheStatistics {
            total_accesses: total_cache_accesses,
            total_hits: total_cache_hits,
            total_misses: total_cache_misses,
            hit_rate: cache_hit_rate,
            miss_penalty_cycles: total_cache_misses * 3, // Assuming 3 cycle penalty
        };
        
        PerformanceMetrics {
            simulated_time_us,
            wall_time_seconds: wall_time,
            total_instructions,
            total_cycles,
            average_ipc,
            peak_ipc,
            energy_stats,
            core_metrics,
            memory_bandwidth_gbps,
            cache_stats,
        }
    }
    
    /// Generate a performance report
    pub fn generate_report(&self) -> String {
        let metrics = self.collect_metrics();
        
        let mut report = String::new();
        report.push_str("=== Neuro-Synaptic Chip Performance Report ===\n\n");
        
        report.push_str(&format!("Simulation Time: {:.2} μs\n", metrics.simulated_time_us));
        report.push_str(&format!("Wall Clock Time: {:.2} s\n", metrics.wall_time_seconds));
        report.push_str(&format!("Simulation Speed: {:.2}x real-time\n", 
            metrics.simulated_time_us / (metrics.wall_time_seconds * 1e6)));
        
        report.push_str("\n--- Performance Metrics ---\n");
        report.push_str(&format!("Total Instructions: {}\n", metrics.total_instructions));
        report.push_str(&format!("Total Cycles: {}\n", metrics.total_cycles));
        report.push_str(&format!("Average IPC: {:.3}\n", metrics.average_ipc));
        report.push_str(&format!("Peak IPC: {:.3}\n", metrics.peak_ipc));
        
        report.push_str("\n--- Energy Metrics ---\n");
        report.push_str(&format!("Total Energy: {:.6} J\n", metrics.energy_stats.total_energy_joules));
        report.push_str(&format!("Average Power: {:.3} W\n", metrics.energy_stats.average_power_watts));
        report.push_str(&format!("Peak Power: {:.3} W\n", metrics.energy_stats.peak_power_watts));
        report.push_str(&format!("Energy Efficiency: {:.2} GOPS/W\n", 
            metrics.energy_stats.energy_efficiency_gops_per_watt));
        
        report.push_str("\n--- Memory & Cache ---\n");
        report.push_str(&format!("Memory Bandwidth: {:.2} GB/s\n", metrics.memory_bandwidth_gbps));
        report.push_str(&format!("Cache Hit Rate: {:.1}%\n", metrics.cache_stats.hit_rate * 100.0));
        report.push_str(&format!("Cache Misses: {}\n", metrics.cache_stats.total_misses));
        
        report.push_str("\n--- Core Utilization ---\n");
        let active_cores = metrics.core_metrics.iter()
            .filter(|c| c.utilization_percent > 0.0)
            .count();
        let avg_utilization = metrics.core_metrics.iter()
            .map(|c| c.utilization_percent)
            .sum::<f64>() / metrics.core_metrics.len() as f64;
        
        report.push_str(&format!("Active Cores: {}/256\n", active_cores));
        report.push_str(&format!("Average Utilization: {:.1}%\n", avg_utilization));
        
        report
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::timing::TimingModel;
    use crate::timing::PowerModel;
    
    #[test]
    fn test_performance_monitor() {
        let timing_model = Arc::new(TimingModel::new(4, 1.5));
        let power_model = Arc::new(PowerModel::new(4, 2.0));
        let monitor = PerformanceMonitor::new(timing_model.clone(), power_model.clone());
        
        // Simulate some activity
        timing_model.update_core_counters(0, 1000, 500, 100, 80, 20, 0);
        monitor.record_memory_access(1024 * 1024); // 1MB
        monitor.update_peak_ipc(0.75);
        
        let metrics = monitor.collect_metrics();
        
        assert_eq!(metrics.total_instructions, 500);
        assert_eq!(metrics.total_cycles, 1000);
        assert_eq!(metrics.peak_ipc, 0.75);
        assert!(metrics.memory_bandwidth_gbps > 0.0);
    }
}