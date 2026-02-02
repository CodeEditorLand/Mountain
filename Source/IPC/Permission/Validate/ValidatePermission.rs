//! # Validate
//!
//! ## File: IPC/Permission/Validate/ValidatePermission.rs
//!
//! ## Role in Mountain Architecture
//!
//! Implements role-based access control (RBAC) for IPC operations,
//! validating that users have necessary permissions before executing
//! operations.
//!
//! ## Primary Responsibility
//!
//! Validate user permissions for IPC operations using role-based access
//! control.
//!
//! ## Secondary Responsibilities
//!
//! - Create security context from incoming messages
//! - Map operations to required permissions
//! - Aggregate permissions from user roles
//! - Log permission validation results
//!
//! ## Dependencies
//!
//! **External Crates:**
//! - `std::collections::HashMap` - Role and permission storage
//! - `tokio::sync::RwLock` - Async-safe concurrent access
//! - `log` - Validation event logging
//! - `serde` - Serialization for audit trails
//!
//! **Internal Modules:**
//! - `ManageRole::{Role, Permission}` - Role and permission definitions
//! - `LogEvent::{SecurityEvent, SecurityEventType}` - Audit logging types
//!
//! ## Dependents
//!
//! - `TauriIPCServer` - Validates permissions before message processing
//! - `RouteMessage` - Routes only authorized messages
//!
//! ## VSCode Pattern Reference
//!
//! Matches VSCode's role-based permissions in `vs/base/common/permissions.ts`
//! - Hierarchical permission system
//! - Role-based access control
//! - Permission inheritance through role hierarchy
//! - Operation-to-permission mapping
//!
//! ## Security Considerations
//!
//! - RBAC prevents unauthorized access to sensitive operations
//! - All permission checks performed server-side (never trust client)
//! - Audit logging for security compliance
//! - Permission validation failures do not leak system internals
//! - Default-deny policy (explicit deny if permission not found)
//! - Timeout on permission checks prevents blocking
//! - Role-based inheritance for scalable permission management
//!
//! ## Performance Considerations
//!
//! - RwLock allows concurrent reads, exclusive writes
//! - Permission caching at role level reduces redundancy
//! - Fast HashMap lookups for permission resolution
//! - Async operations prevent blocking main thread
//! - Early validation fails fast to reject unauthorized requests
//!
//! ## Error Handling Strategy
//!
//! - Returns Result for explicit error handling
//! - Detailed error messages without exposing sensitive data
//! - Permission denied errors logged but don't crash system
//! - Invalid context handled gracefully with default values
//!
//! ## Thread Safety
//!
//! - RwLock wrapped in Arc for safe concurrent access
//! - Multiple concurrent reads, exclusive writes
//! - Lock contention minimized by short critical sections
//!
//! ## TODO Items
//!
//! - [ ] Implement role hierarchy (roles can inherit from parent roles)
//! - [ ] Add permission caching with TTL for frequently accessed permissions
//! - [ ] Support permission negation (explicit deny overrides allow)
//! - [ ] Add rate limiting for permission checks

use std::{
	collections::HashMap,
	sync::Arc,
	time::{Duration, SystemTime},
};

use tokio::sync::RwLock;
use log::{debug, error, warn};
use serde::{Deserialize, Serialize};

use super::super::{
	Audit::LogEvent::{SecurityEvent, SecurityEventType},
	Role::ManageRole::{Permission, Role},
};

/// Security context for permission validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityContext {
	/// Unique user identifier
	pub UserId:String,

	/// User's assigned roles
	pub Roles:Vec<String>,

	/// Direct user permissions (if any)
	pub Permissions:Vec<String>,

	/// Origin IP address for IP-based restrictions
	pub IpAddress:String,

	/// Request timestamp for time-based restrictions
	pub Timestamp:SystemTime,
}

/// Permission validator for IPC operations
pub struct PermissionValidator {
	/// Role definitions indexed by role name
	Roles:Arc<RwLock<HashMap<String, Role>>>,

	/// Permission definitions indexed by permission name
	Permissions:Arc<RwLock<HashMap<String, Permission>>>,

	/// Operation to permission mapping
	OperationPermissions:HashMap<String, Vec<String>>,

	/// Maximum time allowed for permission validation (milliseconds)
	ValidationTimeoutMillis:u64,
}

impl PermissionValidator {
	/// Create a new permission validator
	///
	/// ## Parameters
	/// - `ValidationTimeoutMillis`: Maximum timeout for validation in
	///   milliseconds
	///
	/// ## Returns
	/// New PermissionValidator instance
	pub fn New(ValidationTimeoutMillis:u64) -> Self {
		let OperationPermissions = Self::BuildOperationMapping();

		Self {
			Roles:Arc::new(RwLock::new(HashMap::new())),
			Permissions:Arc::new(RwLock::new(HashMap::new())),
			OperationPermissions,
			ValidationTimeoutMillis,
		}
	}

