//! # IPC Module
//!
//! Provides the core Inter-Process Communication server for Mountain,
//! establishing and managing the bidirectional communication bridge between
//! Mountain's Rust backend and Wind's TypeScript frontend.
//!
//! ## RESPONSIBILITIES
//!
//! ### Core Responsibilities
//! - **Message Routing**: Routes incoming messages from Wind to appropriate handlers
//! - **Connection Management**: Maintains connection health and manages connection pooling
//! - **Security Layer**: Implements permissions, encryption, and audit logging
//! - **Performance Optimization**: Provides message compression and batching
//!
//! ## Module Structure
//!
//! - [`types`]: Shared types used across the IPC subsystem
//! - [`compression`]: Message compression utilities for efficient transfer
//! - [`connection_pool`]: Connection pooling and health monitoring
//! - [`encryption`]: Secure message channel with AES-256-GCM encryption
//! - [`permissions`]: Role-based access control (RBAC) and permission management
//! - [`ipc_server`]: Core IPC server implementation
//! - [`commands`]: Tauri command handlers for IPC operations

pub mod commands;
pub mod compression;
pub mod connection_pool;
pub mod encryption;
pub mod ipc_server;
pub mod permissions;
pub mod types;
