//! # ApplicationRunTime (RunTime)
//!
//! ## RESPONSIBILITIES
//!
//! Main runtime struct that provides effect execution and lifecycle management
//! capabilities for the Mountain application. Acts as the central coordinator
//! between the Echo scheduler and application services.
//!
//! ## ARCHITECTURAL ROLE
//!
//! Core execution engine in Mountain's architecture that bridges declarative
//! effect system with high-performance task execution through Echo scheduler.
//!
//! ## KEY COMPONENTS
//!
//! - **ApplicationRunTime**: Main runtime struct holding Scheduler and Environment
//!
//! ## ERROR HANDLING
//!
//! All errors are propagated through Result types with detailed context.
//! Shutdown operations use error recovery to continue cleanup even when
//! individual services fail.
//!
//! ## LOGGING
//!
//! Uses log crate with appropriate severity levels:
//! - `info`: Initialization, successful completion
//! - `debug`: Detailed operation steps
//! - `warn`: Recoverable errors during shutdown
//! - `error`: Failed operations
//!
//! ## PERFORMANCE CONSIDERATIONS
//!
//! - Uses Arc for shared state to minimize cloning
//! - Scheduler handles parallel execution efficiently
//! - Shutdown operations are optimized to complete quickly
//!
//! ## TODO
//!
//! Medium Priority:
//! - [ ] Add effect execution metrics
//! - [ ] Implement effect cancellation support
//! - [ ] Add effect execution tracing
//!
//! Low Priority:
//! - [ ] Add effect prioritization levels beyond Normal
//! - [ ] Implement circuit breaker pattern for failing effects

// --- Main struct definition ---

/// Main runtime struct definition.
pub mod RuntimeStruct;

// --- Re-export main struct ---
pub use RuntimeStruct::ApplicationRunTime;
