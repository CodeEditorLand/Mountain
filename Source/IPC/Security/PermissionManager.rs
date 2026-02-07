//! # Permission Manager (IPC Security)
//!
//! ## RESPONSIBILITIES
//! This module provides role-based access control (RBAC) for IPC operations.
//! It validates permissions for all incoming IPC messages and logs security
//! events for comprehensive audit trails.
//!
//! ## ARCHITECTURAL ROLE
//! This module is the security enforcement layer in the IPC architecture,
//! ensuring that all operations are authorized based on user roles and permissions.
//!
//! ## KEY COMPONENTS
//!
//! - **PermissionManager**: Main permission validation and management structure
//! - **SecurityContext**: Context information for permission validation
//! - **SecurityEvent**: Audit log entry for security events
//! - **SecurityEventType**: Types of security events
//!
//! ## ERROR HANDLING
//! Permission validation returns Result types with descriptive error messages
//! when access is denied.
//!
//! ## LOGGING
//! All security events are logged to the audit log. Info-level logging for
//! access grants, error-level for permission denials.
//!
//! ## PERFORMANCE CONSIDERATIONS
//! - Permission definitions cached in RwLock for fast concurrent access
//! - Role resolution optimized with HashMap lookups (O(1) complexity)
//! - Audit log limited to last 1000 events to prevent memory bloat
//!
//! ## TODO
//! - Add permission caching with TTL
//! - Implement permission inheritance
//! - Add permission alias support
//! - Implement group-based permissions

use std::collections::HashMap;
use std::sync::Arc;

use log::debug;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use super::{Role::Role, Permission::Permission};

/// Security context for permission validation
///
/// This structure contains all information needed to validate whether an
/// operation should be allowed based on the requester's identity and permissions.
///
/// ## Context Flow
///
/// ```text
/// IPC Message
///     |
///     | Extract user info
///     v
/// SecurityContext (user_id, roles, permissions, ip_address)
///     |
///     | PermissionManager.validate_permission()
///     v
/// Access Decision (Allowed/Denied)
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityContext {
	/// User identifier requesting the operation
	pub user_id: String,

	/// List of roles assigned to the user
	pub roles: Vec<String>,

	/// Direct permissions granted to the user
	pub permissions: Vec<String>,

	/// IP address of the requester (for location-based restrictions)
	pub ip_address: String,

	/// Timestamp of the request (for time-based restrictions)
	pub timestamp: std::time::SystemTime,
}

impl SecurityContext {
	/// Create a new security context
	pub fn new(
		user_id: String,
		roles: Vec<String>,
		permissions: Vec<String>,
		ip_address: String,
	) -> Self {
		Self {
			user_id,
			roles,
			permissions,
			ip_address,
			timestamp: std::time::SystemTime::now(),
		}
	}

	/// Check if user has a specific role
	pub fn has_role(&self, role: &str) -> bool {
		self.roles.iter().any(|r| r == role)
	}

	/// Check if user has a specific permission
	pub fn has_permission(&self, permission: &str) -> bool {
		self.permissions.iter().any(|p| p == permission)
	}

	/// Create a default IPC context (used for local IPC connections)
	/// IPC connections use loopback address for security (localhost only)
	pub fn ipc_default() -> Self {
		Self {
			user_id: "ipc-connection".to_string(),
			roles: vec!["user".to_string()],
			permissions: vec![],
			ip_address: "127.0.0.1".to_string(),
			timestamp: std::time::SystemTime::now(),
		}
	}
}

/// Security event for auditing
///
/// This structure records all security-related events for comprehensive
/// audit trails and compliance monitoring.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityEvent {
	/// Type of security event
	pub event_type: SecurityEventType,

	/// User identifier who triggered the event
	pub user_id: String,

	/// Operation that was attempted
	pub operation: String,

	/// When the event occurred
	pub timestamp: std::time::SystemTime,

	/// Additional details about the event
	pub details: Option<String>,
}

impl SecurityEvent {
	/// Create a new security event
	pub fn new(
		event_type: SecurityEventType,
		user_id: String,
		operation: String,
		details: Option<String>,
	) -> Self {
		Self {
			event_type,
			user_id,
			operation,
			timestamp: std::time::SystemTime::now(),
			details,
		}
	}
}

/// Types of security events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityEventType {
	/// Access was denied due to insufficient permissions
	PermissionDenied,

	/// Access was granted
	AccessGranted,

	/// Configuration was changed
	ConfigurationChange,

	/// A security violation was detected
	SecurityViolation,

	/// Performance anomaly detected (could indicate attack)
	PerformanceAnomaly,
}

