//! # Permission Definition (IPC Security)
//!
//! ## RESPONSIBILITIES
//! This module defines the Permission structure used for role-based access control
//! (RBAC) in the IPC layer.
//!
//! ## ARCHITECTURAL ROLE
//! This module provides the permission definition that represents individual
//! access rights that can be granted to roles.
//!
//! ## KEY COMPONENTS
//!
//! - **Permission**: Permission definition with name, description, and category
//!
//! ## ERROR HANDLING
//! N/A - This is a data definition module.
//!
//! ## LOGGING
//! N/A - Permission creation is logged by PermissionManager.
//!
//! ## PERFORMANCE CONSIDERATIONS
//! - Permission definitions are stored in HashMap for O(1) lookup
//! - Minimal memory footprint for efficient storage
//!
//! ## TODO
//! - Add permission metadata (creation time, last used)
//! - Implement permission aliases
//! - Support permission hierarchies (e.g., "file.*" includes all file permissions)

use serde::{Deserialize, Serialize};

/// Permission definition for RBAC
///
/// Permissions represent individual access rights that can be granted to roles.
/// They follow a naming convention of "resource.action" (e.g., "file.read",
/// "config.update") for clear organization.
///
/// ## Permission Categories
///
/// - **file**: File system operations (read, write, delete)
/// - **config**: Configuration management (read, update)
/// - **storage**: Storage operations (read, write)
/// - **system**: System-level operations (external access)
///
/// ## Example Usage
///
/// ```rust,ignore
/// let permission = Permission {
///     name: "file.write".to_string(),
///     description: "Write file operations".to_string(),
///     category: "file".to_string(),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Permission {
	/// Unique permission identifier (e.g., "file.read", "config.update")
	pub name: String,

	/// Human-readable description of what this permission allows
	pub description: String,

	/// Category for groupings (e.g., "file", "config", "storage")
	pub category: String,
}

impl Permission {
	/// Create a new permission
	pub fn new(name: String, description: String, category: String) -> Self {
		Self {
			name,
			description,
			category,
		}
	}

	/// Check if this permission belongs to a specific category
	pub fn is_in_category(&self, category: &str) -> bool {
		self.category == category
	}

	/// Get the resource part of the permission name (before the dot)
	pub fn resource(&self) -> Option<&str> {
		self.name.split('.').next()
	}

	/// Get the action part of the permission name (after the dot)
	pub fn action(&self) -> Option<&str> {
		self.name.split('.').nth(1)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

#[test]
	fn test_permission_creation() {
		let permission = Permission::new(
			"file.read".to_string(),
			"Read file operations".to_string(),
			"file".to_string(),
		);

		assert_eq!(permission.name, "file.read");
		assert_eq!(permission.description, "Read file operations");
		assert_eq!(permission.category, "file");
	}

#[test]
	fn test_is_in_category() {
		let permission = Permission::new(
			"file.read".to_string(),
			"Read file operations".to_string(),
			"file".to_string(),
		);

		assert!(permission.is_in_category("file"));
		assert!(!permission.is_in_category("config"));
	}

#[test]
	fn test_resource() {
		let permission = Permission::new(
			"file.read".to_string(),
			"Read file operations".to_string(),
			"file".to_string(),
		);

		assert_eq!(permission.resource(), Some("file"));
	}

#[test]
	fn test_action() {
		let permission = Permission::new(
			"file.read".to_string(),
			"Read file operations".to_string(),
			"file".to_string(),
		);

		assert_eq!(permission.action(), Some("read"));
	}

#[test]
	fn test_invalid_permission_name() {
		let permission = Permission::new(
			"invalid".to_string(),
			"Invalid permission".to_string(),
			"test".to_string(),
		);

		// Should return None for invalid format
		assert_eq!(permission.resource(), Some("invalid"));
		assert_eq!(permission.action(), None);
	}
}
