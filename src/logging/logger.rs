//! Logger trait and implementations for ruv-FANN
//!
//! This module provides the core logging interface and various implementations
//! for different output formats and destinations.

use crate::logging::schema::*;
use std::io::Write;
use std::sync::{Arc, Mutex};
use std::path::Path;
use std::fs::{File, OpenOptions};
use std::collections::VecDeque;

/// Core logging trait for ruv-FANN
pub trait Logger: Send + Sync {
    /// Log an event
    fn log(&self, event: LogEvent) -> Result<(), LogError>;
    
    /// Flush any buffered events
    fn flush(&self) -> Result<(), LogError>;
    
    /// Get logger statistics
    fn stats(&self) -> LoggerStats;
    
    /// Check if a log level is enabled
    fn is_enabled(&self, level: LogLevel) -> bool;
}

/// Log levels for filtering
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    Trace = 0,
    Debug = 1,
    Info = 2,
    Warn = 3,
    Error = 4,
}

/// Logger statistics
#[derive(Debug, Clone, Default)]
pub struct LoggerStats {
    pub events_logged: usize,
    pub events_dropped: usize,
    pub bytes_written: usize,
    pub errors_encountered: usize,
}

/// Logging errors
#[derive(Debug, thiserror::Error)]
pub enum LogError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(String),
    
    #[error("Logger is closed")]
    Closed,
    
    #[error("Buffer full")]
    BufferFull,
}

/// Configuration for loggers
#[derive(Debug, Clone)]
pub struct LoggerConfig {
    /// Minimum log level
    pub min_level: LogLevel,
    
    /// Buffer size for async logging
    pub buffer_size: usize,
    
    /// Whether to include metadata in output
    pub include_metadata: bool,
    
    /// Whether to pretty-print JSON
    pub pretty_json: bool,
    
    /// Maximum file size for rotation (bytes)
    pub max_file_size: Option<usize>,
    
    /// Whether to compress old log files
    pub compress_old_logs: bool,
}

impl Default for LoggerConfig {
    fn default() -> Self {
        Self {
            min_level: LogLevel::Info,
            buffer_size: 10000,
            include_metadata: true,
            pretty_json: false,
            max_file_size: Some(100 * 1024 * 1024), // 100MB
            compress_old_logs: true,
        }
    }
}

/// JSON logger implementation
pub struct JsonLogger {
    writer: Arc<Mutex<Box<dyn Write + Send>>>,
    config: LoggerConfig,
    stats: Arc<Mutex<LoggerStats>>,
}

impl JsonLogger {
    /// Create a new JSON logger writing to stdout
    pub fn stdout(config: LoggerConfig) -> Self {
        Self {
            writer: Arc::new(Mutex::new(Box::new(std::io::stdout()))),
            config,
            stats: Arc::new(Mutex::new(LoggerStats::default())),
        }
    }
    
    /// Create a new JSON logger writing to a file
    pub fn file<P: AsRef<Path>>(path: P, config: LoggerConfig) -> Result<Self, LogError> {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)?;
        
        Ok(Self {
            writer: Arc::new(Mutex::new(Box::new(file))),
            config,
            stats: Arc::new(Mutex::new(LoggerStats::default())),
        })
    }
    
    /// Create a new JSON logger writing to a buffer
    pub fn buffer(buffer: Vec<u8>, config: LoggerConfig) -> Self {
        Self {
            writer: Arc::new(Mutex::new(Box::new(buffer))),
            config,
            stats: Arc::new(Mutex::new(LoggerStats::default())),
        }
    }
}

impl Logger for JsonLogger {
    fn log(&self, event: LogEvent) -> Result<(), LogError> {
        if !self.should_log(&event) {
            return Ok(());
        }
        
        let json = if self.config.pretty_json {
            serde_json::to_string_pretty(&event)
        } else {
            serde_json::to_string(&event)
        }.map_err(|e| LogError::Serialization(e.to_string()))?;
        
        let mut writer = self.writer.lock().unwrap();
        let bytes_written = writer.write(json.as_bytes())?;
        writer.write_all(b"\n")?;
        
        let mut stats = self.stats.lock().unwrap();
        stats.events_logged += 1;
        stats.bytes_written += bytes_written + 1;
        
        Ok(())
    }
    
    fn flush(&self) -> Result<(), LogError> {
        let mut writer = self.writer.lock().unwrap();
        writer.flush()?;
        Ok(())
    }
    
    fn stats(&self) -> LoggerStats {
        self.stats.lock().unwrap().clone()
    }
    
    fn is_enabled(&self, level: LogLevel) -> bool {
        level >= self.config.min_level
    }
}

