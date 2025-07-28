//! Logging and verification system for ruv-FANN
//!
//! This module provides comprehensive logging capabilities for neural network
//! operations, including structured JSON logging, human-readable output, and
//! verification pipelines for analyzing logged events.
//!
//! # Features
//! - Structured JSON logging with schema validation
//! - Human-readable logging for development
//! - Buffered logging for high-throughput scenarios
//! - Multi-destination logging
//! - Verification pipeline for post-processing analysis
//! - Built-in verification rules for common issues
//!
//! # Example
//! ```no_run
//! use ruv_fann::logging::{JsonLogger, LoggerConfig, EventBuilder, LogEvent, TaskStartEvent};
//! use std::collections::HashMap;
//!
//! // Create a JSON logger
//! let config = LoggerConfig::default();
//! let logger = JsonLogger::stdout(config);
//!
//! // Build and log an event
//! let builder = EventBuilder::new("session-123".to_string());
//! let event = LogEvent::TaskStart(TaskStartEvent {
//!     base: builder.build_base(),
//!     task_name: "training".to_string(),
//!     task_category: "neural_network".to_string(),
//!     parameters: HashMap::new(),
//!     parent_task_id: None,
//! });
//!
//! logger.log(event).unwrap();
//! ```

pub mod schema;
pub mod logger;
pub mod verification;

// Re-export commonly used types
pub use schema::{
    LogEvent, EventBase, EventBuilder,
    TaskStartEvent, TaskEndEvent, TaskStatus,
    MemoryAllocEvent, MemoryFreeEvent, MemoryPressureEvent, MemoryPressureLevel,
    NetworkInitEvent, NetworkForwardEvent, NetworkBackwardEvent,
    TrainingEpochEvent, TrainingConvergenceEvent,
    GpuOperationEvent, GpuMemoryTransferEvent, GpuKernelExecutionEvent,
    PerformanceMetricEvent, ErrorEvent, CustomEvent,
    NetworkTopology, LayerInfo,
};

pub use logger::{
    Logger, LogLevel, LoggerConfig, LoggerStats, LogError,
    JsonLogger, HumanReadableLogger, BufferedLogger, MultiLogger, NullLogger,
};

pub use verification::{
    VerificationRule, VerificationResult, VerificationSeverity,
    VerificationReport, VerificationIssue, VerificationPipeline,
    PipelineReport, VerificationStatus,
    MemoryLeakDetector, TaskCompletionVerifier, 
    GpuPerformanceVerifier, TrainingConvergenceVerifier,
};

use std::sync::{Arc, RwLock};
use lazy_static::lazy_static;

lazy_static! {
    /// Global logger instance
    static ref GLOBAL_LOGGER: Arc<RwLock<Option<Box<dyn Logger>>>> = Arc::new(RwLock::new(None));
}

/// Initialize the global logger
pub fn init_logger(logger: Box<dyn Logger>) {
    let mut global = GLOBAL_LOGGER.write().unwrap();
    *global = Some(logger);
}

/// Log an event using the global logger
pub fn log(event: LogEvent) -> Result<(), LogError> {
    let global = GLOBAL_LOGGER.read().unwrap();
    if let Some(logger) = global.as_ref() {
        logger.log(event)
    } else {
        // Silently succeed if no logger is configured
        Ok(())
    }
}

/// Flush the global logger
pub fn flush() -> Result<(), LogError> {
    let global = GLOBAL_LOGGER.read().unwrap();
    if let Some(logger) = global.as_ref() {
        logger.flush()
    } else {
        Ok(())
    }
}

/// Get statistics from the global logger
pub fn stats() -> Option<LoggerStats> {
    let global = GLOBAL_LOGGER.read().unwrap();
    global.as_ref().map(|logger| logger.stats())
}

/// Check if a log level is enabled in the global logger
pub fn is_enabled(level: LogLevel) -> bool {
    let global = GLOBAL_LOGGER.read().unwrap();
    global.as_ref().map(|logger| logger.is_enabled(level)).unwrap_or(false)
}

/// Helper macro for logging events
#[macro_export]
macro_rules! log_event {
    ($event:expr) => {
        $crate::logging::log($event).ok();
    };
}

/// Helper macro for logging task start
#[macro_export]
macro_rules! log_task_start {
    ($session:expr, $name:expr, $category:expr) => {{
        let builder = $crate::logging::EventBuilder::new($session);
        let event = $crate::logging::LogEvent::TaskStart($crate::logging::TaskStartEvent {
            base: builder.build_base(),
            task_name: $name.to_string(),
            task_category: $category.to_string(),
            parameters: std::collections::HashMap::new(),
            parent_task_id: None,
        });
        $crate::logging::log(event)
    }};
    ($session:expr, $name:expr, $category:expr, $params:expr) => {{
        let builder = $crate::logging::EventBuilder::new($session);
        let event = $crate::logging::LogEvent::TaskStart($crate::logging::TaskStartEvent {
            base: builder.build_base(),
            task_name: $name.to_string(),
            task_category: $category.to_string(),
            parameters: $params,
            parent_task_id: None,
        });
        $crate::logging::log(event)
    }};
}

