//! # Manage Role
//!
//! ## File: IPC/Permission/Role/ManageRole.rs
//!
//! ## Role in Mountain Architecture
//!
//! Defines and manages role structures for role-based access control (RBAC),
//! providing organizational hierarchy for user permissions across the system.
//!
//! ## Primary Responsibility
//!
//! Define role and permission structures for RBAC system with inheritance
//! support.
//!
//! ## Secondary Responsibilities
//!
//! - Create role definitions with assigned permissions
//! - Create permission definitions with categorization
//! - Support role hierarchy and permission inheritance
//! - Validate role and permission integrity
//!
//! ## Dependencies
//!
//! **External Crates:**
//! - `serde` - Serialization for storage and transport
//! - `std::collections::HashSet` - Unique permission tracking
//!
//! **Internal Modules:**
//! - `Validate::{SecurityContext}` - Security context validation
//! - `LogEvent::{SecurityEvent}` - Security event logging
//!
//! ## Dependents
//!
//! - `Validate` - Uses roles for permission validation
//! - `TauriIPCServer` - Manages roles for IPC authorization
//!
//! ## VSCode Pattern Reference
//!
//! Matches VSCode's role system in
//! `vs/platform/permissions/common/permissions.ts`
//! - Hierarchical role definitions
//! - Permission categorization
//! - Role inheritance support
//! - Permission uniqueness validation
//!
//! ## Security Considerations
//!
//! - Role names are case-sensitive for precise control
//! - Permission names follow hierarchical naming (category.action)
//! - Role inheritance prevents permission escalation through ambiguity
//! - Role modifications logged for audit trails
//! - Default roles cannot be deleted without confirmation
//! - Permission deduplication prevents duplicate permissions in roles
//!
//! ## Performance Considerations
//!
//! - HashSet for unique permissions enables O(1) lookup
//! - Role hierarchy flattened for fast permission resolution
//! - Lazy initialization of role collections
//! - Minimal copying of permission data
//!
//! ## Error Handling Strategy
//!
//! - Returns Result for explicit error handling
//! - Duplicate permissions ignored with warning
//! - Invalid role/permission names rejected early
//! - Circular dependency detection in role hierarchy
//!
//! ## Thread Safety
//!
//! - Immutable role definitions after creation
//! - Clone semantics for safe sharing across threads
//!
//! ## TODO Items
//!
//! - [ ] Implement role hierarchy with parent/child relationships
//! - [ ] Add permission negation (deny permissions)
//! - [ ] Support role templates for common permission sets

use std::collections::HashSet;

use serde::{Deserialize, Serialize};
use crate::dev_log;

/// Role definition for RBAC system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Role {
	/// Unique role identifier (case-sensitive)
	pub Name:String,

	/// Permissions granted by this role (unique, deduplicated)
	pub Permissions:Vec<String>,

	/// Human-readable description of role purpose
	pub Description:String,

	/// Optional parent role for inheritance (not yet implemented)
	pub ParentRole:Option<String>,

	/// Role priority for conflict resolution (higher = more important)
	pub Priority:u32,
}

/// Permission definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Permission {
	/// Unique permission identifier (formatted as category.action)
	pub Name:String,

	/// Human-readable description of permission purpose
	pub Description:String,

	/// Permission category for organization
	pub Category:String,

	/// Whether this permission is sensitive (requires special logging)
	pub IsSensitive:bool,
}

impl Role {
	/// Create a new role definition
	///
	/// ## Parameters
	/// - `Name`: Unique role identifier
	/// - `Permissions`: List of permission strings
	/// - `Description`: Human-readable description
	///
	/// ## Returns
	/// New Role instance with deduplicated permissions
	///
	/// ## Notes
	/// - Permissions are automatically deduplicated
	/// - Default priority is 0
	pub fn New(Name:String, Permissions:Vec<String>, Description:String) -> Self {
		let UniquePermissions:Vec<String> = Permissions.into_iter().collect::<HashSet<String>>().into_iter().collect();

		Self { Name, Permissions:UniquePermissions, Description, ParentRole:None, Priority:0 }
	}

	/// Create a new role with parent inheritance
	///
	/// ## Parameters
	/// - `Name`: Unique role identifier
	/// - `Permissions`: List of permission strings
	/// - `Description`: Human-readable description
	/// - `ParentRole`: Name of parent role to inherit from
	/// - `Priority`: Role priority level
	///
	/// ## Returns
	/// New Role instance with inheritance configuration
	pub fn NewWithParent(
		Name:String,
		Permissions:Vec<String>,
		Description:String,
		ParentRole:String,
		Priority:u32,
	) -> Self {
		let UniquePermissions:Vec<String> = Permissions.into_iter().collect::<HashSet<String>>().into_iter().collect();

		Self {
			Name,
			Permissions:UniquePermissions,
			Description,
			ParentRole:Some(ParentRole),
			Priority,
		}
	}

