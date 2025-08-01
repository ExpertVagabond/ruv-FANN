//! Metrics collection and JSON export for the Semantic Cartan Matrix system
//! 
//! This crate provides comprehensive metrics collection, logging, and JSON
//! export capabilities for monitoring system performance and behavior.

#![cfg_attr(not(feature = "std"), no_std)]
#![forbid(unsafe_code)]

extern crate alloc;

pub mod collector;
pub mod timing;
pub mod export;
pub mod dashboard;

pub use collector::{MetricsCollector, SystemMetrics, AgentMetrics};
pub use timing::{Timer, TimingInfo};
pub use export::{JsonExporter, MetricsReport};
pub use dashboard::{DashboardData, HeatmapData};

// Standalone version - no dependency on micro_core prelude
use alloc::string::String;
use alloc::vec::Vec;

/// Result type for metrics operations
pub type Result<T> = core::result::Result<T, &'static str>;