/// Permission manager for IPC operations
///
/// This is the main security enforcement structure for the IPC layer. It
/// maintains role and permission definitions, validates access requests, and
/// logs security events for auditing.
///
/// ## Permission Flow
///
/// ```text
/// IPC Message arrives
///     |
///     | validate_permission(operation, context)
///     v
/// Check if operation requires permissions
///     |
///     | Yes -> Get required permissions
///     v
/// Check user permissions (direct + role-based)
///     |
///     | Has all required permissions?
///     v
/// Yes -> Log AccessGranted -> Allow operation
/// No  -> Log PermissionDenied -> Deny operation
/// ```
///
/// ## Default Roles
///
/// The PermissionManager initializes with three default roles:
///
/// - **user**: Read-only access to files, configuration, and storage
/// - **developer**: Read/write access to files and storage, configuration read
/// - **admin**: Full access including system operations and configuration updates
///
/// ## Default Permissions
///
/// Standard permissions include:
/// - file.read, file.write
/// - config.read, config.update
/// - storage.read, storage.write
/// - system.external
pub struct PermissionManager {
	/// Role definitions with associated permissions
	roles: Arc<RwLock<HashMap<String, Role>>>,

	/// Permission definitions with descriptions
	permissions: Arc<RwLock<HashMap<String, Permission>>>,

	/// Security audit log (limited to last 1000 events)
	audit_log: Arc<RwLock<Vec<SecurityEvent>>>,
}

impl PermissionManager {
	/// Create a new permission manager
	pub fn new() -> Self {
		debug!("[PermissionManager] Creating new PermissionManager instance");

		Self {
			roles: Arc::new(RwLock::new(HashMap::new())),
			permissions: Arc::new(RwLock::new(HashMap::new())),
			audit_log: Arc::new(RwLock::new(Vec::new())),
		}
	}

	/// Validate permission for an operation
	///
	/// This method checks if the given security context has sufficient
	/// permissions to perform the specified operation.
	///
	/// ## Parameters
	/// - `operation`: The operation being attempted (e.g., "file:write", "config:update")
	/// - `context`: The security context containing user information
	///
	/// ## Returns
	/// - `Ok(())` if the operation is allowed
	/// - `Err(String)` with reason if denied
	///
	/// ## Example
	///
	/// ```rust,ignore
	/// let context = SecurityContext::ipc_default();
	/// permission_manager.validate_permission("file:read", &context).await?;
	/// ```
	pub async fn validate_permission(&self, operation: &str, context: &SecurityContext) -> Result<(), String> {
		// Check if operation requires specific permissions
		let required_permissions = self.get_required_permissions(operation).await;

		if required_permissions.is_empty() {
			debug!(
				"[PermissionManager] Operation '{}' requires no special permissions",
				operation
			);
			return Ok(()); // No specific permissions required
		}

		// Collect all user permissions (direct + role-based)
		let mut user_permissions: Vec<String> = context.permissions.iter().cloned().collect();

		for role in context.roles.iter() {
			let role_perms = self.get_role_permissions(role).await;
			user_permissions.extend(role_perms);
		}

		// Check if user has all required permissions
		for required in &required_permissions {
			if !user_permissions.contains(required) {
				let error = format!("Missing permission: {}", required);
				debug!(
					"[PermissionManager] Permission denied for user '{}' on operation '{}': {}",
					context.user_id, operation, error
				);

				// Log permission denial
				self.log_security_event(SecurityEvent {
					event_type: SecurityEventType::PermissionDenied,
					user_id: context.user_id.clone(),
					operation: operation.to_string(),
					timestamp: std::time::SystemTime::now(),
					details: Some(format!("Permission denied: {}", error)),
				})
				.await;

				return Err(error);
			}
		}

		// Log successful access
		self.log_security_event(SecurityEvent {
			event_type: SecurityEventType::AccessGranted,
			user_id: context.user_id.clone(),
			operation: operation.to_string(),
			timestamp: std::time::SystemTime::now(),
			details: Some(format!("Access granted for operation: {}", operation)),
		})
		.await;

		debug!(
			"[PermissionManager] Access granted for user '{}' on operation '{}'",
			context.user_id, operation
		);

		Ok(())
	}