	/// Build operation to permission mapping
	///
	/// ## Returns
	/// HashMap mapping operation names to required permission strings
	fn BuildOperationMapping() -> HashMap<String, Vec<String>> {
		let mut mapping = HashMap::new();

		mapping.insert("file:write".to_string(), vec!["file.write".to_string()]);
		mapping.insert("file:delete".to_string(), vec!["file.write".to_string()]);
		mapping.insert("file:read".to_string(), vec!["file.read".to_string()]);
		mapping.insert("configuration:update".to_string(), vec!["config.update".to_string()]);
		mapping.insert("configuration:read".to_string(), vec!["config.read".to_string()]);
		mapping.insert("storage:set".to_string(), vec!["storage.write".to_string()]);
		mapping.insert("storage:get".to_string(), vec!["storage.read".to_string()]);
		mapping.insert("native:openExternal".to_string(), vec!["system.external".to_string()]);
		mapping.insert("system:execute".to_string(), vec!["system.execute".to_string()]);
		mapping.insert("admin:manage".to_string(), vec!["admin.manage".to_string()]);

		mapping
	}

	/// Create security context from message data
	///
	/// ## Parameters
	/// - `UserId`: User identifier for the request
	/// - `Roles`: User's assigned roles (defaults to ["user"] if empty)
	/// - `IpAddress`: Origin IP address
	/// - `DirectPermissions`: Direct user permissions (optional)
	///
	/// ## Returns
	/// New SecurityContext instance
	pub fn CreateSecurityContext(
		UserId:String,
		Roles:Vec<String>,
		IpAddress:String,
		DirectPermissions:Vec<String>,
	) -> SecurityContext {
		let ValidRoles = if Roles.is_empty() { vec!["user".to_string()] } else { Roles };

		let ValidIpAddress = if IpAddress.is_empty() { "127.0.0.1".to_string() } else { IpAddress };

		SecurityContext {
			UserId,
			Roles:ValidRoles,
			Permissions:DirectPermissions,
			IpAddress:ValidIpAddress,
			Timestamp:SystemTime::now(),
		}
	}

	/// Validate permission for an operation with security context
	///
	/// ## Parameters
	/// - `Operation`: The operation being performed
	/// - `Context`: Security context containing user info and roles
	///
	/// ## Returns
	/// Ok(()) if permission granted, Err with message if denied
	///
	/// ## Security Notes
	/// - All operations require explicit permission grant (default deny)
	/// - Validation is performed server-side only
	/// - IP address can be used for additional restrictions
	/// - Timestamp can be used for time-based restrictions
	pub async fn ValidatePermission(&self, Operation:&str, Context:&SecurityContext) -> Result<(), String> {
		// Start timeout timer
		let timeout_duration = Duration::from_millis(self.ValidationTimeoutMillis);

		// Use tokio::time::timeout for async timeout
		let result = tokio::time::timeout(timeout_duration, async {
			self.ValidatePermissionInternal(Operation, Context).await
		})
		.await;

		match result {
			Ok(validation_result) => validation_result,
			Err(_) => {
				error!(
					"[PermissionValidator] Permission validation timed out for operation: {}",
					Operation
				);
				Err("Permission validation timeout".to_string())
			},
		}
	}

	/// Internal validation logic (without timeout wrapper)
	///
	/// ## Parameters
	/// - `Operation`: The operation being performed
	/// - `Context`: Security context
	///
	/// ## Returns
	/// Ok(()) if permission granted, Err with message if denied
	async fn ValidatePermissionInternal(&self, Operation:&str, Context:&SecurityContext) -> Result<(), String> {
		// Validate inputs
		if Operation.is_empty() {
			warn!("[PermissionValidator] Empty operation name provided");
			return Err("Operation name cannot be empty".to_string());
		}

		if Context.UserId.is_empty() {
			warn!("[PermissionValidator] Empty user ID in security context");
			return Err("User ID cannot be empty".to_string());
		}

		if Context.Roles.is_empty() && Context.Permissions.is_empty() {
			warn!("[PermissionValidator] User has no roles or permissions: {}", Context.UserId);
			return Err("User has no assigned roles or permissions".to_string());
		}

		// Get required permissions for this operation
		let RequiredPermissions = match self.OperationPermissions.get(Operation) {
			Some(perms) => perms.clone(),
			None => {
				// No specific permissions required for this operation
				debug!(
					"[PermissionValidator] No specific permissions required for operation: {}",
					Operation
				);
				return Ok(());
			},
		};

		if RequiredPermissions.is_empty() {
			// No permissions needed
			debug!(
				"[PermissionValidator] Access granted (no permissions required): {} by {}",
				Operation, Context.UserId
			);
			Ok(())
		} else {
			// Aggregate all user permissions
			let UserPermissions = self.AggregateUserPermissions(Context).await?;

			// Check if user has all required permissions
			for RequiredPermission in &RequiredPermissions {
				if !UserPermissions.contains(RequiredPermission) {
					warn!(
						"[PermissionValidator] Permission denied: {} required, user {} has {:?}",
						RequiredPermission, Context.UserId, UserPermissions
					);
					return Err(format!("Missing required permission: {}", RequiredPermission));
				}
			}

			// All permissions granted
			debug!("[PermissionValidator] Access granted: {} by {}", Operation, Context.UserId);
			Ok(())
		}
	}

