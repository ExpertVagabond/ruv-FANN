//! Verification pipeline for analyzing and validating logged events
//!
//! This module provides tools for post-processing logged events to verify
//! correctness, performance, and behavior of neural network operations.

use crate::logging::schema::*;
use std::collections::{HashMap, HashSet};
use std::time::Duration;
use serde::{Deserialize, Serialize};

/// Verification rule trait
pub trait VerificationRule: Send + Sync {
    /// Name of the verification rule
    fn name(&self) -> &str;
    
    /// Verify a single event
    fn verify_event(&mut self, event: &LogEvent) -> VerificationResult;
    
    /// Get final verification report
    fn finalize(&self) -> VerificationReport;
}

/// Result of a verification check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationResult {
    /// Whether the verification passed
    pub passed: bool,
    
    /// Severity if failed
    pub severity: Option<VerificationSeverity>,
    
    /// Description of the issue
    pub message: Option<String>,
    
    /// Suggested fix if available
    pub suggested_fix: Option<String>,
}

impl VerificationResult {
    pub fn pass() -> Self {
        Self {
            passed: true,
            severity: None,
            message: None,
            suggested_fix: None,
        }
    }
    
    pub fn fail(severity: VerificationSeverity, message: String) -> Self {
        Self {
            passed: false,
            severity: Some(severity),
            message: Some(message),
            suggested_fix: None,
        }
    }
    