impl JsonLogger {
    fn should_log(&self, event: &LogEvent) -> bool {
        let level = match event {
            LogEvent::Error(_) => LogLevel::Error,
            LogEvent::TaskStart(_) | LogEvent::TaskEnd(_) => LogLevel::Info,
            LogEvent::MemoryAlloc(_) | LogEvent::MemoryFree(_) => LogLevel::Debug,
            LogEvent::MemoryPressure(e) => match e.pressure_level {
                MemoryPressureLevel::Critical | MemoryPressureLevel::High => LogLevel::Warn,
                _ => LogLevel::Debug,
            },
            LogEvent::NetworkForward(_) | LogEvent::NetworkBackward(_) => LogLevel::Trace,
            LogEvent::GpuOperation(_) | LogEvent::GpuKernelExecution(_) => LogLevel::Debug,
            _ => LogLevel::Info,
        };
        
        self.is_enabled(level)
    }
}

/// Human-readable logger implementation
pub struct HumanReadableLogger {
    writer: Arc<Mutex<Box<dyn Write + Send>>>,
    config: LoggerConfig,
    stats: Arc<Mutex<LoggerStats>>,
}

impl HumanReadableLogger {
    /// Create a new human-readable logger writing to stdout
    pub fn stdout(config: LoggerConfig) -> Self {
        Self {
            writer: Arc::new(Mutex::new(Box::new(std::io::stdout()))),
            config,
            stats: Arc::new(Mutex::new(LoggerStats::default())),
        }
    }
    
    /// Create a new human-readable logger writing to stderr
    pub fn stderr(config: LoggerConfig) -> Self {
        Self {
            writer: Arc::new(Mutex::new(Box::new(std::io::stderr()))),
            config,
            stats: Arc::new(Mutex::new(LoggerStats::default())),
        }
    }
    
    fn format_event(&self, event: &LogEvent) -> String {
        match event {
            LogEvent::TaskStart(e) => {
                format!("[TASK START] {} ({}) - Session: {}", 
                    e.task_name, e.task_category, e.base.session_id)
            },
            LogEvent::TaskEnd(e) => {
                format!("[TASK END] {} - Status: {:?}, Duration: {:.3}s", 
                    e.task_name, e.status, e.duration.as_secs_f64())
            },
            LogEvent::Error(e) => {
                format!("[ERROR] [{}] {} - Component: {}, Recoverable: {}", 
                    e.error_category, e.message, e.component, e.recoverable)
            },
            LogEvent::MemoryAlloc(e) => {
                format!("[MEM ALLOC] {} bytes ({}) for {} - Total: {} bytes", 
                    e.size_bytes, e.memory_type, e.purpose, e.total_usage_bytes)
            },
            LogEvent::MemoryPressure(e) => {
                format!("[MEM PRESSURE] Level: {:?} - Usage: {}/{} bytes", 
                    e.pressure_level, e.current_usage_bytes, e.available_bytes)
            },
            LogEvent::NetworkInit(e) => {
                format!("[NETWORK INIT] {} - Layers: {}, Params: {}, Memory: {} bytes", 
                    e.network_id, e.topology.layers.len(), e.total_parameters, e.estimated_memory_bytes)
            },
            LogEvent::TrainingEpochEnd(e) => {
                format!("[EPOCH {}] Loss: {:.6}, Val Loss: {:?}", 
                    e.epoch, e.loss, e.validation_loss)
            },
            LogEvent::GpuOperation(e) => {
                format!("[GPU] {} on {} - Duration: {:.3}ms, Memory: {} bytes", 
                    e.operation, e.device_id, e.duration.as_secs_f64() * 1000.0, e.memory_bytes)
            },
            LogEvent::PerformanceMetric(e) => {
                format!("[METRIC] {}: {:.3} {} ({})", 
                    e.metric_name, e.value, e.unit, e.component)
            },
            _ => format!("[{:?}]", event),
        }
    }
}

impl Logger for HumanReadableLogger {
    fn log(&self, event: LogEvent) -> Result<(), LogError> {
        let formatted = self.format_event(&event);
        
        let mut writer = self.writer.lock().unwrap();
        let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
        let line = format!("{} {}\n", timestamp, formatted);
        let bytes_written = line.len();
        writer.write_all(line.as_bytes())?;
        
        let mut stats = self.stats.lock().unwrap();
        stats.events_logged += 1;
        stats.bytes_written += bytes_written;
        
        Ok(())
    }
    
    fn flush(&self) -> Result<(), LogError> {
        let mut writer = self.writer.lock().unwrap();
        writer.flush()?;
        Ok(())
    }
    
    fn stats(&self) -> LoggerStats {
        self.stats.lock().unwrap().clone()
    }
    
    fn is_enabled(&self, level: LogLevel) -> bool {
        level >= self.config.min_level
    }
}

/// Buffered logger for high-throughput scenarios
pub struct BufferedLogger {
    buffer: Arc<Mutex<VecDeque<LogEvent>>>,
    inner: Box<dyn Logger>,
    config: LoggerConfig,
}