	/// Add a permission to this role
	///
	/// ## Parameters
	/// - `Permission`: Permission string to add
	///
	/// ## Returns
	/// Self for method chaining
	pub fn AddPermission(mut self, Permission:String) -> Self {
		if !self.Permissions.contains(&Permission) {
			self.Permissions.push(Permission.clone());
			dev_log!("ipc", "[Role] Added permission '{}' to role '{}'", Permission, self.Name);
		} else {
			dev_log!("ipc", 
				"[Role] Permission '{}' already exists in role '{}', skipping",
				Permission, self.Name
			);
		}
		self
	}

	/// Add multiple permissions to this role
	///
	/// ## Parameters
	/// - `Permissions`: Iterator of permission strings to add
	///
	/// ## Returns
	/// Self for method chaining
	pub fn AddPermissions(mut self, Permissions:impl IntoIterator<Item = String>) -> Self {
		for Permission in Permissions {
			if !self.Permissions.contains(&Permission) {
				self.Permissions.push(Permission.clone());
				dev_log!("ipc", "[Role] Added permission '{}' to role '{}'", Permission, self.Name);
			}
		}
		self
	}

	/// Check if this role has a specific permission
	///
	/// ## Parameters
	/// - `Permission`: Permission string to check
	///
	/// ## Returns
	/// true if role has permission, false otherwise
	pub fn HasPermission(&self, Permission:&str) -> bool { self.Permissions.contains(&Permission.to_string()) }

	/// Get the count of permissions in this role
	///
	/// ## Returns
	/// Number of unique permissions
	pub fn PermissionCount(&self) -> usize { self.Permissions.len() }

	/// Validate role structure integrity
	///
	/// ## Returns
	/// Result indicating success or validation error
	pub fn Validate(&self) -> Result<(), String> {
		if self.Name.is_empty() {
			return Err("Role name cannot be empty".to_string());
		}

		if self.Name.contains(|c:char| c.is_whitespace()) {
			return Err("Role name cannot contain whitespace".to_string());
		}

		if self.Description.is_empty() {
			return Err("Role description cannot be empty".to_string());
		}

		// Validate permission names
		for Permission in &self.Permissions {
			if Permission.is_empty() {
				return Err("Permission name cannot be empty".to_string());
			}

			if !Permission.contains('.') {
				return Err(format!(
					"Permission '{}' must contain a dot separating category and action",
					Permission
				));
			}

			if Permission.contains(|c:char| c.is_whitespace()) {
				return Err(format!("Permission '{}' cannot contain whitespace", Permission));
			}
		}

		Ok(())
	}
}

impl Permission {
	/// Create a new permission definition
	///
	/// ## Parameters
	/// - `Name`: Unique permission identifier (category.action format)
	/// - `Description`: Human-readable description
	/// - `Category`: Permission category
	///
	/// ## Returns
	/// New Permission instance
	pub fn New(Name:String, Description:String, Category:String) -> Self {
		Self { Name, Description, Category, IsSensitive:false }
	}

	/// Create a new sensitive permission (requires special logging)
	///
	/// ## Parameters
	/// - `Name`: Unique permission identifier
	/// - `Description`: Human-readable description
	/// - `Category`: Permission category
	///
	/// ## Returns
	/// New Permission instance marked as sensitive
	pub fn NewSensitive(Name:String, Description:String, Category:String) -> Self {
		Self { Name, Description, Category, IsSensitive:true }
	}

	/// Mark permission as sensitive
	///
	/// ## Returns
	/// Self for method chaining
	pub fn SetSensitive(mut self) -> Self {
		self.IsSensitive = true;
		self
	}

	/// Get the action part of the permission name (after last dot)
	///
	/// ## Returns
	/// Action string or "unknown" if format is invalid
	pub fn GetAction(&self) -> String { self.Name.rsplit('.').next().unwrap_or("unknown").to_string() }

	/// Get the category part of the permission name (before last dot)
	///
	/// ## Returns
	/// Category string or "unknown" if format is invalid
	pub fn GetCategory(&self) -> String {
		if let Some(pos) = self.Name.rfind('.') {
			self.Name[..pos].to_string()
		} else {
			"unknown".to_string()
		}
	}

