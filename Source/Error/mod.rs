//! # Error Handling System
//!
//! This module provides a centralized error handling framework for Mountain.
//! It eliminates inconsistent error handling across the codebase and provides
//! a consistent, type-safe approach to error management.
//!
//! ## Architecture
//!
//! The Error module is organized into focused, atomic modules:
//!
//! - **CoreError**: Base error types and traits
//! - **IPCError**: IPC-specific error types
//! - **FileSystemError**: File system operation errors
//! - **ConfigurationError**: Configuration management errors
//! - **ServiceError**: Service-related errors
//! - **ProviderError**: Provider-specific errors
//!
//! ## Design Principles
//!
//! 1. **Single Responsibility**: Each error type has one clear category
//! 2. **Reusability**: Common error patterns are shared
//! 3. **Type Safety**: Strong typing prevents common errors
//! 4. **Rich Context**: Errors carry detailed context for debugging
//!
//! ## Example Usage
//!
//! ```rust
//! use crate::Error::{
//! 	CoreError,
//! 	CoreError::{ErrorContext, ErrorSeverity},
//! 	FileSystemError,
//! 	IPCError,
//! };
//!
//! let error = IPCError::ConnectionFailed {
//! 	context:ErrorContext::new("Failed to connect to IPC server"),
//! 	source:None,
//! };
//! ```

pub mod CoreError;
pub mod FileSystemError;
pub mod IPCError;
pub mod ConfigurationError;
pub mod ServiceError;
pub mod ProviderError;

// Re-export commonly used error types from CoreError
pub use CoreError::{ErrorContext, ErrorKind, ErrorSeverity, MountainError};
// Error types are available through their modules: Error::FileSystemError,
// Error::IPCError, etc.
