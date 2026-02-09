//! # Message Module (IPC)
//!
//! ## RESPONSIBILITIES
//! This module provides the core message types and routing infrastructure for
//! the IPC layer. It defines the standard message format used for all
//! communication between Wind (frontend) and Mountain (backend), and handles
//! message routing to appropriate handlers.
//!
//! ## ARCHITECTURAL ROLE
//! This module is part of the IPC communication layer in Mountain's
//! architecture. It sits at the entry point of the IPC system, defining the
//! contract for all IPC messages and managing their distribution to handlers.
//!
//! ## KEY COMPONENTS
//!
//! - **Types**: Core message structures (TauriIPCMessage, ConnectionStatus,
//!   etc.)
//! - **Router**: Message routing logic to dispatch messages to handlers
//!
//! ## ERROR HANDLING
//! All message operations return Result types with descriptive error messages
//! for troubleshooting communication failures.
//!
//! ## LOGGING
//! Trace-level logging for message routing, debug for listener registration,
//! error forFailed operations.
//!
//! ## PERFORMANCE CONSIDERATIONS
//! - Message cloning is minimized where possible
//! - Listener lookups use HashMap for O(1) complexity
//! - Message queuing handles backpressure when disconnected
//!
//! ## TODO
//! - Add message batching support
//! - Implement message priority queuing
//! - Add message validation schemas

pub mod Types;

pub use Types::{ListenerCallback, SimpleConnectionStatus, TauriIPCMessage};
