//! Visualization module for neural network activity
//! 
//! Provides capabilities for visualizing neural network state,
//! activations, and performance metrics.

use serde::{Serialize, Deserialize};
use std::path::Path;
use anyhow::Result;

/// Visualization output format
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum OutputFormat {
    /// HTML with interactive visualizations
    Html,
    /// PNG static images
    Png,
    /// SVG vector graphics
    Svg,
    /// JSON data export
    Json,
}

/// Visualization configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualizationConfig {
    /// Output directory for visualization files
    pub output_dir: String,
    /// Output format
    pub format: OutputFormat,
    /// Enable network graph visualization
    pub network_graph: bool,
    /// Enable activation heatmaps
    pub activation_heatmap: bool,
    /// Enable performance charts
    pub performance_charts: bool,
}

impl Default for VisualizationConfig {
    fn default() -> Self {
        Self {
            output_dir: String::from("./viz_output"),
            format: OutputFormat::Html,
            network_graph: true,
            activation_heatmap: true,
            performance_charts: true,
        }
    }
}

/// Neural network visualization data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkVisualization {
    /// Number of layers
    pub layers: Vec<LayerInfo>,
    /// Connections between layers
    pub connections: Vec<Connection>,
    /// Current activations
    pub activations: Vec<Vec<f32>>,
    /// Performance metrics
    pub metrics: PerformanceData,
}

/// Layer information for visualization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerInfo {
    pub id: usize,
    pub neurons: usize,
    pub layer_type: String,
}

/// Connection between neurons
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Connection {
    pub from_layer: usize,
    pub from_neuron: usize,
    pub to_layer: usize,
    pub to_neuron: usize,
    pub weight: f32,
}

/// Performance data for visualization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceData {
    pub execution_time_ms: f64,
    pub memory_usage_mb: f64,
    pub operations_per_second: f64,
}

/// Visualizer for neural network simulations
pub struct Visualizer {
    config: VisualizationConfig,
}

impl Visualizer {
    /// Create a new visualizer with the given configuration
    pub fn new(config: VisualizationConfig) -> Self {
        Self { config }
    }

    /// Generate visualizations for the network
    pub fn visualize(&self, data: &NetworkVisualization) -> Result<()> {
        // Create output directory
        std::fs::create_dir_all(&self.config.output_dir)?;

        if self.config.network_graph {
            self.generate_network_graph(data)?;
        }

        if self.config.activation_heatmap {
            self.generate_activation_heatmap(data)?;
        }

        if self.config.performance_charts {
            self.generate_performance_charts(data)?;
        }

        Ok(())
    }

    /// Generate network graph visualization
    fn generate_network_graph(&self, data: &NetworkVisualization) -> Result<()> {
        let output_path = Path::new(&self.config.output_dir).join("network_graph.html");
        
        // For now, create a simple HTML file
        let html = format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <title>Neural Network Graph</title>
    <style>
        body {{ font-family: Arial, sans-serif; margin: 20px; }}
        .info {{ background: #f0f0f0; padding: 10px; border-radius: 5px; }}
    </style>
</head>
<body>
    <h1>Neural Network Graph</h1>
    <div class="info">
        <h2>Network Structure</h2>
        <p>Layers: {}</p>
        <p>Total Connections: {}</p>
        <p>Note: Full interactive visualization would be implemented with D3.js or similar</p>
    </div>
</body>
</html>"#,
            data.layers.len(),
            data.connections.len()
        );

        std::fs::write(output_path, html)?;
        Ok(())
    }

    /// Generate activation heatmap
    fn generate_activation_heatmap(&self, data: &NetworkVisualization) -> Result<()> {
        let output_path = Path::new(&self.config.output_dir).join("activation_heatmap.png");
        
        // For now, create a placeholder file
        // In a real implementation, this would use an image library to generate the heatmap
        std::fs::write(output_path, b"PNG placeholder for activation heatmap")?;
        
        Ok(())
    }

    /// Generate performance charts
    fn generate_performance_charts(&self, data: &NetworkVisualization) -> Result<()> {
        let output_path = Path::new(&self.config.output_dir).join("performance.json");
        
        // Export performance data as JSON
        let json = serde_json::to_string_pretty(&data.metrics)?;
        std::fs::write(output_path, json)?;
        
        Ok(())
    }
}

/// Extension trait for neural network visualization
pub trait Visualizable {
    /// Generate visualization data
    fn visualize(&self, activations: &[Vec<f32>]) -> NetworkVisualization;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_visualization_config() {
        let config = VisualizationConfig::default();
        assert_eq!(config.output_dir, "./viz_output");
        assert!(config.network_graph);
        assert!(config.activation_heatmap);
    }

    #[test]
    fn test_visualizer_creation() {
        let config = VisualizationConfig::default();
        let visualizer = Visualizer::new(config);
        
        let data = NetworkVisualization {
            layers: vec![
                LayerInfo { id: 0, neurons: 10, layer_type: "input".to_string() },
                LayerInfo { id: 1, neurons: 5, layer_type: "hidden".to_string() },
                LayerInfo { id: 2, neurons: 2, layer_type: "output".to_string() },
            ],
            connections: vec![],
            activations: vec![],
            metrics: PerformanceData {
                execution_time_ms: 100.0,
                memory_usage_mb: 50.0,
                operations_per_second: 1000.0,
            },
        };

        // Should not panic
        let _ = visualizer.visualize(&data);
    }
}