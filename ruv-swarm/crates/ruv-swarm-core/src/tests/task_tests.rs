//! Simple task tests that match the actual API

use crate::task::*;
use crate::agent::Capability;

#[test]
fn test_task_creation() {
    let task = Task::new("task-1", "compute");
    
    assert_eq!(task.id.0, "task-1");
    assert_eq!(task.task_type, "compute");
    assert_eq!(task.priority, TaskPriority::Normal);
    assert_eq!(task.retry_count, 0);
    assert_eq!(task.max_retries, 3);
}

#[test]
fn test_task_builder_pattern() {
    let task = Task::new("task-1", "analysis")
        .with_priority(TaskPriority::High)
        .with_timeout(5000)
        .with_payload(TaskPayload::Text("Analyze this data".into()))
        .require_capability("analyze");
    
    assert_eq!(task.priority, TaskPriority::High);
    assert_eq!(task.timeout_ms, Some(5000));
    assert!(matches!(task.payload, TaskPayload::Text(_)));
    assert_eq!(task.required_capabilities.len(), 1);
    assert_eq!(task.required_capabilities[0], "analyze");
}

#[test]
fn test_task_priority_ordering() {
    assert!(TaskPriority::Critical > TaskPriority::High);
    assert!(TaskPriority::High > TaskPriority::Normal);
    assert!(TaskPriority::Normal > TaskPriority::Low);
}

#[test]
fn test_task_retry_logic() {
    let mut task = Task::new("retry-task", "compute");
    
    assert!(task.can_retry());
    assert_eq!(task.retry_count, 0);
    
    task.increment_retry();
    assert_eq!(task.retry_count, 1);
    assert!(task.can_retry());
    
    // Max out retries
    task.increment_retry();
    task.increment_retry();
    assert_eq!(task.retry_count, 3);
    assert!(!task.can_retry());
}

#[test]
fn test_task_result_success() {
    let result = TaskResult::success("Operation completed")
        .with_task_id(TaskId::new("task-1"))
        .with_execution_time(150);
    
    assert_eq!(result.status, TaskStatus::Completed);
    assert!(result.output.is_some());
    assert!(result.error.is_none());
    assert_eq!(result.execution_time_ms, 150);
}

#[test]
fn test_task_result_failure() {
    let result = TaskResult::failure("Operation failed")
        .with_task_id(TaskId::new("task-2"))
        .with_execution_time(75);
    
    assert_eq!(result.status, TaskStatus::Failed);
    assert!(result.output.is_none());
    assert!(result.error.is_some());
    assert_eq!(result.execution_time_ms, 75);
}

#[test]
fn test_task_payload_variants() {
    let empty_payload = TaskPayload::Empty;
    let text_payload = TaskPayload::Text("Hello".into());
    let binary_payload = TaskPayload::Binary(vec![1, 2, 3, 4]);
    let json_payload = TaskPayload::Json(r#"{"key": "value"}"#.into());
    
    match empty_payload {
        TaskPayload::Empty => assert!(true),
        _ => panic!("Expected empty payload"),
    }
    
    match text_payload {
        TaskPayload::Text(s) => assert_eq!(s, "Hello"),
        _ => panic!("Expected text payload"),
    }
    
    match binary_payload {
        TaskPayload::Binary(bytes) => assert_eq!(bytes, vec![1, 2, 3, 4]),
        _ => panic!("Expected binary payload"),
    }
    
    match json_payload {
        TaskPayload::Json(s) => assert!(s.contains("key")),
        _ => panic!("Expected JSON payload"),
    }
}

#[test]
fn test_task_status_transitions() {
    let statuses = vec![
        TaskStatus::Pending,
        TaskStatus::Assigned,
        TaskStatus::Running,
        TaskStatus::Completed,
        TaskStatus::Failed,
        TaskStatus::Cancelled,
        TaskStatus::TimedOut,
    ];
    
    // Verify all statuses are distinct
    for (i, status1) in statuses.iter().enumerate() {
        for (j, status2) in statuses.iter().enumerate() {
            if i == j {
                assert_eq!(status1, status2);
            } else {
                assert_ne!(status1, status2);
            }
        }
    }
}

#[test]
fn test_task_id_display() {
    let task_id = TaskId::new("my-task-123");
    assert_eq!(format!("{}", task_id), "my-task-123");
}

#[test]
fn test_distribution_strategy_default() {
    let strategy = DistributionStrategy::default();
    assert_eq!(strategy, DistributionStrategy::LeastLoaded);
}