use clap::{Parser, Subcommand};
use anyhow::Result;
use tracing::{info, error};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod core;
mod memory;
mod wasm;
mod timing;
mod logging;
mod performance;
mod visualization;

/// Neuro-Synaptic Chip Simulator
/// 
/// A high-performance simulator for a 256-core neuro-synaptic chip with
/// 28MB shared memory and WASM execution support.
#[derive(Parser)]
#[command(name = "neuro_synaptic_simulator")]
#[command(author = "Neuro-Synaptic Simulator Team")]
#[command(version = "0.1.0")]
#[command(about = "High-performance neuro-synaptic chip simulator", long_about = None)]
struct Cli {
    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,

    /// Number of cores to simulate (max 256)
    #[arg(short, long, default_value_t = 256, value_parser = clap::value_parser!(u16).range(1..=256))]
    cores: u16,

    /// Memory size in MB (max 28)
    #[arg(short, long, default_value_t = 28, value_parser = clap::value_parser!(u16).range(1..=28))]
    memory: u16,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run batch simulations with multiple configurations
    Batch {
        /// Configuration file for batch runs
        #[arg(short, long)]
        config: String,

        /// Number of parallel jobs
        #[arg(short, long, default_value_t = 1)]
        jobs: usize,

        /// Output directory for results
        #[arg(short, long, default_value = "batch_results")]
        output_dir: String,
    },
    /// Run a simulation with specified WASM module
    Run {
        /// Path to WASM module to execute
        #[arg(short, long)]
        wasm_module: String,

        /// Number of timesteps to simulate
        #[arg(short, long, default_value_t = 1000)]
        timesteps: u32,

        /// Output file for JSON logs
        #[arg(short, long)]
        output: Option<String>,

        /// Enable visualization output
        #[arg(long)]
        visualize: bool,
    },

    /// Run test suite
    Test {
        /// Run only specific test category
        #[arg(short, long)]
        filter: Option<String>,

        /// Enable benchmark mode
        #[arg(short, long)]
        bench: bool,
    },

    /// Verify system configuration and dependencies
    Verify {
        /// Check WASM runtime
        #[arg(long)]
        wasm: bool,

        /// Check memory subsystem
        #[arg(long)]
        memory: bool,

        /// Run all verification checks
        #[arg(long)]
        all: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize logging
    let log_level = if cli.verbose { "debug" } else { "info" };
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(log_level))
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("Neuro-Synaptic Simulator v0.1.0");
    info!("Configured with {} cores and {}MB memory", cli.cores, cli.memory);

    match cli.command {
        Commands::Run { wasm_module, timesteps, output, visualize } => {
            info!("Running simulation with WASM module: {}", wasm_module);
            info!("Simulating {} timesteps", timesteps);
            
            run_simulation(cli.cores, cli.memory, &wasm_module, timesteps, output, visualize)?;
        }

        Commands::Batch { config, jobs, output_dir } => {
            info!("Running batch simulations from config: {}", config);
            info!("Using {} parallel jobs", jobs);
            
            run_batch_simulations(&config, jobs, &output_dir)?;
        }

        Commands::Test { filter, bench } => {
            info!("Running tests...");
            if let Some(filter) = filter {
                info!("Filter: {}", filter);
            }
            if bench {
                info!("Benchmark mode enabled");
            }
            
            // TODO: Run test suite
            
            error!("Test command not yet implemented");
        }

        Commands::Verify { wasm, memory, all } => {
            info!("Verifying system configuration...");
            
            if all || wasm {
                info!("Checking WASM runtime...");
                // TODO: Verify WASM runtime
            }
            
            if all || memory {
                info!("Checking memory subsystem...");
                // TODO: Verify memory subsystem
            }
            
            if !wasm && !memory && !all {
                info!("Running basic verification...");
                // TODO: Basic verification
            }
            
            error!("Verify command not yet implemented");
        }
    }

    Ok(())
}

