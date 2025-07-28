//! Memory management module for parallel neural simulation

pub mod shared;

pub use shared::{
    SharedMemory,
    // LayerMemory,
    PartitionStrategy,
    // PartitionReadGuard,
    // PartitionWriteGuard,
};