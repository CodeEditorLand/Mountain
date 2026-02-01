//! # Vine gRPC Server Module
//!
//! This module contains the implementation for the Mountain gRPC server. It is
//! responsible for listening for incoming connections from sidecars like
//! `Cocoon`, handling RPC requests, and dispatching them into the Mountain
//! application logic.
//!
//! ## Architecture
//!
//! The Vine server implements two complementary gRPC services:
//!
//! ### MountainService (Listens on one port)
//! - **ProcessCocoonRequest**: Handles request-response calls from Cocoon
//! - **SendCocoonNotification**: Processes fire-and-forget notifications from
//!   Cocoon
//! - **CancelOperation**: Cancels long-running operations requested by Cocoon
//!
//! ### CocoonService (Listens on separate port)
//! - **ProcessMountainRequest**: Handles request-response calls from Mountain
//!   to Cocoon
//! - **SendMountainNotification**: Processes notifications from Mountain to
//!   Cocoon
//! - **CancelOperation**: Cancels operations in Cocoon
//!
//! ## Lifecycle Management
//!
//! 1. **Initialization**: Servers are spawned as background tasks via
//!    `Initialize::Initialize`
//! 2. **Service Registration**: gRPC services are registered with tonic's
//!    Server builder
//! 3. **Request Handling**: Each RPC call is dispatched to appropriate handlers
//! 4. **Graceful Shutdown**: Servers terminate when tokio runtime is shut down
//!
//! ## Data Flow
//!
//! ```text
//! Cocoon Sidecar                          Mountain Extension Host
//!      │                                          │
//!      │  ┌──────────────────────────────────►    │
//!      │  │ MountainService::ProcessCocoonRequest │
//!      │  │ (extensions, queries, state)          │
//!      │  ◄───────────────────────────────────    │
//!      │                                          │
//!      │  ┌──────────────────────────────────►    │
//!      │  │ MountainService::SendCocoonNotification
//!      │  │ (status updates, events)              │
//!      │  ◄───────────────────────────────────    │
//!      │                                          │
//!      │  ◄────────────────────────────────────── │
//!      │  │ CocoonService::ProcessMountainRequest │
//!      │  │ (WebView operations, IPC)             │
//!      │  ┌──────────────────────────────────►    │
//!      │                                          │
//!      │  ◄────────────────────────────────────── │
//!      │  │ CocoonService::SendMountainNotification
//!      │  │ (config changes, commands)             │
//!      │  ┌──────────────────────────────────►    │
//!      │                                          │
//! ```
//!
//! ## Error Handling
//!
//! - Request validation before processing
//! - Comprehensive error conversion to tonic::Status
//! - Detailed logging of all errors
//! - Graceful error responses to clients
//!
//! ## Security Considerations
//!
//! - Request size limits (4MB default)
//! - Method whitelisting (prevents arbitrary method calls)
//! - Parameter validation before processing
//! - Safe error messages (no sensitive data leakage)
//!
//! ## Modules
//!
//! - [`Initialize`]: Server initialization and startup logic
//! - [`MountainVinegRPCService`]: Implementation of MountainService (Cocoon →
//!   Mountain)
//! - [`CocoonServiceImpl`]: Implementation of CocoonService (Mountain → Cocoon)

#![allow(non_snake_case)]

pub mod Initialize;

pub mod MountainVinegRPCService;

pub mod CocoonServiceImpl;