/// Run a single simulation
fn run_simulation(
    cores: u16,
    memory_mb: u16,
    wasm_path: &str,
    timesteps: u32,
    output: Option<String>,
    visualize: bool,
) -> Result<()> {
    use crate::core::{CoreScheduler, SchedulerConfig, DistributionStrategy, SyncMode};
    use crate::memory::{SharedMemory, PartitionStrategy};
    use crate::wasm::{WasmEngine, InstancePool};
    use crate::performance::PerformanceMonitor;
    use crate::visualization::{Visualizer, VisualizationConfig, NetworkVisualization, LayerInfo, PerformanceData};
    use std::sync::Arc;
    use parking_lot::RwLock;

    info!("Initializing simulator with {} cores and {}MB memory", cores, memory_mb);

    // Initialize shared memory
    let memory_size = (memory_mb as usize) * 1024 * 1024;
    let shared_memory = Arc::new(RwLock::new(SharedMemory::new(
        memory_size,
        cores as usize,
        PartitionStrategy::Dynamic,
    )));

    // Initialize scheduler
    let scheduler_config = SchedulerConfig {
        num_cores: cores as usize,
        distribution_strategy: DistributionStrategy::Dynamic,
        sync_mode: SyncMode::Barrier,
        max_batch_size: 64,
    };
    let scheduler = CoreScheduler::new(scheduler_config);

    // Initialize WASM engine
    let wasm_engine = WasmEngine::new()?;
    let instance_pool = Arc::new(RwLock::new(InstancePool::new(cores as usize)));

    // Initialize performance monitor
    let performance_monitor = Arc::new(PerformanceMonitor::new());

    // Load WASM module
    info!("Loading WASM module from: {}", wasm_path);
    let wasm_bytes = std::fs::read(wasm_path)?;
    
    // Create WASM instances for each core
    for core_id in 0..cores {
        let instance = wasm_engine.instantiate(&wasm_bytes, shared_memory.clone())?;
        instance_pool.write().add_instance(instance);
    }

    // Run simulation
    info!("Starting simulation for {} timesteps", timesteps);
    let start_time = std::time::Instant::now();

    // Simulate timesteps
    for timestep in 0..timesteps {
        if timestep % 100 == 0 {
            info!("Timestep {}/{}", timestep, timesteps);
        }

        // Execute parallel computation
        scheduler.execute_timestep(timestep, &instance_pool, &performance_monitor)?;
    }

    let elapsed = start_time.elapsed();
    info!("Simulation completed in {:?}", elapsed);

    // Collect metrics
    let metrics = performance_monitor.get_summary();
    info!("Performance metrics: {:?}", metrics);

    // Save output
    if let Some(output_path) = output {
        let results = serde_json::json!({
            "config": {
                "cores": cores,
                "memory_mb": memory_mb,
                "timesteps": timesteps,
                "wasm_module": wasm_path,
            },
            "performance": metrics,
            "execution_time_seconds": elapsed.as_secs_f64(),
        });

        std::fs::write(&output_path, serde_json::to_string_pretty(&results)?)?;
        info!("Results saved to: {}", output_path);
    }

    // Generate visualization if requested
    if visualize {
        info!("Generating visualizations...");
        let viz_config = VisualizationConfig::default();
        let visualizer = Visualizer::new(viz_config);

        // Create sample visualization data
        let viz_data = NetworkVisualization {
            layers: vec![
                LayerInfo { id: 0, neurons: 256, layer_type: "input".to_string() },
                LayerInfo { id: 1, neurons: 128, layer_type: "hidden".to_string() },
                LayerInfo { id: 2, neurons: 64, layer_type: "output".to_string() },
            ],
            connections: vec![],
            activations: vec![],
            metrics: PerformanceData {
                execution_time_ms: elapsed.as_millis() as f64,
                memory_usage_mb: memory_mb as f64,
                operations_per_second: (timesteps as f64 * cores as f64) / elapsed.as_secs_f64(),
            },
        };

        visualizer.visualize(&viz_data)?;
        info!("Visualizations saved to: ./viz_output");
    }

    Ok(())
}

/// Run batch simulations from configuration file
fn run_batch_simulations(config_path: &str, jobs: usize, output_dir: &str) -> Result<()> {
    use rayon::prelude::*;
    use serde::Deserialize;
    
    #[derive(Debug, Deserialize)]
    struct BatchConfig {
        simulations: Vec<SimulationConfig>,
    }

    #[derive(Debug, Deserialize)]
    struct SimulationConfig {
        name: String,
        cores: u16,
        memory_mb: u16,
        wasm_module: String,
        timesteps: u32,
    }

    // Load batch configuration
    let config_content = std::fs::read_to_string(config_path)?;
    let batch_config: BatchConfig = serde_json::from_str(&config_content)?;

    info!("Loaded {} simulations from config", batch_config.simulations.len());

    // Create output directory
    std::fs::create_dir_all(output_dir)?;

    // Set up thread pool
    rayon::ThreadPoolBuilder::new()
        .num_threads(jobs)
        .build_global()?;

    // Run simulations in parallel
    let results: Vec<_> = batch_config.simulations
        .par_iter()
        .map(|sim_config| {
            info!("Running simulation: {}", sim_config.name);
            let output_path = format!("{}/{}_results.json", output_dir, sim_config.name);
            
            let result = run_simulation(
                sim_config.cores,
                sim_config.memory_mb,
                &sim_config.wasm_module,
                sim_config.timesteps,
                Some(output_path),
                false, // No visualization for batch runs
            );

            match result {
                Ok(_) => {
                    info!("Simulation {} completed successfully", sim_config.name);
                    Ok(sim_config.name.clone())
                }
                Err(e) => {
                    error!("Simulation {} failed: {}", sim_config.name, e);
                    Err((sim_config.name.clone(), e))
                }
            }
        })
        .collect();

    // Summary
    let successful: Vec<_> = results.iter().filter_map(|r| r.as_ref().ok()).collect();
    let failed: Vec<_> = results.iter().filter_map(|r| r.as_ref().err()).collect();

    info!("Batch simulation complete:");
    info!("  Successful: {}", successful.len());
    info!("  Failed: {}", failed.len());

    if !failed.is_empty() {
        error!("Failed simulations:");
        for (name, err) in failed {
            error!("  {}: {}", name, err);
        }
    }

    Ok(())
}