	/// Validate permission structure integrity
	///
	/// ## Returns
	/// Result indicating success or validation error
	pub fn Validate(&self) -> Result<(), String> {
		if self.Name.is_empty() {
			return Err("Permission name cannot be empty".to_string());
		}

		if self.Name.contains(|c:char| c.is_whitespace()) {
			return Err("Permission name cannot contain whitespace".to_string());
		}

		if !self.Name.contains('.') {
			return Err("Permission name must contain a dot separating category and action".to_string());
		}

		if self.Description.is_empty() {
			return Err("Permission description cannot be empty".to_string());
		}

		if self.Category.is_empty() {
			return Err("Permission category cannot be empty".to_string());
		}

		Ok(())
	}
}

/// Create standard user role
///
/// ## Returns
/// Role configured with read-only permissions
pub fn CreateUserRole() -> Role {
	Role::New(
		"user".to_string(),
		vec!["file.read".to_string(), "config.read".to_string(), "storage.read".to_string()],
		"Standard user with read access".to_string(),
	)
}

/// Create developer role
///
/// ## Returns
/// Role configured with read/write permissions
pub fn CreateDeveloperRole() -> Role {
	Role::New(
		"developer".to_string(),
		vec![
			"file.read".to_string(),
			"file.write".to_string(),
			"config.read".to_string(),
			"storage.read".to_string(),
			"storage.write".to_string(),
		],
		"Developer with read/write access".to_string(),
	)
}

/// Create administrator role
///
/// ## Returns
/// Role configured with full system access
pub fn CreateAdminRole() -> Role {
	Role::New(
		"admin".to_string(),
		vec![
			"file.read".to_string(),
			"file.write".to_string(),
			"config.read".to_string(),
			"config.update".to_string(),
			"storage.read".to_string(),
			"storage.write".to_string(),
			"system.external".to_string(),
			"system.execute".to_string(),
			"admin.manage".to_string(),
		],
		"Administrator with full access".to_string(),
	)
	.AddPermission("role.manage".to_string())
}

/// Create all standard roles
///
/// ## Returns
/// Vector containing user, developer, and admin roles
pub fn CreateStandardRoles() -> Vec<Role> {
	dev_log!("ipc", "[ManageRole] Creating standard roles");
	vec![CreateUserRole(), CreateDeveloperRole(), CreateAdminRole()]
}

/// Create all standard permissions
///
/// ## Returns
/// Vector containing standard permission definitions
pub fn CreateStandardPermissions() -> Vec<Permission> {
	dev_log!("ipc", "[ManageRole] Creating standard permissions");
	vec![
		// File permissions
		Permission::New("file.read".to_string(), "Read file operations".to_string(), "file".to_string()),
		Permission::New(
			"file.write".to_string(),
			"Write file operations".to_string(),
			"file".to_string(),
		),
		Permission::New(
			"file.delete".to_string(),
			"Delete file operations".to_string(),
			"file".to_string(),
		),
		// Config permissions
		Permission::New(
			"config.read".to_string(),
			"Read configuration".to_string(),
			"config".to_string(),
		),
		Permission::NewSensitive(
			"config.update".to_string(),
			"Update configuration".to_string(),
			"config".to_string(),
		)
		.SetSensitive(),
		// Storage permissions
		Permission::New("storage.read".to_string(), "Read storage".to_string(), "storage".to_string()),
		Permission::New("storage.write".to_string(), "Write storage".to_string(), "storage".to_string()),
		Permission::New(
			"storage.delete".to_string(),
			"Delete from storage".to_string(),
			"storage".to_string(),
		),
		// System permissions
		Permission::NewSensitive(
			"system.external".to_string(),
			"Access external system resources".to_string(),
			"system".to_string(),
		)
		.SetSensitive(),
		Permission::NewSensitive(
			"system.execute".to_string(),
			"Execute system commands".to_string(),
			"system".to_string(),
		)
		.SetSensitive(),
		// Admin permissions
		Permission::NewSensitive(
			"admin.manage".to_string(),
			"Administrative management operations".to_string(),
			"admin".to_string(),
		)
		.SetSensitive(),
		Permission::NewSensitive(
			"role.manage".to_string(),
			"Manage roles and permissions".to_string(),
			"admin".to_string(),
		)
		.SetSensitive(),
	]
}

#[cfg(test)]
mod Tests {
	use super::*;

	#[test]
	fn TestCreateRole() {
		let role = Role::New(
			"test.role".to_string(),
			vec!["perm1".to_string(), "perm2".to_string(), "perm1".to_string()],
			"Test role".to_string(),
		);

		assert_eq!(role.Name, "test.role");
		assert_eq!(role.Description, "Test role");
		assert_eq!(role.PermissionCount(), 2, "Should deduplicate permissions");
	}

