//! Simple integration tests that match the actual API

use crate::error::SwarmError;

#[test]
fn test_swarm_error_retriable() {
    let timeout_error = SwarmError::Timeout { duration_ms: 1000 };
    assert!(timeout_error.is_retriable());
    
    let comm_error = SwarmError::CommunicationError { 
        reason: "Network issue".to_string() 
    };
    assert!(comm_error.is_retriable());
    
    let resource_error = SwarmError::ResourceExhausted { 
        resource: "memory".to_string() 
    };
    assert!(resource_error.is_retriable());
    
    // Non-retriable errors
    let agent_error = SwarmError::AgentNotFound { 
        id: "missing".to_string() 
    };
    assert!(!agent_error.is_retriable());
}

#[test]
fn test_custom_error() {
    let error = SwarmError::custom("Test error message");
    assert!(format!("{}", error).contains("Test error message"));
}