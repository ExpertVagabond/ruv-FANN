use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::Colorize;
use std::path::PathBuf;

use geometric_langlands_cli::*;
use crate::commands::{compute, export, train, verify, visual};
use crate::config::Config;

/// Geometric Langlands CLI - A user-friendly interface for mathematical computations
#[derive(Parser)]
#[command(name = "langlands")]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    /// Configuration file path
    #[arg(short, long, global = true)]
    config: Option<PathBuf>,

    /// Verbosity level (can be used multiple times)
    #[arg(short, long, action = clap::ArgAction::Count, global = true)]
    verbose: u8,

    /// Output format
    #[arg(short, long, global = true, default_value = "pretty")]
    format: OutputFormat,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Run correspondence checks and computations
    Compute {
        /// Type of computation to perform
        #[arg(short = 't', long)]
        computation_type: String,

        /// Input file or expression
        #[arg(short, long)]
        input: Option<String>,

        /// Output file for results
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Enable parallel computation
        #[arg(short, long)]
        parallel: bool,

        /// Use GPU acceleration if available
        #[arg(short, long)]
        gpu: bool,
    },

    /// Visualize mathematical objects (sheaves, representations, etc.)
    Visual {
        /// Type of object to visualize
        #[arg(short, long)]
        object_type: String,

        /// Input data file
        #[arg(short, long)]
        input: Option<PathBuf>,

        /// Output image file
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Interactive mode
        #[arg(short, long)]
        interactive: bool,

        /// Resolution (e.g., "1920x1080")
        #[arg(short, long, default_value = "800x600")]
        resolution: String,
    },

    /// Train neural networks for pattern recognition
    Train {
        /// Training dataset
        #[arg(short, long)]
        dataset: PathBuf,

        /// Model architecture
        #[arg(short, long, default_value = "default")]
        architecture: String,

        /// Number of epochs
        #[arg(short, long, default_value = "100")]
        epochs: u32,

        /// Batch size
        #[arg(short, long, default_value = "32")]
        batch_size: usize,

        /// Learning rate
        #[arg(short, long, default_value = "0.001")]
        learning_rate: f64,

        /// Save model to file
        #[arg(short, long)]
        save_model: Option<PathBuf>,
    },

    /// Verify mathematical properties and conjectures
    Verify {
        /// Property to verify
        #[arg(short, long)]
        property: String,

        /// Input object or file
        #[arg(short, long)]
        input: Option<String>,

        /// Verification depth
        #[arg(short, long, default_value = "standard")]
        depth: VerificationDepth,

        /// Generate proof certificate
        #[arg(short, long)]
        proof: bool,
    },

    /// Export results in various formats
    Export {
        /// Source file or computation ID
        #[arg(short, long)]
        source: String,

        /// Output file
        #[arg(short, long)]
        output: PathBuf,

        /// Export format
        #[arg(short, long)]
        format: ExportFormat,

        /// Include metadata
        #[arg(short, long)]
        metadata: bool,
    },

    /// Start interactive REPL session
    Repl {
        /// Load session from file
        #[arg(short, long)]
        load: Option<PathBuf>,

        /// Enable auto-save
        #[arg(short, long)]
        auto_save: bool,

        /// History file
        #[arg(short = 'H', long)]
        history: Option<PathBuf>,
    },

    /// Generate shell completions
    Completions {
        /// Shell to generate completions for
        shell: clap_complete::Shell,
    },

    /// Manage configuration
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },

    /// Database operations
    Db {
        #[command(subcommand)]
        action: DbAction,
    },
}


#[derive(Subcommand)]
enum ConfigAction {
    /// Show current configuration
    Show,
    /// Set a configuration value
    Set { key: String, value: String },
    /// Get a configuration value
    Get { key: String },
    /// Reset to defaults
    Reset,
    /// Edit configuration file
    Edit,
}

#[derive(Subcommand)]
enum DbAction {
    /// Initialize database
    Init,
    /// Run migrations
    Migrate,
    /// List stored computations
    List {
        #[arg(short, long, default_value = "10")]
        limit: u32,
    },
    /// Show computation details
    Show { id: String },
    /// Delete computation
    Delete { id: String },
    /// Export database
    Export { path: PathBuf },
    /// Import database
    Import { path: PathBuf },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize logging
    init_logging(cli.verbose)?;

    // Load configuration
    let config = Config::load(cli.config.as_deref())?;

    // Print banner
    if cli.verbose > 0 {
        print_banner();
    }