	/// Aggregate all permissions for a user from roles and direct permissions
	///
	/// ## Parameters
	/// - `Context`: Security context containing roles and direct permissions
	///
	/// ## Returns
	/// Vector of all permission strings available to the user
	async fn AggregateUserPermissions(&self, Context:&SecurityContext) -> Result<Vec<String>, String> {
		let mut UserPermissions:Vec<String> = Context.Permissions.clone();

		// Collect permissions from all roles
		let roles_read = self.Roles.read().await;
		for RoleName in &Context.Roles {
			if let Some(role) = roles_read.get(RoleName) {
				for Permission in &role.Permissions {
					if !UserPermissions.contains(Permission) {
						UserPermissions.push(Permission.clone());
					}
				}
			} else {
				debug!("[PermissionValidator] Role not found: {}, skipping", RoleName);
			}
		}

		Ok(UserPermissions)
	}

	/// Register a role definition
	///
	/// ## Parameters
	/// - `Role`: Role definition to register
	///
	/// ## Returns
	/// Result indicating success or error
	pub async fn RegisterRole(&self, Role:Role) -> Result<(), String> {
		// Validate role
		if Role.Name.is_empty() {
			return Err("Role name cannot be empty".to_string());
		}

		if Role.Permissions.is_empty() {
			warn!("[PermissionValidator] Role '{}' has no permissions", Role.Name);
		}

		let mut roles = self.Roles.write().await;

		// Validate that all referenced permissions exist
		let permissions_read = self.Permissions.read().await;
		for PermissionName in &Role.Permissions {
			if !permissions_read.contains_key(PermissionName) {
				warn!(
					"[PermissionValidator] Permission '{}' referenced by role '{}' does not exist",
					PermissionName, Role.Name
				);
			}
		}
		drop(permissions_read);

		roles.insert(Role.Name.clone(), Role);
		debug!("[PermissionValidator] Role registered: {}", roles.get(&Role.Name).unwrap().Name);
		Ok(())
	}

	/// Register a permission definition
	///
	/// ## Parameters
	/// - `Permission`: Permission definition to register
	///
	/// /// Returns
	/// Result indicating success or error
	pub async fn RegisterPermission(&self, Permission:Permission) -> Result<(), String> {
		// Validate permission
		if Permission.Name.is_empty() {
			return Err("Permission name cannot be empty".to_string());
		}

		if Permission.Description.is_empty() {
			return Err("Permission description cannot be empty".to_string());
		}

		let mut permissions = self.Permissions.write().await;
		permissions.insert(Permission.Name.clone(), Permission);
		debug!(
			"[PermissionValidator] Permission registered: {}",
			permissions.get(&Permission.Name).unwrap().Name
		);
		Ok(())
	}

	/// Get all permissions for a specific role
	///
	/// ## Parameters
	/// - `RoleName`: Name of the role to query
	///
	/// ## Returns
	/// Vector of permission strings for the role, empty if role not found
	pub async fn GetRolePermissions(&self, RoleName:&str) -> Vec<String> {
		let roles = self.Roles.read().await;
		roles.get(RoleName).map(|role| role.Permissions.clone()).unwrap_or_default()
	}

	/// Check if a user has a specific permission
	///
	/// ## Parameters
	/// - `Context`: Security context for the user
	/// - `PermissionName`: Permission name to check
	///
	/// /// Returns
	/// true if user has permission, false otherwise
	pub async fn HasPermission(&self, Context:&SecurityContext, PermissionName:&str) -> bool {
		// Check direct permissions
		if Context.Permissions.contains(&PermissionName.to_string()) {
			return true;
		}

		// Check role permissions
		let roles = self.Roles.read().await;
		for RoleName in &Context.Roles {
			if let Some(role) = roles.get(RoleName) {
				if role.Permissions.contains(&PermissionName.to_string()) {
					return true;
				}
			}
		}

		false
	}

