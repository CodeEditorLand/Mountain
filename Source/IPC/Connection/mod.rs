//! # Connection Module (IPC)
//!
//! ## RESPONSIBILITIES
//! This module provides connection management and health monitoring for IPC
//! operations. It manages connection pooling, tracks connection health, and
//! provides statistics for monitoring.
//!
//! ## ARCHITECTURAL ROLE
//! This module is part of the infrastructure layer in the IPC architecture,
//! providing connection lifecycle management and health monitoring.
//!
//! ## KEY COMPONENTS
//!
//! - **Manager**: Connection pool and lifecycle management
//! - **Types**: Connection data structures (ConnectionHandle, ConnectionStats)
//! - **Health**: Connection health checking and monitoring
//!
//! ## ERROR HANDLING
//! All operations return Result types with descriptive error messages for
//! connection failures.
//!
//! ## LOGGING
//! Debug-level logging for connection lifecycle, error for failures.
//!
//! ## PERFORMANCE CONSIDERATIONS
//! - Connection pooling prevents resource exhaustion
//! - Health monitoring runs in background tasks
//! - Stale connection cleanup prevents memory leaks
//! - Semaphore-based connection limiting
//!
//! ## TODO
//! - Add connection timeout handling
//! - Implement connection retry logic
//! - Add connection metrics dashboard
//! - Support multiple connection types

pub mod Manager;
pub mod Types;
pub mod Health;

pub use Manager::{ConnectionManager, ConnectionPool};
pub use Types::{ConnectionHandle, ConnectionStats, ConnectionStatus};
pub use Health::HealthChecker;
