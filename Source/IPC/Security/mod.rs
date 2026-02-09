//! # Security Module (IPC)
//!
//! ## RESPONSIBILITIES
//! This module provides role-based access control (RBAC) and security auditing
//! for IPC operations. It validates permissions for all incoming IPC messages
//! and logs security events for audit trails.
//!
//! ## ARCHITECTURAL ROLE
//! This module is the security layer in the IPC architecture, sitting between
//! the message router and handlers to enforce permission policies.
//!
//! ## KEY COMPONENTS
//!
//! - **PermissionManager**: Validates permissions and manages role/permission
//!   definitions
//! - **Role**: Role definitions with associated permissions
//! - **Permission**: Individual permission definitions
//!
//! ## ERROR HANDLING
//! Permission validation returns Result types with descriptive errors for
//! debugging access denials.
//!
//! ## LOGGING
//! Info-level security event logging, debug for permission checks, error for
//! violations.
//!
//! ## PERFORMANCE CONSIDERATIONS
//! - Permission definitions cached in RwLock for fast concurrent access
//! - Role resolution optimized with HashMap lookups
//! - Audit log limited to last 1000 events to prevent memory bloat
//!
//! ## TODO
//! - Add permission caching with TTL
//! - Implement permission inheritance
//! - Add permission alias support
//! - Implement group-based permissions

pub mod PermissionManager;
pub mod Role;
pub mod Permission;

// Note: Consumers should use Security::PermissionManager::PermissionManager
// This avoids naming conflicts between module name and type name
