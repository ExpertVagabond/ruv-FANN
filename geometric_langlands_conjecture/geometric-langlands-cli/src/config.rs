use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub database_path: PathBuf,
    pub default_precision: u32,
    pub max_iterations: u32,
    pub convergence_threshold: f64,
    pub computation: ComputationConfig,
    pub visualization: VisualizationConfig,
    pub neural: NeuralConfig,
    pub repl: ReplConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputationConfig {
    pub enable_parallel: bool,
    pub enable_gpu: bool,
    pub cache_results: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualizationConfig {
    pub default_resolution: [u32; 2],
    pub color_scheme: String,
    pub enable_latex: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeuralConfig {
    pub default_architecture: String,
    pub learning_rate: f64,
    pub batch_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplConfig {
    pub history_size: usize,
    pub auto_save: bool,
    pub prompt: String,
}

impl Default for Config {
    fn default() -> Self {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        let config_dir = home.join(".config").join("langlands-cli");
        
        Self {
            database_path: config_dir.join("database.db"),
            default_precision: 64,
            max_iterations: 10000,
            convergence_threshold: 1e-10,
            computation: ComputationConfig {
                enable_parallel: true,
                enable_gpu: false,
                cache_results: true,
            },
            visualization: VisualizationConfig {
                default_resolution: [800, 600],
                color_scheme: "viridis".to_string(),
                enable_latex: true,
            },
            neural: NeuralConfig {
                default_architecture: "langlands_v1".to_string(),
                learning_rate: 0.001,
                batch_size: 32,
            },
            repl: ReplConfig {
                history_size: 1000,
                auto_save: true,
                prompt: "langlands> ".to_string(),
            },
        }
    }
}

impl Config {
    pub fn load(config_path: Option<&Path>) -> Result<Self> {
        if let Some(path) = config_path {
            if path.exists() {
                let content = std::fs::read_to_string(path)?;
                let config: Config = toml::from_str(&content)?;
                return Ok(config);
            }
        }
        
        // Try default config location
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        let default_path = home.join(".config").join("langlands-cli").join("config.toml");
        
        if default_path.exists() {
            let content = std::fs::read_to_string(default_path)?;
            let config: Config = toml::from_str(&content)?;
            Ok(config)
        } else {
            // Create default config
            let config = Config::default();
            
            // Create config directory
            if let Some(parent) = default_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            
            // Save default config
            let content = toml::to_string_pretty(&config)?;
            std::fs::write(default_path, content)?;
            
            Ok(config)
        }
    }
    
    pub fn save(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }
}