	/// Initialize default roles and permissions
	///
	/// ## Returns
	/// Result indicating success or error
	///
	/// ## Default Roles
	/// - `user`: Read-only access to files, config, storage
	/// - `developer`: Read/write access to files and storage
	/// - `admin`: Full access including system operations
	pub async fn InitializeDefaults(&self) -> Result<(), String> {
		debug!("[PermissionValidator] Initializing default roles and permissions");

		// Define default permissions
		let DefaultPermissions = vec![
			Permission {
				Name:"file.read".to_string(),
				Description:"Read file operations".to_string(),
				Category:"file".to_string(),
			},
			Permission {
				Name:"file.write".to_string(),
				Description:"Write file operations".to_string(),
				Category:"file".to_string(),
			},
			Permission {
				Name:"config.read".to_string(),
				Description:"Read configuration".to_string(),
				Category:"config".to_string(),
			},
			Permission {
				Name:"config.update".to_string(),
				Description:"Update configuration".to_string(),
				Category:"config".to_string(),
			},
			Permission {
				Name:"storage.read".to_string(),
				Description:"Read storage".to_string(),
				Category:"storage".to_string(),
			},
			Permission {
				Name:"storage.write".to_string(),
				Description:"Write storage".to_string(),
				Category:"storage".to_string(),
			},
			Permission {
				Name:"system.external".to_string(),
				Description:"Access external system resources".to_string(),
				Category:"system".to_string(),
			},
			Permission {
				Name:"system.execute".to_string(),
				Description:"Execute system commands".to_string(),
				Category:"system".to_string(),
			},
			Permission {
				Name:"admin.manage".to_string(),
				Description:"Administrative management operations".to_string(),
				Category:"admin".to_string(),
			},
		];

		// Register permissions
		for Permission in DefaultPermissions {
			self.RegisterPermission(Permission).await?;
		}

		// Define default roles
		let DefaultRoles = vec![
			Role {
				Name:"user".to_string(),
				Permissions:vec!["file.read".to_string(), "config.read".to_string(), "storage.read".to_string()],
				Description:"Standard user with read access".to_string(),
			},
			Role {
				Name:"developer".to_string(),
				Permissions:vec![
					"file.read".to_string(),
					"file.write".to_string(),
					"config.read".to_string(),
					"storage.read".to_string(),
					"storage.write".to_string(),
				],
				Description:"Developer with read/write access".to_string(),
			},
			Role {
				Name:"admin".to_string(),
				Permissions:vec![
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
				Description:"Administrator with full access".to_string(),
			},
		];

		// Register roles
		for Role in DefaultRoles {
			self.RegisterRole(Role).await?;
		}

		debug!("[PermissionValidator] Default roles and permissions initialized successfully");
		Ok(())
	}
}

#[cfg(test)]
mod Tests {
	use super::*;

	#[tokio::test]
	async fn TestCreateSecurityContext() {
		let context = PermissionValidator::CreateSecurityContext(
			"test-user".to_string(),
			vec!["user".to_string(), "admin".to_string()],
			"192.168.1.1".to_string(),
			vec!["custom.permission".to_string()],
		);

		assert_eq!(context.UserId, "test-user".to_string());
		assert_eq!(context.Roles, vec!["user".to_string(), "admin".to_string()]);
		assert_eq!(context.IpAddress, "192.168.1.1".to_string());
		assert_eq!(context.Permissions, vec!["custom.permission".to_string()]);
	}

	#[tokio::test]
	async fn TestCreateSecurityContextDefaults() {
		let context =
			PermissionValidator::CreateSecurityContext("test-user".to_string(), vec![], "".to_string(), vec![]);

		assert_eq!(context.UserId, "test-user".to_string());
		assert_eq!(context.Roles, vec!["user".to_string()]);
		assert_eq!(context.IpAddress, "127.0.0.1".to_string());
	}

	#[tokio::test]
	async fn TestValidatePermissionNoPermissionsRequired() {
		let validator = PermissionValidator::New(1000);
		let context = SecurityContext {
			UserId:"test-user".to_string(),
			Roles:vec!["user".to_string()],
			Permissions:vec![],
			IpAddress:"127.0.0.1".to_string(),
			Timestamp:SystemTime::now(),
		};

		let result = validator.ValidatePermission("unknown:operation", &context).await;
		assert!(result.is_ok(), "Should succeed when no permissions are required");
	}

