//! Dashboard data structures

use alloc::string::String;
use alloc::vec::Vec;

/// Dashboard data for visualization
pub struct DashboardData {
    /// Title of the dashboard
    pub title: String,
    /// Data points
    pub data: Vec<f32>,
}

impl DashboardData {
    /// Create new dashboard data
    pub fn new(title: String) -> Self {
        Self {
            title,
            data: Vec::new(),
        }
    }
}

/// Heatmap visualization data
pub struct HeatmapData {
    /// Width of heatmap
    pub width: u32,
    /// Height of heatmap
    pub height: u32,
    /// Data values
    pub values: Vec<f32>,
}

impl HeatmapData {
    /// Create new heatmap data
    pub fn new(width: u32, height: u32) -> Self {
        let size = (width * height) as usize;
        Self {
            width,
            height,
            values: vec![0.0; size],
        }
    }
}

impl Default for DashboardData {
    fn default() -> Self {
        Self::new("Default Dashboard".into())
    }
}

impl Default for HeatmapData {
    fn default() -> Self {
        Self::new(32, 32)
    }
}