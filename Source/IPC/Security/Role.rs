//! # Role Definition (IPC Security)
//!
//! ## RESPONSIBILITIES
//! This module defines the Role structure used for role-based access control
//! (RBAC) in the IPC layer.
//!
//! ## ARCHITECTURAL ROLE
//! This module provides the role definition that groups permissions for
//! assignment to users.
//!
//! ## KEY COMPONENTS
//!
//! - **Role**: Role definition with name, permissions, and description
//!
//! ## ERROR HANDLING
//! N/A - This is a data definition module.
//!
//! ## LOGGING
//! N/A - Role creation is logged by PermissionManager.
//!
//! ## PERFORMANCE CONSIDERATIONS
//! - Role definitions are stored in HashMap for O(1) lookup
//! - Permissions are stored as Vec<String> for iteration
//!
//! ## TODO
//! - Add role inheritance support
//! - Implement role activation/deactivation
//! - Add role metadata (creation time, last modified)
//! - Support role aliases

use serde::{Deserialize, Serialize};

/// Role definition for RBAC
///
/// Roles are collections of permissions that can be assigned to users.
/// They provide a convenient way to manage access control by grouping
/// related permissions together.
///
/// ## Role Hierarchy
///
/// ```text
/// admin (full access)
///     |
///     ├── developer (read/write files, read config)
///     └── user (read-only access)
/// ```
///
/// ## Example Usage
///
/// ```rust,ignore
/// let role = Role {
///     name: "editor".to_string(),
///     permissions: vec![
///         "file.read".to_string(),
///         "file.write".to_string(),
///         "storage.read".to_string(),
///     ],
///     description: "Editor role with file access".to_string(),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Role {
	/// Unique role identifier
	pub name: String,

	/// List of permissions granted by this role
	pub permissions: Vec<String>,

	/// Human-readable description of the role
	pub description: String,
}

impl Role {
	/// Create a new role
	pub fn new(name: String, permissions: Vec<String>, description: String) -> Self {
		Self {
			name,
			permissions,
			description,
		}
	}

	/// Check if role has a specific permission
	pub fn has_permission(&self, permission: &str) -> bool {
		self.permissions.iter().any(|p| p == permission)
	}

	/// Add a permission to the role
	pub fn add_permission(&mut self, permission: String) {
		if !self.has_permission(&permission) {
			self.permissions.push(permission);
		}
	}

	/// Remove a permission from the role
	pub fn remove_permission(&mut self, permission: &str) {
		self.permissions.retain(|p| p != permission);
	}

	/// Get the count of permissions
	pub fn permission_count(&self) -> usize {
		self.permissions.len()
	}
}

#[cfg(test)]
mod tests {
	use super::*;

#[test]
	fn test_role_creation() {
		let role = Role::new(
			"test".to_string(),
			vec!["permission1".to_string(), "permission2".to_string()],
			"Test role".to_string(),
		);

		assert_eq!(role.name, "test");
		assert_eq!(role.permission_count(), 2);
	}

#[test]
	fn test_has_permission() {
		let role = Role::new(
			"test".to_string(),
			vec!["permission1".to_string(), "permission2".to_string()],
			"Test role".to_string(),
		);

		assert!(role.has_permission("permission1"));
		assert!(!role.has_permission("permission3"));
	}

#[test]
	fn test_add_permission() {
		let mut role = Role::new(
			"test".to_string(),
			vec!["permission1".to_string()],
			"Test role".to_string(),
		);

		role.add_permission("permission2".to_string());
		assert_eq!(role.permission_count(), 2);

		// Adding duplicate should not increase count
		role.add_permission("permission1".to_string());
		assert_eq!(role.permission_count(), 2);
	}

#[test]
	fn test_remove_permission() {
		let mut role = Role::new(
			"test".to_string(),
			vec!["permission1".to_string(), "permission2".to_string()],
			"Test role".to_string(),
		);

		role.remove_permission("permission1");
		assert_eq!(role.permission_count(), 1);
		assert!(!role.has_permission("permission1"));

		// Removing non-existent permission should not cause issues
		role.remove_permission("permission3");
		assert_eq!(role.permission_count(), 1);
	}
}