/// Helper macro for logging task end
#[macro_export]
macro_rules! log_task_end {
    ($session:expr, $start_id:expr, $name:expr, $status:expr, $duration:expr) => {{
        let builder = $crate::logging::EventBuilder::new($session);
        let event = $crate::logging::LogEvent::TaskEnd($crate::logging::TaskEndEvent {
            base: builder.build_base(),
            task_start_id: $start_id,
            task_name: $name.to_string(),
            status: $status,
            duration: $duration,
            result: None,
            error: None,
        });
        $crate::logging::log(event)
    }};
}

/// Helper macro for logging errors
#[macro_export]
macro_rules! log_error {
    ($session:expr, $category:expr, $message:expr, $component:expr) => {{
        let builder = $crate::logging::EventBuilder::new($session);
        let event = $crate::logging::LogEvent::Error($crate::logging::ErrorEvent {
            base: builder.build_base(),
            error_category: $category.to_string(),
            message: $message.to_string(),
            stack_trace: None,
            component: $component.to_string(),
            recovery_action: None,
            recoverable: false,
        });
        $crate::logging::log(event)
    }};
}

/// Helper macro for logging performance metrics
#[macro_export]
macro_rules! log_metric {
    ($session:expr, $name:expr, $value:expr, $unit:expr, $component:expr) => {{
        let builder = $crate::logging::EventBuilder::new($session);
        let event = $crate::logging::LogEvent::PerformanceMetric($crate::logging::PerformanceMetricEvent {
            base: builder.build_base(),
            metric_name: $name.to_string(),
            value: $value,
            unit: $unit.to_string(),
            component: $component.to_string(),
            dimensions: std::collections::HashMap::new(),
        });
        $crate::logging::log(event)
    }};
}

/// Preset logger configurations
pub mod presets {
    use super::*;
    
    /// Development configuration with human-readable output to stdout
    pub fn development() -> Box<dyn Logger> {
        Box::new(HumanReadableLogger::stdout(LoggerConfig {
            min_level: LogLevel::Debug,
            pretty_json: true,
            ..Default::default()
        }))
    }
    
    /// Production configuration with JSON output and file rotation
    pub fn production(log_file: &str) -> Result<Box<dyn Logger>, LogError> {
        Ok(Box::new(JsonLogger::file(log_file, LoggerConfig {
            min_level: LogLevel::Info,
            pretty_json: false,
            max_file_size: Some(100 * 1024 * 1024), // 100MB
            compress_old_logs: true,
            ..Default::default()
        })?))
    }
    
    /// High-performance configuration with buffering
    pub fn high_performance(log_file: &str) -> Result<Box<dyn Logger>, LogError> {
        let json_logger = JsonLogger::file(log_file, LoggerConfig {
            min_level: LogLevel::Info,
            pretty_json: false,
            ..Default::default()
        })?;
        
        Ok(Box::new(BufferedLogger::new(
            Box::new(json_logger),
            LoggerConfig {
                buffer_size: 50000,
                ..Default::default()
            }
        )))
    }
    
    /// Multi-output configuration for comprehensive logging
    pub fn comprehensive(log_file: &str) -> Result<Box<dyn Logger>, LogError> {
        let json_file = JsonLogger::file(log_file, LoggerConfig {
            min_level: LogLevel::Debug,
            pretty_json: false,
            ..Default::default()
        })?;
        
        let human_stderr = HumanReadableLogger::stderr(LoggerConfig {
            min_level: LogLevel::Warn,
            ..Default::default()
        });
        
        Ok(Box::new(MultiLogger::new(vec![
            Box::new(json_file),
            Box::new(human_stderr),
        ])))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_global_logger() {
        // Initialize with null logger for testing
        init_logger(Box::new(NullLogger));
        
        // Test logging
        let builder = EventBuilder::new("test".to_string());
        let event = LogEvent::Custom(CustomEvent {
            base: builder.build_base(),
            name: "test".to_string(),
            data: serde_json::json!({}),
        });
        
        assert!(log(event).is_ok());
        assert!(flush().is_ok());
    }
    
    #[test]
    fn test_log_macros() {
        init_logger(Box::new(NullLogger));
        
        // Test task start macro
        assert!(log_task_start!("session", "test_task", "testing").is_ok());
        
        // Test error macro
        assert!(log_error!("session", "test", "error message", "test_component").is_ok());
        
        // Test metric macro
        assert!(log_metric!("session", "latency", 42.5, "ms", "network").is_ok());
    }
}