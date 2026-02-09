//! # TauriIPCServer Module (IPC)
//!
//! ## RESPONSIBILITIES
//! This module provides the main IPC server orchestrator for Mountain,
//! establishing and managing the bidirectional communication bridge between
//! Mountain's Rust backend and Wind's TypeScript frontend.
//!
//! ## ARCHITECTURAL ROLE
//! This module is the core of the IPC layer, orchestrating all IPC operations
//! and coordinating between submodules.
//!
//! ## KEY COMPONENTS
//!
//! - **Server**: Main TauriIPCServer orchestrator
//!
//! ## ERROR HANDLING
//! All operations return Result types with descriptive error messages.
//!
//! ## LOGGING
//! Info-level logging for lifecycle events, debug for operations, error for
//! failures.
//!
//! ## PERFORMANCE CONSIDERATIONS
//! - Async/await for non-blocking operations
//! - Message queuing for offline scenarios
//! - Health monitoring for reliability
//!
//! ## TODO
//! - Add message priority queuing
//! - Implement connection retry logic
//! - Add message persistence for offline mode
//! - Support multiple transport protocols

pub mod Server;

pub use Server::TauriIPCServer;