    pub fn with_fix(mut self, fix: String) -> Self {
        self.suggested_fix = Some(fix);
        self
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum VerificationSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

/// Verification report for a rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationReport {
    pub rule_name: String,
    pub total_events: usize,
    pub passed_events: usize,
    pub failed_events: usize,
    pub issues: Vec<VerificationIssue>,
    pub summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationIssue {
    pub event_id: String,
    pub timestamp: f64,
    pub severity: VerificationSeverity,
    pub message: String,
    pub context: HashMap<String, serde_json::Value>,
}

/// Verification pipeline for running multiple rules
pub struct VerificationPipeline {
    rules: Vec<Box<dyn VerificationRule>>,
}

impl VerificationPipeline {
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }
    
    pub fn add_rule(mut self, rule: Box<dyn VerificationRule>) -> Self {
        self.rules.push(rule);
        self
    }
    
    pub fn verify_events(&mut self, events: &[LogEvent]) -> PipelineReport {
        let mut rule_reports = Vec::new();
        
        for event in events {
            for rule in &mut self.rules {
                rule.verify_event(event);
            }
        }
        
        for rule in &self.rules {
            rule_reports.push(rule.finalize());
        }
        
        let overall_status = self.calculate_overall_status(&rule_reports);
        
        PipelineReport {
            total_events: events.len(),
            rule_reports,
            overall_status,
        }
    }
    
    fn calculate_overall_status(&self, reports: &[VerificationReport]) -> VerificationStatus {
        let total_issues: usize = reports.iter().map(|r| r.issues.len()).sum();
        let critical_issues = reports.iter()
            .flat_map(|r| &r.issues)
            .filter(|i| i.severity == VerificationSeverity::Critical)
            .count();
        
        if critical_issues > 0 {
            VerificationStatus::Failed
        } else if total_issues > 0 {
            VerificationStatus::Warning
        } else {
            VerificationStatus::Passed
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineReport {
    pub total_events: usize,
    pub rule_reports: Vec<VerificationReport>,
    pub overall_status: VerificationStatus,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum VerificationStatus {
    Passed,
    Warning,
    Failed,
}

// Built-in verification rules

/// Verifies memory allocation and deallocation balance
pub struct MemoryLeakDetector {
    allocations: HashMap<String, (usize, String)>, // event_id -> (size, type)
    issues: Vec<VerificationIssue>,
    total_events: usize,
}

impl MemoryLeakDetector {
    pub fn new() -> Self {
        Self {
            allocations: HashMap::new(),
            issues: Vec::new(),
            total_events: 0,
        }
    }
}

impl VerificationRule for MemoryLeakDetector {
    fn name(&self) -> &str {
        "Memory Leak Detector"
    }
    
    fn verify_event(&mut self, event: &LogEvent) -> VerificationResult {
        self.total_events += 1;
        
        match event {
            LogEvent::MemoryAlloc(alloc) => {
                self.allocations.insert(
                    alloc.base.event_id.clone(),
                    (alloc.size_bytes, alloc.memory_type.clone())
                );
                VerificationResult::pass()
            },
            LogEvent::MemoryFree(free) => {
                if let Some(alloc_id) = &free.alloc_event_id {
                    if self.allocations.remove(alloc_id).is_none() {
                        let result = VerificationResult::fail(
                            VerificationSeverity::Warning,
                            format!("Free without matching allocation: {}", alloc_id)
                        );
                        self.issues.push(VerificationIssue {
                            event_id: free.base.event_id.clone(),
                            timestamp: free.base.timestamp.duration_since(std::time::UNIX_EPOCH)
                                .unwrap_or_default().as_secs_f64(),
                            severity: VerificationSeverity::Warning,
                            message: result.message.clone().unwrap(),
                            context: HashMap::new(),
                        });
                        result
                    } else {
                        VerificationResult::pass()
                    }
                } else {
                    VerificationResult::pass()
                }
            },
            _ => VerificationResult::pass(),
        }
    }
    
    fn finalize(&self) -> VerificationReport {
        let mut final_issues = self.issues.clone();
        
        // Report remaining allocations as potential leaks
        for (event_id, (size, mem_type)) in &self.allocations {
            final_issues.push(VerificationIssue {
                event_id: event_id.clone(),
                timestamp: 0.0,
                severity: VerificationSeverity::Error,
                message: format!("Potential memory leak: {} bytes of {} memory", size, mem_type),
                context: HashMap::new(),
            });
        }
        
        VerificationReport {
            rule_name: self.name().to_string(),
            total_events: self.total_events,
            passed_events: self.total_events - final_issues.len(),
            failed_events: final_issues.len(),
            issues: final_issues,
            summary: format!("Found {} potential memory leaks", self.allocations.len()),
        }
    }
}

/// Verifies task completion and timing
pub struct TaskCompletionVerifier {
    open_tasks: HashMap<String, TaskStartEvent>,
    issues: Vec<VerificationIssue>,
    total_events: usize,
    task_durations: Vec<(String, Duration)>,
}

impl TaskCompletionVerifier {
    pub fn new() -> Self {
        Self {
            open_tasks: HashMap::new(),
            issues: Vec::new(),
            total_events: 0,
            task_durations: Vec::new(),
        }
    }
}

impl VerificationRule for TaskCompletionVerifier {
    fn name(&self) -> &str {
        "Task Completion Verifier"
    }
    
    fn verify_event(&mut self, event: &LogEvent) -> VerificationResult {
        self.total_events += 1;
        
        match event {
            LogEvent::TaskStart(start) => {
                self.open_tasks.insert(start.base.event_id.clone(), start.clone());
                VerificationResult::pass()
            },
            LogEvent::TaskEnd(end) => {
                if let Some(start) = self.open_tasks.remove(&end.task_start_id) {
                    self.task_durations.push((start.task_name.clone(), end.duration));
                    
                    if end.status == TaskStatus::Failed && end.error.is_none() {
                        let result = VerificationResult::fail(
                            VerificationSeverity::Warning,
                            "Failed task without error message".to_string()
                        ).with_fix("Include error details for failed tasks".to_string());
                        
                        self.issues.push(VerificationIssue {
                            event_id: end.base.event_id.clone(),
                            timestamp: end.base.timestamp.duration_since(std::time::UNIX_EPOCH)
                                .unwrap_or_default().as_secs_f64(),
                            severity: VerificationSeverity::Warning,
                            message: result.message.clone().unwrap(),
                            context: HashMap::new(),
                        });
                        
                        result
                    } else {
                        VerificationResult::pass()
                    }
                } else {
                    let result = VerificationResult::fail(
                        VerificationSeverity::Error,
                        format!("Task end without matching start: {}", end.task_name)
                    );
                    
                    self.issues.push(VerificationIssue {
                        event_id: end.base.event_id.clone(),
                        timestamp: end.base.timestamp.duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default().as_secs_f64(),
                        severity: VerificationSeverity::Error,
                        message: result.message.clone().unwrap(),
                        context: HashMap::new(),
                    });
                    
                    result
                }
            },
            _ => VerificationResult::pass(),
        }
    }
    
    fn finalize(&self) -> VerificationReport {
        let mut final_issues = self.issues.clone();
        
        // Report uncompleted tasks
        for (event_id, task) in &self.open_tasks {
            final_issues.push(VerificationIssue {
                event_id: event_id.clone(),
                timestamp: task.base.timestamp.duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default().as_secs_f64(),
                severity: VerificationSeverity::Error,
                message: format!("Uncompleted task: {}", task.task_name),
                context: HashMap::new(),
            });
        }
        
        VerificationReport {
            rule_name: self.name().to_string(),
            total_events: self.total_events,
            passed_events: self.total_events - final_issues.len(),
            failed_events: final_issues.len(),
            issues: final_issues,
            summary: format!("Found {} uncompleted tasks", self.open_tasks.len()),
        }
    }
}

/// Verifies GPU operation performance
pub struct GpuPerformanceVerifier {
    slow_operations: Vec<(String, Duration, f64)>, // operation, duration, threshold
    issues: Vec<VerificationIssue>,
    total_events: usize,
    thresholds: HashMap<String, Duration>,
}

impl GpuPerformanceVerifier {
    pub fn new() -> Self {
        let mut thresholds = HashMap::new();
        // Default thresholds for common operations
        thresholds.insert("matrix_multiply".to_string(), Duration::from_millis(100));
        thresholds.insert("forward_pass".to_string(), Duration::from_millis(50));
        thresholds.insert("backward_pass".to_string(), Duration::from_millis(100));
        
        Self {
            slow_operations: Vec::new(),
            issues: Vec::new(),
            total_events: 0,
            thresholds,
        }
    }
    
    pub fn with_threshold(mut self, operation: String, threshold: Duration) -> Self {
        self.thresholds.insert(operation, threshold);
        self
    }
}

impl VerificationRule for GpuPerformanceVerifier {
    fn name(&self) -> &str {
        "GPU Performance Verifier"
    }
    
    fn verify_event(&mut self, event: &LogEvent) -> VerificationResult {
        self.total_events += 1;
        
        match event {
            LogEvent::GpuOperation(op) => {
                if let Some(threshold) = self.thresholds.get(&op.operation) {
                    if op.duration > *threshold {
                        let result = VerificationResult::fail(
                            VerificationSeverity::Warning,
                            format!("Slow GPU operation: {} took {:?} (threshold: {:?})",
                                op.operation, op.duration, threshold)
                        ).with_fix("Consider optimizing kernel or reducing workload".to_string());
                        
                        self.slow_operations.push((
                            op.operation.clone(),
                            op.duration,
                            threshold.as_secs_f64()
                        ));
                        
                        self.issues.push(VerificationIssue {
                            event_id: op.base.event_id.clone(),
                            timestamp: op.base.timestamp.duration_since(std::time::UNIX_EPOCH)
                                .unwrap_or_default().as_secs_f64(),
                            severity: VerificationSeverity::Warning,
                            message: result.message.clone().unwrap(),
                            context: {
                                let mut ctx = HashMap::new();
                                ctx.insert("duration_ms".to_string(), 
                                    serde_json::json!(op.duration.as_secs_f64() * 1000.0));
                                ctx.insert("threshold_ms".to_string(), 
                                    serde_json::json!(threshold.as_secs_f64() * 1000.0));
                                ctx
                            },
                        });
                        
                        result
                    } else {
                        VerificationResult::pass()
                    }
                } else {
                    VerificationResult::pass()
                }
            },
            _ => VerificationResult::pass(),
        }
    }
    
    fn finalize(&self) -> VerificationReport {
        VerificationReport {
            rule_name: self.name().to_string(),
            total_events: self.total_events,
            passed_events: self.total_events - self.issues.len(),
            failed_events: self.issues.len(),
            issues: self.issues.clone(),
            summary: format!("Found {} slow GPU operations", self.slow_operations.len()),
        }
    }
}

/// Verifies training convergence patterns
pub struct TrainingConvergenceVerifier {
    epoch_losses: Vec<(usize, f64)>,
    issues: Vec<VerificationIssue>,
    total_events: usize,
    min_improvement_rate: f64,
    max_epochs_without_improvement: usize,
}

impl TrainingConvergenceVerifier {
    pub fn new() -> Self {
        Self {
            epoch_losses: Vec::new(),
            issues: Vec::new(),
            total_events: 0,
            min_improvement_rate: 0.001, // 0.1% minimum improvement
            max_epochs_without_improvement: 10,
        }
    }
}

impl VerificationRule for TrainingConvergenceVerifier {
    fn name(&self) -> &str {
        "Training Convergence Verifier"
    }
    
    fn verify_event(&mut self, event: &LogEvent) -> VerificationResult {
        self.total_events += 1;
        
        match event {
            LogEvent::TrainingEpochEnd(epoch) => {
                self.epoch_losses.push((epoch.epoch, epoch.loss));
                
                // Check for NaN or infinite loss
                if !epoch.loss.is_finite() {
                    let result = VerificationResult::fail(
                        VerificationSeverity::Critical,
                        format!("Non-finite loss detected at epoch {}: {}", epoch.epoch, epoch.loss)
                    ).with_fix("Check learning rate and gradient clipping".to_string());
                    
                    self.issues.push(VerificationIssue {
                        event_id: epoch.base.event_id.clone(),
                        timestamp: epoch.base.timestamp.duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default().as_secs_f64(),
                        severity: VerificationSeverity::Critical,
                        message: result.message.clone().unwrap(),
                        context: HashMap::new(),
                    });
                    
                    return result;
                }
                
                // Check for stagnation
                if self.epoch_losses.len() > self.max_epochs_without_improvement {
                    let recent_losses: Vec<f64> = self.epoch_losses
                        .iter()
                        .rev()
                        .take(self.max_epochs_without_improvement)
                        .map(|(_, loss)| *loss)
                        .collect();
                    
                    let min_loss = recent_losses.iter().fold(f64::INFINITY, |a, &b| a.min(b));
                    let max_loss = recent_losses.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
                    
                    if (max_loss - min_loss) / min_loss < self.min_improvement_rate {
                        let result = VerificationResult::fail(
                            VerificationSeverity::Warning,
                            format!("Training stagnation detected: no significant improvement in {} epochs", 
                                self.max_epochs_without_improvement)
                        ).with_fix("Consider adjusting learning rate or using a scheduler".to_string());
                        
                        self.issues.push(VerificationIssue {
                            event_id: epoch.base.event_id.clone(),
                            timestamp: epoch.base.timestamp.duration_since(std::time::UNIX_EPOCH)
                                .unwrap_or_default().as_secs_f64(),
                            severity: VerificationSeverity::Warning,
                            message: result.message.clone().unwrap(),
                            context: HashMap::new(),
                        });
                        
                        return result;
                    }
                }
                
                VerificationResult::pass()
            },
            _ => VerificationResult::pass(),
        }
    }
    
    fn finalize(&self) -> VerificationReport {
        VerificationReport {
            rule_name: self.name().to_string(),
            total_events: self.total_events,
            passed_events: self.total_events - self.issues.len(),
            failed_events: self.issues.len(),
            issues: self.issues.clone(),
            summary: format!("Analyzed {} training epochs", self.epoch_losses.len()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_memory_leak_detector() {
        let mut detector = MemoryLeakDetector::new();
        let builder = EventBuilder::new("test".to_string());
        
        // Test matching alloc/free
        let alloc = LogEvent::MemoryAlloc(MemoryAllocEvent {
            base: builder.build_base(),
            size_bytes: 1024,
            memory_type: "heap".to_string(),
            purpose: "test".to_string(),
            pool_name: None,
            total_usage_bytes: 1024,
        });
        
        if let LogEvent::MemoryAlloc(ref alloc_event) = alloc {
            let alloc_id = alloc_event.base.event_id.clone();
            assert!(detector.verify_event(&alloc).passed);
            
            let free = LogEvent::MemoryFree(MemoryFreeEvent {
                base: builder.build_base(),
                size_bytes: 1024,
                memory_type: "heap".to_string(),
                alloc_event_id: Some(alloc_id),
                total_usage_bytes: 0,
            });
            
            assert!(detector.verify_event(&free).passed);
        }
        
        let report = detector.finalize();
        assert_eq!(report.issues.len(), 0);
    }
    
    #[test]
    fn test_task_completion_verifier() {
        let mut verifier = TaskCompletionVerifier::new();
        let builder = EventBuilder::new("test".to_string());
        
        let start = LogEvent::TaskStart(TaskStartEvent {
            base: builder.build_base(),
            task_name: "test_task".to_string(),
            task_category: "test".to_string(),
            parameters: HashMap::new(),
            parent_task_id: None,
        });
        
        if let LogEvent::TaskStart(ref start_event) = start {
            let start_id = start_event.base.event_id.clone();
            assert!(verifier.verify_event(&start).passed);
            
            let end = LogEvent::TaskEnd(TaskEndEvent {
                base: builder.build_base(),
                task_start_id: start_id,
                task_name: "test_task".to_string(),
                status: TaskStatus::Success,
                duration: Duration::from_secs(1),
                result: None,
                error: None,
            });
            
            assert!(verifier.verify_event(&end).passed);
        }
        
        let report = verifier.finalize();
        assert_eq!(report.issues.len(), 0);
    }
}