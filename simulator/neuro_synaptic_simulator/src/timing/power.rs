use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Power state of a processing core
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PowerState {
    /// Core is completely powered off
    Off,
    /// Core is in deep sleep, minimal leakage power
    DeepSleep,
    /// Core is idle, ready to execute
    Idle,
    /// Core is actively executing at low frequency
    ActiveLow,
    /// Core is actively executing at medium frequency
    ActiveMedium,
    /// Core is actively executing at high frequency
    ActiveHigh,
    /// Core is in turbo mode (temporary boost)
    Turbo,
}

impl PowerState {
    /// Get power consumption in watts for this state
    /// Based on 12nm ASIC characteristics with 2W total power budget
    pub fn power_watts(&self) -> f64 {
        match self {
            PowerState::Off => 0.0,
            PowerState::DeepSleep => 0.0001,  // 0.1mW leakage
            PowerState::Idle => 0.001,        // 1mW idle
            PowerState::ActiveLow => 0.004,   // 4mW per core low freq
            PowerState::ActiveMedium => 0.006, // 6mW per core medium freq  
            PowerState::ActiveHigh => 0.008,  // 8mW per core high freq
            PowerState::Turbo => 0.010,       // 10mW per core turbo
        }
    }
    
    /// Get the frequency multiplier for this power state
    pub fn frequency_multiplier(&self) -> f64 {
        match self {
            PowerState::Off => 0.0,
            PowerState::DeepSleep => 0.0,
            PowerState::Idle => 0.0,
            PowerState::ActiveLow => 0.5,
            PowerState::ActiveMedium => 0.75,
            PowerState::ActiveHigh => 1.0,
            PowerState::Turbo => 1.2,
        }
    }
}

/// Energy consumption tracking
#[derive(Debug, Clone)]
pub struct EnergyConsumption {
    /// Total energy consumed in joules
    pub total_joules: f64,
    /// Average power in watts
    pub average_watts: f64,
    /// Peak power in watts
    pub peak_watts: f64,
    /// Duration of measurement
    pub duration: Duration,
}

/// Power consumption tracking for a single core
pub struct CorePowerTracking {
    pub core_id: usize,
    pub current_state: AtomicU64, // Encoded PowerState
    pub time_in_state_ns: [AtomicU64; 7], // Nanoseconds in each state
    pub transitions: AtomicU64,
    pub last_transition: AtomicU64, // Timestamp in nanoseconds
}

impl CorePowerTracking {
    fn new(core_id: usize) -> Self {
        Self {
            core_id,
            current_state: AtomicU64::new(PowerState::Idle as u64),
            time_in_state_ns: [
                AtomicU64::new(0),
                AtomicU64::new(0),
                AtomicU64::new(0),
                AtomicU64::new(0),
                AtomicU64::new(0),
                AtomicU64::new(0),
                AtomicU64::new(0),
            ],
            transitions: AtomicU64::new(0),
            last_transition: AtomicU64::new(0),
        }
    }
    
    fn transition_to(&self, new_state: PowerState, timestamp_ns: u64) {
        let old_state = self.current_state.load(Ordering::Relaxed);
        let last_timestamp = self.last_transition.load(Ordering::Relaxed);
        
        if timestamp_ns > last_timestamp {
            let duration = timestamp_ns - last_timestamp;
            self.time_in_state_ns[old_state as usize].fetch_add(duration, Ordering::Relaxed);
        }
        
        self.current_state.store(new_state as u64, Ordering::Relaxed);
        self.last_transition.store(timestamp_ns, Ordering::Relaxed);
        self.transitions.fetch_add(1, Ordering::Relaxed);
    }
    
