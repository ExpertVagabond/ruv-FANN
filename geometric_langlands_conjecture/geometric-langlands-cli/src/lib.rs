// Re-export core types for CLI use
// Temporarily disabled for standalone CLI build
// pub use geometric_langlands::*;

pub mod config;
pub mod persistence;
pub mod repl;
pub mod visualization;
pub mod commands;

// CLI-specific types that should be defined here instead of imported from main
#[derive(Clone, Debug, clap::ValueEnum)]
pub enum OutputFormat {
    Pretty,
    Json,
    Plain,
    LaTeX,
}

#[derive(Clone, Debug, clap::ValueEnum)]
pub enum VerificationDepth {
    Quick,
    Standard,
    Deep,
    Exhaustive,
}

#[derive(Clone, Debug, clap::ValueEnum)]
pub enum ExportFormat {
    Json,
    LaTeX,
    Mathematica,
    SageMath,
    Python,
    Csv,
    Binary,
}