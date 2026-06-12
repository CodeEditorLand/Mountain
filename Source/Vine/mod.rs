//! # Vine — gRPC Communication Module
//!
//! Bidirectional gRPC communication protocol between Mountain (the main
//! extension host process) and Cocoon (the sidecar for web-based operations).
//!
//! ## Architecture
//!
//! Vine implements two complementary gRPC services:
//!
//! - **MountainService (Cocoon → Mountain)**: Processes requests and
//!   notifications from Cocoon
//! - **CocoonService (Mountain → Cocoon)**: Sends requests and notifications to
//!   Cocoon
//!
//! All RPC messages use Protocol Buffers for serialization.
//!
//! ## Data Flow
//!
//! ```text
//! Cocoon (Sidecar)          Mountain (Extension Host)
//!       │                            │
//!       ├──────────────────────────► │ ProcessCocoonRequest
//!       │  Extension/Query           │ (returns GenericResponse)
//!       ├──────────────────────────► │ SendCocoonNotification
//!       │  Status Updates            │ (returns Empty)
//!       │                            │
//!       │◄───────────────────────────┤ ProcessMountainRequest
//!       │  Webview Operations        │ (returns GenericResponse)
//!       │◄───────────────────────────┤ SendMountainNotification
//!       │  Configuration Changes     │ (returns Empty)
//! ```
//!
//! ## Key Features
//!
//! - Thread-safe client with connection pool
//! - Configurable request timeout per RPC call
//! - Comprehensive error handling with gRPC status conversion
//! - Health checks before RPC calls
//! - Automatic reconnection for transient failures
//!
//! ## Message Constraints
//!
//! - Maximum message size: 4 MB (default tonic limit)
//! - JSON serialization validation on all parameters
//! - Request ID tracking for operation correlation
//! - Method whitelisting for security
//!
//! ## Sub-modules
//!
//! - [`Client`]: gRPC client for connecting to Cocoon services
//! - [`Error`]: Error types for Vine operations
//! - [`Server`]: gRPC server implementations for Mountain services

/// gRPC client for connecting to Cocoon services.
pub mod Client;

/// Error types for Vine operations.
pub mod Error;

/// gRPC server implementations for Mountain services.
pub mod Server;