    fn get_energy_consumed(&self, current_timestamp_ns: u64) -> f64 {
        let mut total_energy = 0.0;
        
        // Add time for all completed states
        for (state_idx, time_ns) in self.time_in_state_ns.iter().enumerate() {
            let ns = time_ns.load(Ordering::Relaxed) as f64;
            let seconds = ns / 1_000_000_000.0;
            let state = match state_idx {
                0 => PowerState::Off,
                1 => PowerState::DeepSleep,
                2 => PowerState::Idle,
                3 => PowerState::ActiveLow,
                4 => PowerState::ActiveMedium,
                5 => PowerState::ActiveHigh,
                6 => PowerState::Turbo,
                _ => PowerState::Idle,
            };
            total_energy += state.power_watts() * seconds;
        }
        
        // Add time in current state
        let current_state_idx = self.current_state.load(Ordering::Relaxed) as usize;
        let last_transition = self.last_transition.load(Ordering::Relaxed);
        if current_timestamp_ns > last_transition {
            let duration_ns = (current_timestamp_ns - last_transition) as f64;
            let seconds = duration_ns / 1_000_000_000.0;
            let current_state = match current_state_idx {
                0 => PowerState::Off,
                1 => PowerState::DeepSleep,
                2 => PowerState::Idle,
                3 => PowerState::ActiveLow,
                4 => PowerState::ActiveMedium,
                5 => PowerState::ActiveHigh,
                6 => PowerState::Turbo,
                _ => PowerState::Idle,
            };
            total_energy += current_state.power_watts() * seconds;
        }
        
        total_energy
    }
}

/// Power model for the neuro-synaptic chip simulator
pub struct PowerModel {
    /// Number of cores
    num_cores: usize,
    
    /// Maximum chip power in watts
    max_chip_power: f64,
    
    /// Power tracking per core
    core_power: Vec<Arc<CorePowerTracking>>,
    
    /// Static/leakage power for the chip
    static_power: f64,
    
    /// Memory power consumption per MB active
    memory_power_per_mb: f64,
    
    /// Start time for energy calculations
    start_time: Instant,
}

impl PowerModel {
    /// Create a new power model
    /// Default: 2W max power, 256 cores, 28MB memory
    pub fn new(num_cores: usize, max_chip_power: f64) -> Self {
        let mut core_power = Vec::with_capacity(num_cores);
        for core_id in 0..num_cores {
            core_power.push(Arc::new(CorePowerTracking::new(core_id)));
        }
        
        Self {
            num_cores,
            max_chip_power,
            core_power,
            static_power: 0.1, // 100mW static power
            memory_power_per_mb: 0.01, // 10mW per MB
            start_time: Instant::now(),
        }
    }
    
    /// Update the power state of a core
    pub fn set_core_state(&self, core_id: usize, state: PowerState, timestamp_ns: u64) {
        if let Some(core) = self.core_power.get(core_id) {
            core.transition_to(state, timestamp_ns);
        }
    }
    
    /// Get current power consumption of the entire chip
    pub fn get_current_power(&self) -> f64 {
        let mut total_power = self.static_power;
        
        // Add power from all cores
        for core in &self.core_power {
            let state_idx = core.current_state.load(Ordering::Relaxed) as usize;
            let state = match state_idx {
                0 => PowerState::Off,
                1 => PowerState::DeepSleep,
                2 => PowerState::Idle,
                3 => PowerState::ActiveLow,
                4 => PowerState::ActiveMedium,
                5 => PowerState::ActiveHigh,
                6 => PowerState::Turbo,
                _ => PowerState::Idle,
            };
            total_power += state.power_watts();
        }
        
        // Add memory power (assume all 28MB active for now)
        total_power += 28.0 * self.memory_power_per_mb;
        
        total_power.min(self.max_chip_power)
    }
    
    /// Check if adding more active cores would exceed power budget
    pub fn can_activate_cores(&self, num_additional: usize, target_state: PowerState) -> bool {
        let current_power = self.get_current_power();
        let additional_power = (num_additional as f64) * target_state.power_watts();
        
        (current_power + additional_power) <= self.max_chip_power
    }
    
    /// Get energy consumption statistics
    pub fn get_energy_stats(&self, current_timestamp_ns: u64) -> EnergyConsumption {
        let mut total_energy = 0.0;
        let duration = self.start_time.elapsed();
        
        // Calculate energy from all cores
        for core in &self.core_power {
            total_energy += core.get_energy_consumed(current_timestamp_ns);
        }
        
        // Add static power contribution
        let total_seconds = duration.as_secs_f64();
        total_energy += self.static_power * total_seconds;
        
        // Add memory power contribution
        total_energy += 28.0 * self.memory_power_per_mb * total_seconds;
        
        let average_power = if total_seconds > 0.0 {
            total_energy / total_seconds
        } else {
            0.0
        };
        
        EnergyConsumption {
            total_joules: total_energy,
            average_watts: average_power,
            peak_watts: self.max_chip_power, // Could track actual peak
            duration,
        }
    }
    
