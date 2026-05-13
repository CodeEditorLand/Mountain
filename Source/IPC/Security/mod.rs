//! Role-based access control and security auditing for IPC operations.
//! Validates permissions for all incoming IPC messages and logs security
//! events for audit trails.

pub mod PermissionManager;

pub mod Role;

pub mod Permission;

// Note: Consumers should use Security::PermissionManager::PermissionManager
// This avoids naming conflicts between module name and type name
