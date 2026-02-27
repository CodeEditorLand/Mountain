//! # IPC Common Abstractions
//!
//! This module provides shared types and abstractions used across the IPC
//! layer. It eliminates code duplication and provides a consistent foundation
//! for all IPC communication components.
//!
//! ## Architecture
//!
//! The Common module is organized into focused, atomic modules:
//!
//! - **MessageType**: Core message structures for IPC communication
//! - **ConnectionStatus**: Connection health and state tracking
//! - **HealthStatus**: Health monitoring and scoring
//! - **PerformanceMetrics**: Performance measurement and tracking
//! - **ServiceInfo**: Service discovery and information
//!
//! ## Design Principles
//!
//! 1. **Single Responsibility**: Each module has one clear purpose
//! 2. **Reusability**: Types are shared across IPC components
//! 3. **Type Safety**: Strong typing prevents common errors
//! 4. **Serde Support**: All types support serialization for IPC
//!
//! ## Example Usage
//!
//! ```rust
//! use crate::IPC::Common::{ConnectionStatus, HealthStatus, PerformanceMetrics};
//!
//! let status = ConnectionStatus::Connected;
//! let health = HealthStatus::new(100);
//! let metrics = PerformanceMetrics::default();
//! ```

pub mod ConnectionStatus;
pub mod HealthStatus;
pub mod MessageType;
pub mod PerformanceMetrics;
pub mod ServiceInfo;

// Re-export commonly used types (use module prefix to avoid naming conflicts)
pub use ConnectionStatus::ConnectionState;
pub use HealthStatus::{HealthIssue, HealthMonitor, SeverityLevel};
pub use MessageType::{IPCCommand, IPCMessage, IPCResponse};
pub use PerformanceMetrics::ThroughputMetrics;
pub use ServiceInfo::{ServiceRegistry, ServiceState};
// Re-exports for struct types (using module prefix)
pub use ConnectionStatus::ConnectionStatus as Status;
// Note: PerformanceMetrics and ServiceInfo are modules, not types - use
// directly
