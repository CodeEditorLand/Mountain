//! # Vine gRPC Module
//!
//! This module encapsulates all logic related to the gRPC-based

//! Inter-Process Communication (IPC) system, codenamed "Vine".
//!
//! ## Architecture Overview
//!
//! Vine implements a bidirectional gRPC communication protocol between:
//! - **Mountain**: The main VS Code extension host process
//! - **Cocoon**: The sidecar process handling web-based operations
//!
//! The system uses two complementary gRPC services:
//!
//! ### MountainService (Cocoon → Mountain)
//! - **ProcessCocoonRequest**: Request-response pattern for Cocoon to query
//!   Mountain
//! - **SendCocoonNotification**: Fire-and-forget notifications from Cocoon to
//!   Mountain
//! - **CancelOperation**: Request cancellation of long-running operations
//!
//! ### CocoonService (Mountain → Cocoon)
//! - **ProcessMountainRequest**: Request-response pattern for Mountain to query
//!   Cocoon
//! - **SendMountainNotification**: Fire-and-forget notifications from Mountain
//!   to Cocoon
//! - **CancelOperation**: Request cancellation of long-running operations
//!
//! ## Communication Protocol
//!
//! All RPC messages use Protocol Buffers for serialization:
//! - **GenericRequest**: Contains request ID, method name, and JSON parameters
//! - **GenericResponse**: Contains request ID, JSON result, or optional error
//! - **GenericNotification**: Fire-and-forget message with method name and JSON
//!   parameters
//! - **RpcError**: JSON-RPC compliant error structure with code, message, and
//!   optional data
//!
//! ## Data Flow
//!
//! ```text
//! Cocoon (Sidecar)          Mountain (Extension Host)
//!       │                            │
//!       ├──────────────────────────►│ ProcessCocoonRequest
//!       │  Extension/Query           │ (returns GenericResponse)
//!       │                            │
//!       ├──────────────────────────►│ SendCocoonNotification
//!       │  Status Updates            │ (returns Empty)
//!       │                            │
//!       │◄───────────────────────────┤ ProcessMountainRequest
//!       │  Webview Operations         │ (returns GenericResponse)
//!       │                            │
//!       │◄───────────────────────────┤ SendMountainNotification
//!       │  Configuration Changes      │ (returns Empty)
//!       │                            │
//!       ◄═════════════════════════════╡ CancelOperation
//!                              Cancels either process
//! ```
//!
//! ## Key Features
//!
//! - **Thread-Safe Client**: Connection pool with Arc<Mutex<>> for concurrent
//!   access
//! - **Request Timeout**: Configurable timeout per RPC call
//! - **Error Handling**: Comprehensive error types with gRPC status conversion
//! - **Graceful Degradation**: System continues when Cocoon is unavailable
//! - **Health Checks**: Connection validation before RPC calls
//! - **Retry Logic**: Automatic reconnection attempts for transient failures
//!
//! ## Message Validation
//!
//! - Maximum message size: 4MB (default tonic limit)
//! - JSON serialization validation on all parameters
//! - Request ID tracking for operation correlation
//! - Method whitelisting for security
//!
//! ## Modules
//!
//! - [`self::Client`]: gRPC client for connecting to Cocoon services
//! - [`self::Error`]: Comprehensive error types for Vine operations
//! - [`self::Generated`]: Auto-generated protobuf code from Vine.proto
//! - [`self::Server`]: gRPC server implementations for Mountain services

pub mod Server;
