//! Unit tests for swarm core
//! 
//! This module contains meaningful tests for the core functionality of the swarm system.
//! Tests focus on critical paths and core behaviors rather than edge cases.

// Core unit tests for each module
mod agent_tests;
mod task_tests;
mod topology_tests;
mod error_handling_tests;

// Swarm tests
mod swarm_tests;
mod async_swarm_tests;
mod swarm_trait_tests;

// Integration tests
mod swarm_integration_tests;