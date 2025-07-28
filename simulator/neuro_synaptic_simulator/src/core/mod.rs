//! Core execution module for parallel simulation

pub mod scheduler;

pub use scheduler::{
    CoreScheduler,
    SchedulerConfig,
    DistributionStrategy,
    SyncMode,
    WorkUnit,
    SchedulerStats,
};