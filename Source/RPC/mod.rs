//! # RPC Service Module
//!
//! This module contains the implementation of gRPC service handlers for the
//! Mountain backend. These services handle requests from Cocoon, Wind, and other
//! components via the Vine gRPC protocol.
//!
//! ## Architecture
//!
//! The RPC module implements the service side of the Spine Contract, which defines
//! the communication protocol between Mountain and its sidecars:
//!
//! - **CocoonService**: Handles requests from the Cocoon extension host
//! - **WindowService**: Manages window operations (documents, messages, status bars)
//! - **WorkspaceService**: Handles workspace operations (files, edits, search)
//! - **CommandService**: Manages command registration and execution
//! - **SecretStorageService**: Handles secret storage operations
//!
//! ## Module Structure
//!
//! - [`CocoonService`]: Main service implementation for Cocoon integration
//! - [`WindowService`]: Window and UI operations
//! - [`WorkspaceService`]: File and workspace operations
//! - [`CommandService`]: Command registration and execution
//! - [`SecretStorageService`]: Secure secret storage
//!
//! ## Service Registration
//!
//! Services are registered with the gRPC server in the Vine/Server module and
//! exposed via Tauri IPC for Wind integration.
//!
//! ## Error Handling
//!
//! All service methods return `tonic::Result<T>` for success or `tonic::Status`
//! for errors. Errors are logged before being returned to clients.
//!
//! ## Code Style
//!
//! - Use Rust async functions with `async fn`
//! - Return `tonic::Result<Response<T>>` for success
//! - Return `Err(tonic::Status::...)` for errors
//! - Use proper error logging with `error!` macros
//! - Use `info!`, `debug!` for logging
//! - Include comprehensive Rustdoc comments

// Public sub-modules
pub mod CocoonService;
pub mod WindowService;
pub mod WorkspaceService;
pub mod CommandService;
pub mod SecretStorageService;

// State management modules
pub mod WindowState;
pub mod SecretStorageState;
