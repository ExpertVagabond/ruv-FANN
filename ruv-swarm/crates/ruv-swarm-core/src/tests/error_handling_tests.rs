//! Simple error handling tests that match the actual API

use crate::error::*;

#[test]
fn test_swarm_error_variants() {
    // Test that all error variants can be created correctly
    let agent_not_found = SwarmError::AgentNotFound { 
        id: "test-agent".to_string() 
    };
    
    let task_execution_failed = SwarmError::TaskExecutionFailed { 
        reason: "test task failed".to_string() 
    };
    
    let invalid_topology = SwarmError::InvalidTopology { 
        reason: "Test topology error".to_string() 
    };
    
    let resource_exhausted = SwarmError::ResourceExhausted { 
        resource: "memory".to_string(),
    };
    
    let communication_error = SwarmError::CommunicationError { 
        reason: "Connection failed".to_string() 
    };
    
    let serialization_error = SwarmError::SerializationError { 
        reason: "Failed to serialize data".to_string() 
    };
    
    let timeout_error = SwarmError::Timeout { 
        duration_ms: 5000,
    };
    
    let capability_mismatch = SwarmError::CapabilityMismatch { 
        agent_id: "agent-1".to_string(),
        capability: "compute".to_string(),
    };
    
    let strategy_error = SwarmError::StrategyError { 
        reason: "Strategy failed".to_string() 
    };
    
    let custom_error = SwarmError::Custom("Custom error message".to_string());
    
    // Verify all errors are distinct
    let errors = vec![
        agent_not_found,
        task_execution_failed,
        invalid_topology,
        resource_exhausted,
        communication_error,
        serialization_error,
        timeout_error,
        capability_mismatch,
        strategy_error,
        custom_error,
    ];
    
    for (i, error1) in errors.iter().enumerate() {
        for (j, error2) in errors.iter().enumerate() {
            if i != j {
                // Using debug format for comparison since SwarmError implements PartialEq
                assert_ne!(error1, error2);
            }
        }
    }
}

#[test]
fn test_swarm_error_display() {
    let error = SwarmError::AgentNotFound { 
        id: "missing-agent".to_string() 
    };
    
    let error_string = error.to_string();
    assert!(error_string.contains("missing-agent"));
}

#[test]
fn test_swarm_error_retriable() {
    // Test which errors are retriable
    let retriable_error = SwarmError::CommunicationError { 
        reason: "Temporary network issue".to_string() 
    };
    assert!(retriable_error.is_retriable());
    
    let timeout_error = SwarmError::Timeout { duration_ms: 5000 };
    assert!(timeout_error.is_retriable());
    
    let resource_error = SwarmError::ResourceExhausted { 
        resource: "memory".to_string() 
    };
    assert!(resource_error.is_retriable());
    
    // Non-retriable errors
    let config_error = SwarmError::InvalidTopology { 
        reason: "Invalid configuration".to_string() 
    };
    assert!(!config_error.is_retriable());
}

#[test]
fn test_custom_error_creation() {
    let error = SwarmError::custom("Custom message");
    match error {
        SwarmError::Custom(msg) => assert_eq!(msg, "Custom message"),
        _ => panic!("Expected custom error"),
    }
}

#[test]
fn test_agent_error() {
    let swarm_error = SwarmError::AgentNotFound { 
        id: "test-agent".to_string() 
    };
    
    let agent_error = AgentError {
        agent_id: "test-agent".to_string(),
        error: swarm_error,
    };
    
    assert_eq!(agent_error.agent_id, "test-agent");
    assert!(format!("{}", agent_error).contains("test-agent"));
}