impl BufferedLogger {
    pub fn new(inner: Box<dyn Logger>, config: LoggerConfig) -> Self {
        Self {
            buffer: Arc::new(Mutex::new(VecDeque::with_capacity(config.buffer_size))),
            inner,
            config,
        }
    }
    
    /// Process buffered events
    pub fn process_buffer(&self) -> Result<usize, LogError> {
        let mut buffer = self.buffer.lock().unwrap();
        let mut processed = 0;
        
        while let Some(event) = buffer.pop_front() {
            self.inner.log(event)?;
            processed += 1;
        }
        
        Ok(processed)
    }
}

impl Logger for BufferedLogger {
    fn log(&self, event: LogEvent) -> Result<(), LogError> {
        let mut buffer = self.buffer.lock().unwrap();
        
        if buffer.len() >= self.config.buffer_size {
            return Err(LogError::BufferFull);
        }
        
        buffer.push_back(event);
        Ok(())
    }
    
    fn flush(&self) -> Result<(), LogError> {
        self.process_buffer()?;
        self.inner.flush()
    }
    
    fn stats(&self) -> LoggerStats {
        self.inner.stats()
    }
    
    fn is_enabled(&self, level: LogLevel) -> bool {
        self.inner.is_enabled(level)
    }
}

/// Multi-logger that writes to multiple destinations
pub struct MultiLogger {
    loggers: Vec<Box<dyn Logger>>,
}

impl MultiLogger {
    pub fn new(loggers: Vec<Box<dyn Logger>>) -> Self {
        Self { loggers }
    }
}

impl Logger for MultiLogger {
    fn log(&self, event: LogEvent) -> Result<(), LogError> {
        let mut last_error = None;
        
        for logger in &self.loggers {
            if let Err(e) = logger.log(event.clone()) {
                last_error = Some(e);
            }
        }
        
        if let Some(e) = last_error {
            Err(e)
        } else {
            Ok(())
        }
    }
    
    fn flush(&self) -> Result<(), LogError> {
        let mut last_error = None;
        
        for logger in &self.loggers {
            if let Err(e) = logger.flush() {
                last_error = Some(e);
            }
        }
        
        if let Some(e) = last_error {
            Err(e)
        } else {
            Ok(())
        }
    }
    
    fn stats(&self) -> LoggerStats {
        // Aggregate stats from all loggers
        self.loggers.iter()
            .map(|l| l.stats())
            .fold(LoggerStats::default(), |mut acc, stats| {
                acc.events_logged += stats.events_logged;
                acc.events_dropped += stats.events_dropped;
                acc.bytes_written += stats.bytes_written;
                acc.errors_encountered += stats.errors_encountered;
                acc
            })
    }
    
    fn is_enabled(&self, level: LogLevel) -> bool {
        self.loggers.iter().any(|l| l.is_enabled(level))
    }
}

/// Null logger that discards all events
pub struct NullLogger;

impl Logger for NullLogger {
    fn log(&self, _event: LogEvent) -> Result<(), LogError> {
        Ok(())
    }
    
    fn flush(&self) -> Result<(), LogError> {
        Ok(())
    }
    
    fn stats(&self) -> LoggerStats {
        LoggerStats::default()
    }
    
    fn is_enabled(&self, _level: LogLevel) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_json_logger_stdout() {
        let config = LoggerConfig::default();
        let logger = JsonLogger::stdout(config);
        
        let builder = EventBuilder::new("test-session".to_string());
        let event = LogEvent::TaskStart(TaskStartEvent {
            base: builder.build_base(),
            task_name: "test".to_string(),
            task_category: "test".to_string(),
            parameters: Default::default(),
            parent_task_id: None,
        });
        
        assert!(logger.log(event).is_ok());
        assert!(logger.flush().is_ok());
    }
    
    #[test]
    fn test_human_readable_logger() {
        let config = LoggerConfig::default();
        let logger = HumanReadableLogger::stdout(config);
        
        let builder = EventBuilder::new("test-session".to_string());
        let event = LogEvent::Error(ErrorEvent {
            base: builder.build_base(),
            error_category: "test".to_string(),
            message: "Test error".to_string(),
            stack_trace: None,
            component: "test".to_string(),
            recovery_action: None,
            recoverable: true,
        });
        
        assert!(logger.log(event).is_ok());
    }
    
    #[test]
    fn test_buffered_logger() {
        let inner = Box::new(NullLogger);
        let config = LoggerConfig::default();
        let logger = BufferedLogger::new(inner, config);
        
        // Add multiple events
        for i in 0..10 {
            let builder = EventBuilder::new("test-session".to_string());
            let event = LogEvent::Custom(CustomEvent {
                base: builder.build_base(),
                name: format!("event_{}", i),
                data: serde_json::json!({"index": i}),
            });
            assert!(logger.log(event).is_ok());
        }
        
        // Process buffer
        let processed = logger.process_buffer().unwrap();
        assert_eq!(processed, 10);
    }
}