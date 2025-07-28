//! JSON Schema definitions for structured logging in ruv-FANN
//! 
//! This module defines the structured logging formats used for verification,
//! debugging, and performance analysis throughout the neural network operations.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, SystemTime};

/// Core event types for ruv-FANN operations
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "event_type", rename_all = "snake_case")]
pub enum LogEvent {
    /// Task lifecycle events
    TaskStart(TaskStartEvent),
    TaskEnd(TaskEndEvent),
    
    /// Memory management events
    MemoryAlloc(MemoryAllocEvent),
    MemoryFree(MemoryFreeEvent),
    MemoryPressure(MemoryPressureEvent),
    
    /// Network operations
    NetworkInit(NetworkInitEvent),
    NetworkForward(NetworkForwardEvent),
    NetworkBackward(NetworkBackwardEvent),
    
    /// Training events
    TrainingEpochStart(TrainingEpochEvent),
    TrainingEpochEnd(TrainingEpochEvent),
    TrainingConvergence(TrainingConvergenceEvent),
    
    /// GPU/WebGPU operations
    GpuOperation(GpuOperationEvent),
    GpuMemoryTransfer(GpuMemoryTransferEvent),
    GpuKernelExecution(GpuKernelExecutionEvent),
    
    /// Performance metrics
    PerformanceMetric(PerformanceMetricEvent),
    
    /// Error events
    Error(ErrorEvent),
    
    /// Custom events for extensions
    Custom(CustomEvent),
}

/// Base fields common to all events
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EventBase {
    /// Unique event ID
    pub event_id: String,
    
    /// Timestamp when the event occurred
    #[serde(with = "system_time_serde")]
    pub timestamp: SystemTime,
    
    /// Thread ID where the event occurred
    pub thread_id: String,
    
    /// Optional correlation ID for tracking related events
    pub correlation_id: Option<String>,
    
    /// Session ID for grouping events
    pub session_id: String,
    
    /// Additional metadata
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Task start event
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TaskStartEvent {
    #[serde(flatten)]
    pub base: EventBase,
    
    /// Task name/identifier
    pub task_name: String,
    
    /// Task category (training, inference, optimization, etc.)
    pub task_category: String,
    
    /// Task parameters
    pub parameters: HashMap<String, serde_json::Value>,
    
    /// Parent task ID if this is a subtask
    pub parent_task_id: Option<String>,
}

/// Task completion event
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TaskEndEvent {
    #[serde(flatten)]
    pub base: EventBase,
    
    /// Reference to the task start event
    pub task_start_id: String,
    
    /// Task name for correlation
    pub task_name: String,
    
    /// Task completion status
    pub status: TaskStatus,
    
    /// Duration of the task
    #[serde(with = "duration_serde")]
    pub duration: Duration,
    
    /// Result summary
    pub result: Option<serde_json::Value>,
    
    /// Error details if status is failed
    pub error: Option<String>,
}

/// Task completion status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TaskStatus {
    Success,
    Failed,
    Cancelled,
    Timeout,
}

/// Memory allocation event
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MemoryAllocEvent {
    #[serde(flatten)]
    pub base: EventBase,
    
    /// Allocation size in bytes
    pub size_bytes: usize,
    
    /// Memory type (heap, gpu, shared, etc.)
    pub memory_type: String,
    
    /// Purpose of allocation
    pub purpose: String,
    
    /// Memory pool if applicable
    pub pool_name: Option<String>,
    
    /// Current total memory usage after allocation
    pub total_usage_bytes: usize,
}

/// Memory deallocation event
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MemoryFreeEvent {
    #[serde(flatten)]
    pub base: EventBase,
    
    /// Size freed in bytes
    pub size_bytes: usize,
    
    /// Memory type
    pub memory_type: String,
    
    /// Reference to allocation event
    pub alloc_event_id: Option<String>,
    
    /// Current total memory usage after free
    pub total_usage_bytes: usize,
}

