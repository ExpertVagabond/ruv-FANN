//! Simple agent tests that match the actual API

use crate::agent::*;
use crate::task::{Task, TaskResult, TaskStatus};
use crate::error::Result;

#[cfg(test)]
use crate::MockAgent;

#[tokio::test]
async fn test_mock_agent_creation() {
    let agent = MockAgent::new("test-agent");
    assert_eq!(agent.id(), "test-agent");
    assert_eq!(agent.status(), AgentStatus::Idle);
}

#[tokio::test]
async fn test_mock_agent_start_shutdown() {
    let mut agent = MockAgent::new("test-agent");
    
    agent.start().await.unwrap();
    assert_eq!(agent.status(), AgentStatus::Running);
    
    agent.shutdown().await.unwrap();
    assert_eq!(agent.status(), AgentStatus::Offline);
}

#[tokio::test]
async fn test_mock_agent_capabilities() {
    let capabilities = vec![
        Capability::new("compute", "1.0"),
        Capability::new("analyze", "2.0"),
    ];
    
    let agent = MockAgent::new("test-agent")
        .with_capabilities(capabilities);
    
    assert_eq!(agent.capabilities().len(), 2);
    assert_eq!(agent.capabilities()[0].name, "compute");
    assert_eq!(agent.capabilities()[1].name, "analyze");
}

#[tokio::test]
async fn test_mock_agent_can_handle_task() {
    let agent = MockAgent::new("test-agent")
        .with_capabilities(vec![
            Capability::new("compute", "1.0"),
        ]);
    
    let task1 = Task::new("task-1", "compute")
        .require_capability("compute");
    assert!(agent.can_handle(&task1));
    
    let task2 = Task::new("task-2", "analyze")
        .require_capability("analyze");
    assert!(!agent.can_handle(&task2));
}

#[tokio::test]
async fn test_mock_agent_process_task() {
    let mut agent = MockAgent::new("test-agent")
        .with_process_result(Ok(TaskResult::success("Test output")));
    
    let task = Task::new("task-1", "compute");
    let result = agent.process(task).await.unwrap();
    
    assert_eq!(result.status, TaskStatus::Completed);
    assert!(result.output.is_some());
}

#[test]
fn test_capability_creation() {
    let cap = Capability::new("neural-processing", "2.0");
    assert_eq!(cap.name, "neural-processing");
    assert_eq!(cap.version, "2.0");
}

#[test]
fn test_capability_equality() {
    let cap1 = Capability::new("compute", "1.0");
    let cap2 = Capability::new("compute", "1.0");
    let cap3 = Capability::new("compute", "2.0");
    
    assert_eq!(cap1, cap2);
    assert_ne!(cap1, cap3);
}

#[test]
fn test_agent_status_values() {
    let statuses = vec![
        AgentStatus::Idle,
        AgentStatus::Running,
        AgentStatus::Busy,
        AgentStatus::Offline,
        AgentStatus::Error,
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
fn test_agent_metadata_default() {
    let metadata = AgentMetadata::default();
    assert_eq!(metadata.name, "Unknown");
    assert_eq!(metadata.version, "0.0.0");
    assert!(!metadata.description.is_empty());
}

#[test]
fn test_cognitive_patterns() {
    let patterns = CognitivePattern::all();
    assert!(patterns.len() >= 6);
    
    // Test complement relationships
    assert_eq!(CognitivePattern::Convergent.complement(), CognitivePattern::Divergent);
    assert_eq!(CognitivePattern::Divergent.complement(), CognitivePattern::Convergent);
}