    /// Get the number of cores in each power state
    pub fn get_state_distribution(&self) -> [usize; 7] {
        let mut distribution = [0; 7];
        
        for core in &self.core_power {
            let state_idx = core.current_state.load(Ordering::Relaxed) as usize;
            if state_idx < 7 {
                distribution[state_idx] += 1;
            }
        }
        
        distribution
    }
    
    /// Apply dynamic voltage and frequency scaling (DVFS) based on workload
    pub fn apply_dvfs(&self, target_utilization: f64, timestamp_ns: u64) {
        let active_cores = self.core_power.iter()
            .filter(|c| {
                let state = c.current_state.load(Ordering::Relaxed);
                state >= PowerState::ActiveLow as u64
            })
            .count();
        
        let utilization = (active_cores as f64) / (self.num_cores as f64);
        
        // Simple DVFS policy
        let target_state = if utilization < 0.3 {
            PowerState::ActiveLow
        } else if utilization < 0.7 {
            PowerState::ActiveMedium
        } else if utilization < 0.9 {
            PowerState::ActiveHigh
        } else {
            PowerState::Turbo
        };
        
        // Apply new state to active cores
        for (idx, core) in self.core_power.iter().enumerate() {
            let current = core.current_state.load(Ordering::Relaxed);
            if current >= PowerState::ActiveLow as u64 {
                self.set_core_state(idx, target_state, timestamp_ns);
            }
        }
    }
    
    /// Reset all power tracking
    pub fn reset(&self) {
        for core in &self.core_power {
            core.current_state.store(PowerState::Idle as u64, Ordering::Relaxed);
            core.transitions.store(0, Ordering::Relaxed);
            core.last_transition.store(0, Ordering::Relaxed);
            for time_counter in &core.time_in_state_ns {
                time_counter.store(0, Ordering::Relaxed);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_power_states() {
        assert_eq!(PowerState::Off.power_watts(), 0.0);
        assert_eq!(PowerState::Idle.power_watts(), 0.001);
        assert_eq!(PowerState::ActiveHigh.power_watts(), 0.008);
        assert_eq!(PowerState::Turbo.frequency_multiplier(), 1.2);
    }
    
    #[test]
    fn test_power_model_basic() {
        let model = PowerModel::new(256, 2.0);
        
        // Initially all cores idle
        let initial_power = model.get_current_power();
        // Static (0.1W) + 256 idle cores (0.256W) + 28MB memory (0.28W)
        assert!((initial_power - 0.636).abs() < 0.001);
        
        // Activate some cores
        for i in 0..10 {
            model.set_core_state(i, PowerState::ActiveHigh, 1000);
        }
        
        let active_power = model.get_current_power();
        // Should increase by ~0.07W (10 cores * 0.007W difference)
        assert!((active_power - initial_power - 0.07).abs() < 0.001);
    }
    
    #[test]
    fn test_power_budget_check() {
        let model = PowerModel::new(256, 2.0);
        
        // Check if we can activate 100 cores at high power
        assert!(model.can_activate_cores(100, PowerState::ActiveHigh));
        
        // Check if we can activate 250 cores at turbo (should fail)
        assert!(!model.can_activate_cores(250, PowerState::Turbo));
    }
    
    #[test]
    fn test_energy_tracking() {
        let model = PowerModel::new(4, 2.0);
        
        // Run cores for simulated time
        model.set_core_state(0, PowerState::ActiveHigh, 0);
        model.set_core_state(1, PowerState::ActiveMedium, 0);
        
        // Simulate 1 second (1e9 nanoseconds)
        let stats = model.get_energy_stats(1_000_000_000);
        
        // Energy should be approximately:
        // Core 0: 0.008W * 1s = 0.008J
        // Core 1: 0.006W * 1s = 0.006J
        // Core 2,3: 0.001W * 1s = 0.002J
        // Static: 0.1W * 1s = 0.1J
        // Memory: 0.28W * 1s = 0.28J
        // Total: ~0.396J
        assert!(stats.total_joules > 0.0);
    }
}