/// Memory pressure event
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MemoryPressureEvent {
    #[serde(flatten)]
    pub base: EventBase,
    
    /// Pressure level
    pub pressure_level: MemoryPressureLevel,
    
    /// Current memory usage
    pub current_usage_bytes: usize,
    
    /// Available memory
    pub available_bytes: usize,
    
    /// Memory limit if set
    pub limit_bytes: Option<usize>,
    
    /// Actions taken
    pub actions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum MemoryPressureLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// Network initialization event
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NetworkInitEvent {
    #[serde(flatten)]
    pub base: EventBase,
    
    /// Network identifier
    pub network_id: String,
    
    /// Network topology
    pub topology: NetworkTopology,
    
    /// Total parameters
    pub total_parameters: usize,
    
    /// Estimated memory usage
    pub estimated_memory_bytes: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NetworkTopology {
    pub layers: Vec<LayerInfo>,
    pub total_neurons: usize,
    pub total_connections: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LayerInfo {
    pub layer_type: String,
    pub neurons: usize,
    pub activation: String,
}

/// Forward propagation event
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NetworkForwardEvent {
    #[serde(flatten)]
    pub base: EventBase,
    
    /// Network identifier
    pub network_id: String,
    
    /// Batch size
    pub batch_size: usize,
    
    /// Execution time
    #[serde(with = "duration_serde")]
    pub duration: Duration,
    
    /// Operations performed
    pub operations_count: usize,
    
    /// FLOPS if calculated
    pub flops: Option<f64>,
}

/// Backward propagation event
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NetworkBackwardEvent {
    #[serde(flatten)]
    pub base: EventBase,
    
    /// Network identifier
    pub network_id: String,
    
    /// Batch size
    pub batch_size: usize,
    
    /// Execution time
    #[serde(with = "duration_serde")]
    pub duration: Duration,
    
    /// Gradient norm
    pub gradient_norm: Option<f64>,
    
    /// Learning rate used
    pub learning_rate: f64,
}

/// Training epoch event
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TrainingEpochEvent {
    #[serde(flatten)]
    pub base: EventBase,
    
    /// Epoch number
    pub epoch: usize,
    
    /// Training loss
    pub loss: f64,
    
    /// Validation loss if available
    pub validation_loss: Option<f64>,
    
    /// Additional metrics
    pub metrics: HashMap<String, f64>,
}

/// Training convergence event
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TrainingConvergenceEvent {
    #[serde(flatten)]
    pub base: EventBase,
    
    /// Final epoch
    pub final_epoch: usize,
    
    /// Final loss
    pub final_loss: f64,
    
    /// Convergence criteria met
    pub criteria_met: Vec<String>,
    
    /// Total training time
    #[serde(with = "duration_serde")]
    pub total_duration: Duration,
}

/// GPU operation event
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GpuOperationEvent {
    #[serde(flatten)]
    pub base: EventBase,
    
    /// Operation type
    pub operation: String,
    
    /// Device identifier
    pub device_id: String,
    
    /// Execution time
    #[serde(with = "duration_serde")]
    pub duration: Duration,
    
    /// Memory used
    pub memory_bytes: usize,
    
    /// Success status
    pub success: bool,
}

/// GPU memory transfer event
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GpuMemoryTransferEvent {
    #[serde(flatten)]
    pub base: EventBase,
    
    /// Transfer direction (host_to_device, device_to_host, device_to_device)
    pub direction: String,
    
    /// Size transferred
    pub size_bytes: usize,
    
    /// Transfer duration
    #[serde(with = "duration_serde")]
    pub duration: Duration,
    
    /// Bandwidth achieved (bytes/sec)
    pub bandwidth: f64,
}

/// GPU kernel execution event
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GpuKernelExecutionEvent {
    #[serde(flatten)]
    pub base: EventBase,
    
    /// Kernel name
    pub kernel_name: String,
    
    /// Work groups
    pub work_groups: [u32; 3],
    
    /// Work group size
    pub work_group_size: [u32; 3],
    
    /// Execution time
    #[serde(with = "duration_serde")]
    pub duration: Duration,
    
    /// Shared memory used
    pub shared_memory_bytes: usize,
}

/// Performance metric event
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PerformanceMetricEvent {
    #[serde(flatten)]
    pub base: EventBase,
    
    /// Metric name
    pub metric_name: String,
    
    /// Metric value
    pub value: f64,
    
    /// Unit of measurement
    pub unit: String,
    
    /// Component that generated the metric
    pub component: String,
    
    /// Additional dimensions
    pub dimensions: HashMap<String, String>,
}

/// Error event
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ErrorEvent {
    #[serde(flatten)]
    pub base: EventBase,
    
    /// Error category
    pub error_category: String,
    
    /// Error message
    pub message: String,
    
    /// Stack trace if available
    pub stack_trace: Option<Vec<String>>,
    
    /// Component where error occurred
    pub component: String,
    
    /// Recovery action taken
    pub recovery_action: Option<String>,
    
    /// Whether the error was recoverable
    pub recoverable: bool,
}

/// Custom event for extensions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CustomEvent {
    #[serde(flatten)]
    pub base: EventBase,
    
    /// Event name
    pub name: String,
    
    /// Event data
    pub data: serde_json::Value,
}

/// Helper module for serializing SystemTime
mod system_time_serde {
    use super::*;
    use serde::{Deserializer, Serializer};
    
    pub fn serialize<S>(time: &SystemTime, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let duration = time.duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default();
        let timestamp = duration.as_secs_f64();
        serializer.serialize_f64(timestamp)
    }
    
    pub fn deserialize<'de, D>(deserializer: D) -> Result<SystemTime, D::Error>
    where
        D: Deserializer<'de>,
    {
        let timestamp = f64::deserialize(deserializer)?;
        let duration = Duration::from_secs_f64(timestamp);
        Ok(SystemTime::UNIX_EPOCH + duration)
    }
}

/// Helper module for serializing Duration
mod duration_serde {
    use super::*;
    use serde::{Deserializer, Serializer};
    
    pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_f64(duration.as_secs_f64())
    }
    
    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let secs = f64::deserialize(deserializer)?;
        Ok(Duration::from_secs_f64(secs))
    }
}

/// Builder for creating events with common fields
pub struct EventBuilder {
    session_id: String,
    correlation_id: Option<String>,
}

impl EventBuilder {
    pub fn new(session_id: String) -> Self {
        Self {
            session_id,
            correlation_id: None,
        }
    }
    
    pub fn with_correlation_id(mut self, correlation_id: String) -> Self {
        self.correlation_id = Some(correlation_id);
        self
    }
    
    pub fn build_base(&self) -> EventBase {
        EventBase {
            event_id: uuid::Uuid::new_v4().to_string(),
            timestamp: SystemTime::now(),
            thread_id: format!("{:?}", std::thread::current().id()),
            correlation_id: self.correlation_id.clone(),
            session_id: self.session_id.clone(),
            metadata: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_event_serialization() {
        let builder = EventBuilder::new("test-session".to_string());
        let event = LogEvent::TaskStart(TaskStartEvent {
            base: builder.build_base(),
            task_name: "test_task".to_string(),
            task_category: "testing".to_string(),
            parameters: HashMap::new(),
            parent_task_id: None,
        });
        
        let json = serde_json::to_string_pretty(&event).unwrap();
        assert!(json.contains("task_start"));
        assert!(json.contains("test_task"));
        
        let deserialized: LogEvent = serde_json::from_str(&json).unwrap();
        matches!(deserialized, LogEvent::TaskStart(_));
    }
}