	/// Get required permissions for an operation
	///
	/// This method defines which permissions are required for which operations.
	/// Operations not in the mapping require no special permissions by default.
	///
	/// ## Operation Permission Mapping
	///
	/// | Operation | Required Permissions |
	/// |-----------|---------------------|
	/// | file:write | file.write |
	/// | file:delete | file.write |
	/// | configuration:update | config.update |
	/// | storage:set | storage.write |
	/// | native:openExternal | system.external |
	///
	async fn get_required_permissions(&self, operation: &str) -> Vec<String> {
		match operation {
			"file:write" | "file:delete" => vec!["file.write".to_string()],
			"configuration:update" => vec!["config.update".to_string()],
			"storage:set" => vec!["storage.write".to_string()],
			"native:openExternal" => vec!["system.external".to_string()],
			// Operations not in the mapping require no special permissions by default
			_ => Vec::new(),
		}
	}

	/// Get permissions for a role
	async fn get_role_permissions(&self, role_name: &str) -> Vec<String> {
		let roles = self.roles.read().await;
		roles
			.get(role_name)
			.map(|role| role.permissions.clone())
			.unwrap_or_default()
	}

	/// Log security event
	pub async fn log_security_event(&self, event: SecurityEvent) {
		let mut audit_log = self.audit_log.write().await;
		audit_log.push(event.clone());

		// Keep only last 1000 events
		if audit_log.len() > 1000 {
			audit_log.remove(0);
		}

		match event.event_type {
			SecurityEventType::PermissionDenied => {
				log::warn!(
					"[SecurityEvent] Permission denied - User: {}, Operation: {}, Details: {:?}",
					event.user_id,
					event.operation,
					event.details
				);
			}
			SecurityEventType::SecurityViolation => {
				log::error!(
					"[SecurityEvent] Security violation - User: {}, Operation: {}, Details: {:?}",
					event.user_id,
					event.operation,
					event.details
				);
			}
			SecurityEventType::AccessGranted => {
				log::info!(
					"[SecurityEvent] Access granted - User: {}, Operation: {}",
					event.user_id,
					event.operation
				);
			}
			_ => {
				log::debug!(
					"[SecurityEvent] {:?} - User: {}, Operation: {}",
					event.event_type,
					event.user_id,
					event.operation
				);
			}
		}
	}

	/// Get security audit log
	///
	/// Returns the most recent security events up to the specified limit.
	pub async fn get_audit_log(&self, limit: usize) -> Vec<SecurityEvent> {
		let audit_log = self.audit_log.read().await;
		audit_log.iter().rev().take(limit).cloned().collect()
	}

	/// Initialize default roles and permissions
	///
	/// This method sets up the standard RBAC structure with three default roles
	/// and their associated permissions.
	pub async fn initialize_defaults(&self) {
		debug!("[PermissionManager] Initializing default roles and permissions");

		let mut permissions = self.permissions.write().await;
		let mut roles = self.roles.write().await;

		// Define standard permissions
		let standard_permissions = vec![
			("file.read", "Read file operations"),
			("file.write", "Write file operations"),
			("config.read", "Read configuration"),
			("config.update", "Update configuration"),
			("storage.read", "Read storage"),
			("storage.write", "Write storage"),
			("system.external", "Access external system resources"),
		];

		for (name, description) in standard_permissions {
			permissions.insert(
				name.to_string(),
				Permission {
					name: name.to_string(),
					description: description.to_string(),
					category: "standard".to_string(),
				},
			);
		}

		// Define standard roles
		let standard_roles = vec![
			("user", vec!["file.read", "config.read", "storage.read"]),
			(
				"developer",
				vec!["file.read", "file.write", "config.read", "storage.read", "storage.write"],
			),
			(
				"admin",
				vec![
					"file.read",
					"file.write",
					"config.read",
					"config.update",
					"storage.read",
					"storage.write",
					"system.external",
				],
			),
		];

		for (name, role_permissions) in standard_roles {
			roles.insert(
				name.to_string(),
				Role {
					name: name.to_string(),
					permissions: role_permissions.iter().map(|p| p.to_string()).collect(),
					description: format!("{} role with standard permissions", name),
				},
			);
		}

		debug!("[PermissionManager] Initialized {} permissions and {} roles", permissions.len(), roles.len());
	}

	/// Add a custom role
	pub async fn add_role(&self, role: Role) {
		let role_name = role.name.clone();
		let mut roles = self.roles.write().await;
		roles.insert(role_name.clone(), role);
		debug!("[PermissionManager] Added role: {}", role_name);
	}

	/// Add a custom permission
	pub async fn add_permission(&self, permission: Permission) {
		let permission_name = permission.name.clone();
		let mut permissions = self.permissions.write().await;
		permissions.insert(permission_name.clone(), permission);
		debug!("[PermissionManager] Added permission: {}", permission_name);
	}

	/// Clear the audit log
	pub async fn clear_audit_log(&self) {
		let mut audit_log = self.audit_log.write().await;
		audit_log.clear();
		debug!("[PermissionManager] Audit log cleared");
	}