    // Handle commands
    match cli.command {
        None => {
            // No command specified, start REPL
            println!("{}", "Starting interactive REPL...".green());
            repl::start_repl(None, false, None, &config).await?;
        }
        Some(Commands::Compute {
            computation_type,
            input,
            output,
            parallel,
            gpu,
        }) => {
            compute::handle_compute(
                &computation_type,
                input.as_deref(),
                output.as_deref(),
                parallel,
                gpu,
                &cli.format,
                &config,
            )
            .await?;
        }
        Some(Commands::Visual {
            object_type,
            input,
            output,
            interactive,
            resolution,
        }) => {
            visual::handle_visual(
                &object_type,
                input.as_deref(),
                output.as_deref(),
                interactive,
                &resolution,
                &config,
            )
            .await?;
        }
        Some(Commands::Train {
            dataset,
            architecture,
            epochs,
            batch_size,
            learning_rate,
            save_model,
        }) => {
            train::handle_train(
                &dataset,
                &architecture,
                epochs,
                batch_size,
                learning_rate,
                save_model.as_deref(),
                &config,
            )
            .await?;
        }
        Some(Commands::Verify {
            property,
            input,
            depth,
            proof,
        }) => {
            verify::handle_verify(&property, input.as_deref(), &depth, proof, &config).await?;
        }
        Some(Commands::Export {
            source,
            output,
            format,
            metadata,
        }) => {
            export::handle_export(&source, &output, &format, metadata, &config).await?;
        }
        Some(Commands::Repl {
            load,
            auto_save,
            history,
        }) => {
            repl::start_repl(load.as_deref(), auto_save, history.as_deref(), &config).await?;
        }
        Some(Commands::Completions { shell }) => {
            generate_completions(shell);
        }
        Some(Commands::Config { action }) => {
            handle_config_action(action, &config)?;
        }
        Some(Commands::Db { action }) => {
            handle_db_action(action, &config).await?;
        }
    }

    Ok(())
}

fn init_logging(verbosity: u8) -> Result<()> {
    let level = match verbosity {
        0 => log::LevelFilter::Warn,
        1 => log::LevelFilter::Info,
        2 => log::LevelFilter::Debug,
        _ => log::LevelFilter::Trace,
    };

    env_logger::Builder::from_default_env()
        .filter_level(level)
        .init();

    Ok(())
}

fn print_banner() {
    println!(
        "{}",
        r#"
╔═══════════════════════════════════════════════════════════════╗
║                                                               ║
║   ██████╗ ███████╗ ██████╗ ███╗   ███╗███████╗████████╗██████╗██╗ ██████╗║
║  ██╔════╝ ██╔════╝██╔═══██╗████╗ ████║██╔════╝╚══██╔══╝██╔══██╗██║██╔════╝║
║  ██║  ███╗█████╗  ██║   ██║██╔████╔██║█████╗     ██║   ██████╔╝██║██║     ║
║  ██║   ██║██╔══╝  ██║   ██║██║╚██╔╝██║██╔══╝     ██║   ██╔══██╗██║██║     ║
║  ╚██████╔╝███████╗╚██████╔╝██║ ╚═╝ ██║███████╗   ██║   ██║  ██║██║╚██████╗║
║   ╚═════╝ ╚══════╝ ╚═════╝ ╚═╝     ╚═╝╚══════╝   ╚═╝   ╚═╝  ╚═╝╚═╝ ╚═════╝║
║                                                               ║
║                LANGLANDS CORRESPONDENCE CLI                   ║
║              Mathematical Computations Made Easy              ║
╚═══════════════════════════════════════════════════════════════╝
"#
        .cyan()
    );
}

fn generate_completions(shell: clap_complete::Shell) {
    use clap::CommandFactory;
    use clap_complete::generate;
    use std::io;

    let mut cmd = Cli::command();
    let name = cmd.get_name().to_string();
    generate(shell, &mut cmd, name, &mut io::stdout());
}

fn handle_config_action(action: ConfigAction, config: &Config) -> Result<()> {
    match action {
        ConfigAction::Show => {
            println!("{}", serde_json::to_string_pretty(config)?);
        }
        ConfigAction::Set { key, value } => {
            println!("Setting {} = {}", key.yellow(), value.green());
            // Implementation would update config file
        }
        ConfigAction::Get { key } => {
            println!("Getting value for {}", key.yellow());
            // Implementation would retrieve config value
        }
        ConfigAction::Reset => {
            println!("{}", "Resetting configuration to defaults...".yellow());
            // Implementation would reset config
        }
        ConfigAction::Edit => {
            println!("{}", "Opening configuration file in editor...".blue());
            // Implementation would open editor
        }
    }
    Ok(())
}

async fn handle_db_action(action: DbAction, config: &Config) -> Result<()> {
    use persistence::Database;

    let db = Database::new(&config.database_path).await?;

    match action {
        DbAction::Init => {
            println!("{}", "Initializing database...".green());
            db.init().await?;
        }
        DbAction::Migrate => {
            println!("{}", "Running migrations...".green());
            db.migrate().await?;
        }
        DbAction::List { limit } => {
            let computations = db.list_computations(limit).await?;
            println!("Found {} computations", computations.len());
            // Display computations in table format
        }
        DbAction::Show { id } => {
            let computation = db.get_computation(&id).await?;
            println!("{:#?}", computation);
        }
        DbAction::Delete { id } => {
            db.delete_computation(&id).await?;
            println!("Deleted computation {}", id.red());
        }
        DbAction::Export { path } => {
            db.export(&path).await?;
            println!("Exported database to {}", path.display());
        }
        DbAction::Import { path } => {
            db.import(&path).await?;
            println!("Imported database from {}", path.display());
        }
    }

    Ok(())
}