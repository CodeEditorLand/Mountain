//! # RuntimeBuild
//!
//! Builds the Echo scheduler for async task execution.
//!
//! ## RESPONSIBILITIES
//!
//! ### Runtime Construction
//! - Create Echo scheduler with work-stealing threads
//! - Configure worker count based on CPU cores
//! - Initialize task queue system
//!
//! ## ARCHITECTURAL ROLE
//!
//! ### Position in Mountain
//! - Early initialization component in Binary subsystem
//! - Provides high-performance task scheduling
//!
//! ### Dependencies
//! - Echo: scheduler library
//!
//! ### Dependents
//! - Fn() main entry point: Uses scheduler for async execution
//!
//! ## SECURITY
//!
//! ### Considerations
//! - No security impact (scheduler construction only)
//!
//! ## PERFORMANCE
//!
//! ### Considerations
//! - Scheduler construction is one-time cost at startup
//! - Work-stealing maximizes CPU utilization

use std::sync::Arc;

use Echo::Scheduler::{Scheduler::Scheduler, SchedulerBuilder::SchedulerBuilder};

/// Build the Echo scheduler for async task execution.
///
/// Creates a multi-threaded work-stealing scheduler with optimal worker count.
/// This is required for all async operations in the application.
///
/// # Returns
///
/// Returns an Arc-wrapped Echo scheduler.
///
/// # Panics
///
/// Panics if scheduler construction fails.
pub fn Build() -> Arc<Scheduler> { Arc::new(SchedulerBuilder::Create().Build()) }