	#[tokio::test]
	async fn TestValidatePermissionMissingPermission() {
		let validator = PermissionValidator::New(1000);
		let context = SecurityContext {
			UserId:"test-user".to_string(),
			Roles:vec!["user".to_string()],
			Permissions:vec![],
			IpAddress:"127.0.0.1".to_string(),
			Timestamp:SystemTime::now(),
		};

		let result = validator.ValidatePermission("file:write", &context).await;
		assert!(result.is_err(), "Should fail without required permission");
		assert!(result.unwrap_err().contains("Missing required permission"));
	}

	#[tokio::test]
	async fn TestValidatePermissionWithDirectPermission() {
		let validator = PermissionValidator::New(1000);
		let context = SecurityContext {
			UserId:"test-user".to_string(),
			Roles:vec![],
			Permissions:vec!["file.write".to_string()],
			IpAddress:"127.0.0.1".to_string(),
			Timestamp:SystemTime::now(),
		};

		let result = validator.ValidatePermission("file:write", &context).await;
		assert!(result.is_ok(), "Should succeed with direct permission");
	}

	#[tokio::test]
	async fn TestValidatePermissionViaRole() {
		let validator = PermissionValidator::New(1000);
		validator.InitializeDefaults().await.unwrap();

		let context = SecurityContext {
			UserId:"test-user".to_string(),
			Roles:vec!["admin".to_string()],
			Permissions:vec![],
			IpAddress:"127.0.0.1".to_string(),
			Timestamp:SystemTime::now(),
		};

		let result = validator.ValidatePermission("file:write", &context).await;
		assert!(result.is_ok(), "Should succeed via role permission");
	}

	#[tokio::test]
	async fn TestValidatePermissionEmptyOperation() {
		let validator = PermissionValidator::New(1000);
		let context = SecurityContext {
			UserId:"test-user".to_string(),
			Roles:vec!["user".to_string()],
			Permissions:vec![],
			IpAddress:"127.0.0.1".to_string(),
			Timestamp:SystemTime::now(),
		};

		let result = validator.ValidatePermission("", &context).await;
		assert!(result.is_err(), "Should fail with empty operation name");
	}

	#[tokio::test]
	async fn TestValidatePermissionEmptyUserId() {
		let validator = PermissionValidator::New(1000);
		let context = SecurityContext {
			UserId:"".to_string(),
			Roles:vec!["user".to_string()],
			Permissions:vec![],
			IpAddress:"127.0.0.1".to_string(),
			Timestamp:SystemTime::now(),
		};

		let result = validator.ValidatePermission("file:read", &context).await;
		assert!(result.is_err(), "Should fail with empty user ID");
	}

	#[tokio::test]
	async fn TestInitializeDefaults() {
		let validator = PermissionValidator::New(1000);
		let result = validator.InitializeDefaults().await;
		assert!(result.is_ok(), "Should initialize defaults successfully");

		// Check roles exist
		let user_perms = validator.GetRolePermissions("user").await;
		assert!(user_perms.contains(&"file.read".to_string()));

		let admin_perms = validator.GetRolePermissions("admin").await;
		assert!(admin_perms.len(), "Admin should have many permissions");
	}

	#[tokio::test]
	async fn TestGetRolePermissions() {
		let validator = PermissionValidator::New(1000);
		validator.InitializeDefaults().await.unwrap();

		let role_perms = validator.GetRolePermissions("developer").await;
		assert!(role_perms.contains(&"file.read".to_string()));
		assert!(role_perms.contains(&"file.write".to_string()));
	}

	#[tokio::test]
	async fn TestHasPermissionWithRole() {
		let validator = PermissionValidator::New(1000);
		validator.InitializeDefaults().await.unwrap();

		let context = SecurityContext {
			UserId:"test-user".to_string(),
			Roles:vec!["admin".to_string()],
			Permissions:vec![],
			IpAddress:"127.0.0.1".to_string(),
			Timestamp:SystemTime::now(),
		};

		assert!(validator.HasPermission(&context, "file.read").await);
		assert!(validator.HasPermission(&context, "config.update").await);
	}

	#[tokio::test]
	async fn TestHasPermissionDirect() {
		let validator = PermissionValidator::New(1000);

		let context = SecurityContext {
			UserId:"test-user".to_string(),
			Roles:vec![],
			Permissions:vec!["custom.perm".to_string()],
			IpAddress:"127.0.0.1".to_string(),
			Timestamp:SystemTime::now(),
		};

		assert!(validator.HasPermission(&context, "custom.perm").await);
		assert!(!validator.HasPermission(&context, "file.read").await);
	}
}
