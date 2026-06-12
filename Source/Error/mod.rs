//! Local error taxonomy — currently unused.
//!
//! Every Mountain consumer uses `CommonLibrary::Error::CommonError` instead.
//! These files remain to preserve the original error taxonomy; remove or
//! migrate when the strategy is settled.
//!
//! ## Sub-modules
//!
//! - [`ConfigurationError`]: Configuration-related errors
//! - [`CoreError`]: Core error types (ErrorSeverity, ErrorKind, ErrorContext,
//!   MountainError)
//! - [`FileSystemError`]: File system operation errors
//! - [`IPCError`]: Inter-process communication errors
//! - [`ProviderError`]: Capability provider errors
//! - [`ServiceError`]: Service lifecycle errors

/// Configuration read/write/validation errors.
pub mod ConfigurationError;

/// Core error types: severity levels, error categories, context metadata, and
/// the base MountainError type.
pub mod CoreError;

/// File system operation errors.
pub mod FileSystemError;

/// Inter-process communication errors.
pub mod IPCError;

/// Capability provider (file, terminal, document) errors.
pub mod ProviderError;

/// Service lifecycle errors.
pub mod ServiceError;