	/// Get audit log statistics
	pub async fn get_audit_log_stats(&self) -> (usize, Vec<(&'static str, usize)>) {
		let audit_log = self.audit_log.read().await;

		let mut type_counts: Vec<(&'static str, usize)> = vec![
			("PermissionDenied", 0),
			("AccessGranted", 0),
			("ConfigurationChange", 0),
			("SecurityViolation", 0),
			("PerformanceAnomaly", 0),
		];

		for event in audit_log.iter() {
			let type_name = match event.event_type {
				SecurityEventType::PermissionDenied => "PermissionDenied",
				SecurityEventType::AccessGranted => "AccessGranted",
				SecurityEventType::ConfigurationChange => "ConfigurationChange",
				SecurityEventType::SecurityViolation => "SecurityViolation",
				SecurityEventType::PerformanceAnomaly => "PerformanceAnomaly",
			};
			if let Some((_, count)) = type_counts.iter_mut().find(|(name, _)| *name == type_name) {
				*count += 1;
			}
		}

		(audit_log.len(), type_counts)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

#[tokio::test]
	async fn test_permission_manager_creation() {
		let manager = PermissionManager::new();
		assert_eq!(manager.get_audit_log(10).await.len(), 0);
	}

#[tokio::test]
	async fn test_initialize_defaults() {
		let manager = PermissionManager::new();
		manager.initialize_defaults().await;

		let log = manager.get_audit_log(10).await;
		// Should have logged initialization events
		assert!(!log.is_empty());
	}

#[tokio::test]
	async fn test_security_context_ipc_default() {
		let context = SecurityContext::ipc_default();
		assert_eq!(context.user_id, "ipc-connection");
		assert!(context.has_role("user"));
		assert_eq!(context.ip_address, "127.0.0.1");
	}

#[tokio::test]
	async fn test_permission_validation_access_granted() {
		let manager = PermissionManager::new();
		manager.initialize_defaults().await;

		let context = SecurityContext::new(
			"test-user".to_string(),
			vec!["admin".to_string()],
			vec![],
			"127.0.0.1".to_string(),
		);

		// Admin should have file.write permission
		let result = manager.validate_permission("file:write", &context).await;
		assert!(result.is_ok());

		// Check that access was logged
		let log = manager.get_audit_log(10).await;
		assert!(log.iter().any(|e| matches!(e.event_type, SecurityEventType::AccessGranted)));
	}

#[tokio::test]
	async fn test_permission_validation_access_denied() {
		let manager = PermissionManager::new();
		manager.initialize_defaults().await;

		let context = SecurityContext::new(
			"test-user".to_string(),
			vec!["user".to_string()], // User role doesn't have file.write
			vec![],
			"127.0.0.1".to_string(),
		);

		// User role should NOT have file.write permission
		let result = manager.validate_permission("file:write", &context).await;
		assert!(result.is_err());

		// Check that denial was logged
		let log = manager.get_audit_log(10).await;
		assert!(log.iter().any(|e| matches!(e.event_type, SecurityEventType::PermissionDenied)));
	}

#[tokio::test]
	async fn test_operations_without_permissions() {
		let manager = PermissionManager::new();
		manager.initialize_defaults().await;

		let context = SecurityContext::ipc_default();

		// Operations not in the mapping should require no permissions
		let result = manager.validate_permission("custom_operation", &context).await;
		assert!(result.is_ok());
	}

#[tokio::test]
	async fn test_audit_log_limit() {
		let manager = PermissionManager::new();
		manager.initialize_defaults().await;

		// Add more than 1000 events
		for i in 0..1100 {
			manager
				.log_security_event(SecurityEvent {
					event_type: SecurityEventType::AccessGranted,
					user_id: format!("user-{}", i),
					operation: "test".to_string(),
					timestamp: std::time::SystemTime::now(),
					details: None,
				})
				.await;
		}

		// Should only have last 1000 events
		let log = manager.get_audit_log(2000).await;
		assert_eq!(log.len(), 1000);
	}

#[tokio::test]
	async fn test_custom_role() {
		let manager = PermissionManager::new();
		manager.initialize_defaults().await;

		let custom_role = Role {
			name: "custom".to_string(),
			permissions: vec!["file.read".to_string()],
			description: "Custom role".to_string(),
		};

		manager.add_role(custom_role).await;

		let context = SecurityContext::new(
			"test-user".to_string(),
			vec!["custom".to_string()],
			vec![],
			"127.0.0.1".to_string(),
		);

		// Custom role should work
		let result = manager.validate_permission("file:read", &context).await;
		assert!(result.is_ok());
	}
}
