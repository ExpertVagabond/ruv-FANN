//! Metrics collection and monitoring

use std::sync::Arc;
use std::time::{Duration, Instant};
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use tokio::sync::RwLock;
use uuid::Uuid;

/// Metric data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricPoint {
    #[serde(skip, default = "Instant::now")]
    pub timestamp: Instant,
    pub value: f64,
    pub tags: HashMap<String, String>,
}

/// Time series metric
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeSeries {
    pub name: String,
    pub points: Vec<MetricPoint>,
    pub unit: String,
    pub description: String,
}

/// Metrics collector
pub struct MetricsCollector {
    series: Arc<RwLock<HashMap<String, TimeSeries>>>,
    start_time: Instant,
}

impl MetricsCollector {
    /// Create a new metrics collector
    pub fn new() -> Self {
        Self {
            series: Arc::new(RwLock::new(HashMap::new())),
            start_time: Instant::now(),
        }
    }
    
    /// Record a metric value
    pub async fn record(&self, name: &str, value: f64, tags: HashMap<String, String>) {
        let mut series = self.series.write().await;
        
        let metric = series.entry(name.to_string()).or_insert_with(|| TimeSeries {
            name: name.to_string(),
            points: Vec::new(),
            unit: "count".to_string(),
            description: format!("Metric: {}", name),
        });
        
        metric.points.push(MetricPoint {
            timestamp: Instant::now(),
            value,
            tags,
        });
        
        // Keep only last 1000 points
        if metric.points.len() > 1000 {
            metric.points.remove(0);
        }
    }
    
    /// Get all metrics
    pub async fn get_all(&self) -> HashMap<String, TimeSeries> {
        self.series.read().await.clone()
    }
    
    /// Get metric by name
    pub async fn get(&self, name: &str) -> Option<TimeSeries> {
        self.series.read().await.get(name).cloned()
    }
    
    /// Export metrics in Prometheus format
    pub async fn export_prometheus(&self) -> String {
        let series = self.series.read().await;
        let mut output = String::new();
        
        for (name, metric) in series.iter() {
            output.push_str(&format!("# HELP {} {}\n", name, metric.description));
            output.push_str(&format!("# TYPE {} gauge\n", name));
            
            if let Some(point) = metric.points.last() {
                output.push_str(&format!("{} {}\n", name, point.value));
            }
        }
        
        output
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_metrics_collection() {
        let collector = MetricsCollector::new();
        
        // Record metrics
        collector.record("test_metric", 42.0, HashMap::new()).await;
        collector.record("test_metric", 43.0, HashMap::new()).await;
        
        // Get metric
        let metric = collector.get("test_metric").await.unwrap();
        assert_eq!(metric.points.len(), 2);
        assert_eq!(metric.points[0].value, 42.0);
        assert_eq!(metric.points[1].value, 43.0);
        
        // Export Prometheus
        let prometheus = collector.export_prometheus().await;
        assert!(prometheus.contains("test_metric"));
    }
}