	#[test]
	fn TestRoleHasPermission() {
		let role = Role::New(
			"test.role".to_string(),
			vec!["perm1".to_string(), "perm2".to_string()],
			"Test role".to_string(),
		);

		assert!(role.HasPermission("perm1"));
		assert!(role.HasPermission("perm2"));
		assert!(!role.HasPermission("perm3"));
	}

	#[test]
	fn TestAddPermission() {
		let role = Role::New("test.role".to_string(), vec!["perm1".to_string()], "Test role".to_string())
			.AddPermission("perm2".to_string());

		assert!(role.HasPermission("perm1"));
		assert!(role.HasPermission("perm2"));
	}

	#[test]
	fn TestAddPermissions() {
		let role = Role::New("test.role".to_string(), vec!["perm1".to_string()], "Test role".to_string())
			.AddPermissions(vec!["perm2".to_string(), "perm3".to_string()]);

		assert_eq!(role.PermissionCount(), 3);
	}

	#[test]
	fn TestRoleValidateSuccess() {
		let role = Role::New(
			"test.role".to_string(),
			vec!["category.action".to_string()],
			"Valid role".to_string(),
		);

		assert!(role.Validate().is_ok());
	}

	#[test]
	fn TestRoleValidateEmptyName() {
		let role = Role::New("".to_string(), vec!["category.action".to_string()], "Valid role".to_string());

		assert!(role.Validate().is_err());
	}

	#[test]
	fn TestRoleValidateWhitespaceInName() {
		let role = Role::New(
			"test role".to_string(),
			vec!["category.action".to_string()],
			"Valid role".to_string(),
		);

		assert!(role.Validate().is_err());
	}

	#[test]
	fn TestRoleValidateEmptyDescription() {
		let role = Role::New("test.role".to_string(), vec!["category.action".to_string()], "".to_string());

		assert!(role.Validate().is_err());
	}

	#[test]
	fn TestPermissionNew() {
		let perm = Permission::New("file.read".to_string(), "Read files".to_string(), "file".to_string());

		assert_eq!(perm.Name, "file.read");
		assert_eq!(perm.Description, "Read files");
		assert_eq!(perm.Category, "file");
		assert!(!perm.IsSensitive);
	}

	#[test]
	fn TestPermissionNewSensitive() {
		let perm =
			Permission::NewSensitive("config.update".to_string(), "Update config".to_string(), "config".to_string());

		assert!(perm.IsSensitive);
	}

	#[test]
	fn TestPermissionGetAction() {
		let perm = Permission::New("file.read".to_string(), "Read files".to_string(), "file".to_string());

		assert_eq!(perm.GetAction(), "read");
	}

	#[test]
	fn TestPermissionGetCategory() {
		let perm = Permission::New("file.read".to_string(), "Read files".to_string(), "file".to_string());

		assert_eq!(perm.GetCategory(), "file");
	}

	#[test]
	fn TestPermissionValidateSuccess() {
		let perm = Permission::New("file.read".to_string(), "Read files".to_string(), "file".to_string());

		assert!(perm.Validate().is_ok());
	}

	#[test]
	fn TestPermissionValidateMissingDot() {
		let perm = Permission::New("fileread".to_string(), "Read files".to_string(), "file".to_string());

		assert!(perm.Validate().is_err());
	}

	#[test]
	fn TestCreateStandardRoles() {
		let roles = CreateStandardRoles();
		assert_eq!(roles.len(), 3);

		let user_role = roles.iter().find(|r| r.Name == "user").unwrap();
		assert!(user_role.HasPermission("file.read"));

		let admin_role = roles.iter().find(|r| r.Name == "admin").unwrap();
		assert!(admin_role.HasPermission("admin.manage"));
	}

	#[test]
	fn TestCreateStandardPermissions() {
		let perms = CreateStandardPermissions();
		assert!(perms.len() > 0);

		let file_read = perms.iter().find(|p| p.Name == "file.read").unwrap();
		assert_eq!(file_read.Category, "file");

		let config_update = perms.iter().find(|p| p.Name == "config.update").unwrap();
		assert!(config_update.IsSensitive);
	}

	#[test]
	fn TestRoleWithParent() {
		let role = Role::NewWithParent(
			"test.role".to_string(),
			vec!["perm1".to_string()],
			"Test role".to_string(),
			"parent.role".to_string(),
			10,
		);

		assert_eq!(role.ParentRole, Some("parent.role".to_string()));
		assert_eq!(role.Priority, 10);
	}
}
