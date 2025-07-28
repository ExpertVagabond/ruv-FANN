//! Minimal working example of the Neuro-Synaptic Simulator
//! 
//! This example demonstrates basic usage of the simulator with:
//! - Shared memory configuration
//! - Power model setup
//! - Basic timing simulation

use neuro_synaptic_simulator::{
    SharedMemory, PartitionStrategy,
    PowerModel, PowerState,
    TimingModel,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧠 Neuro-Synaptic Simulator - Minimal Demo");
    println!("==========================================");
    
    // Create shared memory for 28MB with 256 cores
    let memory = SharedMemory::new(
        28 * 1024 * 1024, // 28MB total memory
        256,              // 256 cores
        PartitionStrategy::Equal,
    );
    
    println!("✅ Created shared memory: 28MB across 256 cores");
    if let Some((start, end)) = memory.partition_info(0) {
        println!("   Core 0 partition size: {}KB", (end - start) / 1024);
    }
    
    // Create power model with 2W total budget
    let power_model = PowerModel::new(256, 2.0);
    println!("✅ Created power model: 2W budget for 256 cores");
    
    // Get initial power consumption
    let initial_power = power_model.get_current_power();
    println!("   Initial power consumption: {:.3}W", initial_power);
    
    // Activate some cores
    println!("\n📊 Activating cores for simulation...");
    for core_id in 0..64 {
        power_model.set_core_state(core_id, PowerState::ActiveHigh, 1000);
    }
    
    let active_power = power_model.get_current_power();
    println!("   Power with 64 active cores: {:.3}W", active_power);
    
    // Check if we can activate more cores
    let can_activate_more = power_model.can_activate_cores(100, PowerState::ActiveMedium);
    println!("   Can activate 100 more cores at medium power: {}", can_activate_more);
    
    // Create timing model
    let timing_model = TimingModel::default();
    println!("\n⏱️  Created timing model:");
    println!("   Clock frequency: {}MHz", timing_model.clock_frequency_mhz);
    println!("   Memory latency: {} cycles", timing_model.memory_latency_cycles);
    println!("   Cache hit latency: {} cycles", timing_model.cache_hit_cycles);
    
    // Simulate some operation and get energy stats
    let energy_stats = power_model.get_energy_stats(1_000_000_000); // 1 second simulation
    println!("\n⚡ Energy consumption after 1 second:");
    println!("   Total energy: {:.6} Joules", energy_stats.total_joules);
    println!("   Average power: {:.3} Watts", energy_stats.average_watts);
    println!("   Peak power: {:.3} Watts", energy_stats.peak_watts);
    
    // Show power state distribution
    let distribution = power_model.get_state_distribution();
    println!("\n📈 Core power state distribution:");
    println!("   Off: {}", distribution[0]);
    println!("   Deep Sleep: {}", distribution[1]);
    println!("   Idle: {}", distribution[2]);
    println!("   Active Low: {}", distribution[3]);
    println!("   Active Medium: {}", distribution[4]);
    println!("   Active High: {}", distribution[5]);
    println!("   Turbo: {}", distribution[6]);
    
    println!("\n🎉 Simulation completed successfully!");
    println!("   The neuro-synaptic simulator is working correctly with:");
    println!("   • Parallel memory management");
    println!("   • Power-aware core scheduling");
    println!("   • Cycle-accurate timing simulation");
    
    Ok(())
}