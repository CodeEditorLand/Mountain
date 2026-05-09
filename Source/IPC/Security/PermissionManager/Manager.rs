#![allow(non_snake_case)]

//! `Manager::Struct` - the IPC RBAC enforcement core. Holds
//! the role / permission tables and the rolling 1k audit log;
//! `validate_permission` is the gate every IPC operation
//! passes through before dispatch. The struct + impl + tests
//! stay in one file - tightly coupled cluster.

use std::{collections::HashMap, sync::Arc};

use tokio::sync::RwLock;

use crate::{
	IPC::Security::{
		Permission::Permission,
		PermissionManager::{
			SecurityContext::Struct as SecurityContext,
			SecurityEvent::Struct as SecurityEvent,
			SecurityEventType::Enum as SecurityEventType,
		},
		Role::Role,
	},
	dev_log,
};

pub struct Struct {

	pub(super) roles:Arc<RwLock<HashMap<String, Role>>>,

	pub(super) permissions:Arc<RwLock<HashMap<String, Permission>>>,

	pub(super) audit_log:Arc<RwLock<Vec<SecurityEvent>>>,
}

impl Struct {

	pub fn new() -> Self {

		dev_log!("ipc", "[PermissionManager] Creating new PermissionManager instance");

		Self {

			roles:Arc::new(RwLock::new(HashMap::new())),

			permissions:Arc::new(RwLock::new(HashMap::new())),

			audit_log:Arc::new(RwLock::new(Vec::new())),
		}
	}

	pub async fn validate_permission(&self, operation:&str, context:&SecurityContext) -> Result<(), String> {

		let required_permissions = self.get_required_permissions(operation).await;

		if required_permissions.is_empty() {

			dev_log!(
				"ipc",

				"[PermissionManager] Operation '{}' requires no special permissions",

				operation
			);

			return Ok(());
		}

		let mut user_permissions:Vec<String> = context.permissions.iter().cloned().collect();

		for role in context.roles.iter() {

			let role_perms = self.get_role_permissions(role).await;

			user_permissions.extend(role_perms);
		}

		for required in &required_permissions {

			if !user_permissions.contains(required) {

				let error = format!("Missing permission: {}", required);

				dev_log!(
					"ipc",

					"[PermissionManager] Permission denied for user '{}' on operation '{}': {}",

					context.user_id,

					operation,

					error
				);

				self.log_security_event(SecurityEvent {
					event_type:SecurityEventType::PermissionDenied,
					user_id:context.user_id.clone(),
					operation:operation.to_string(),
					timestamp:std::time::SystemTime::now(),
					details:Some(format!("Permission denied: {}", error)),
				})
				.await;

				return Err(error);
			}
		}

		self.log_security_event(SecurityEvent {
			event_type:SecurityEventType::AccessGranted,
			user_id:context.user_id.clone(),
			operation:operation.to_string(),
			timestamp:std::time::SystemTime::now(),
			details:Some(format!("Access granted for operation: {}", operation)),
		})
		.await;

		dev_log!(
			"ipc",

			"[PermissionManager] Access granted for user '{}' on operation '{}'",

			context.user_id,

			operation
		);

		Ok(())
	}

	async fn get_required_permissions(&self, operation:&str) -> Vec<String> {

		match operation {

			"file:write" | "file:delete" => vec!["file.write".to_string()],

			"configuration:update" => vec!["config.update".to_string()],

			"storage:set" => vec!["storage.write".to_string()],

			"native:openExternal" => vec!["system.external".to_string()],

			_ => Vec::new(),
		}
	}

	async fn get_role_permissions(&self, role_name:&str) -> Vec<String> {

		let roles = self.roles.read().await;

		roles.get(role_name).map(|role| role.permissions.clone()).unwrap_or_default()
	}

	pub async fn log_security_event(&self, event:SecurityEvent) {

		let mut audit_log = self.audit_log.write().await;

		audit_log.push(event.clone());

		if audit_log.len() > 1000 {

			audit_log.remove(0);
		}

		match event.event_type {

			SecurityEventType::PermissionDenied => {

				dev_log!(
					"ipc",

					"warn: [SecurityEvent] Permission denied - User: {}, Operation: {}, Details: {:?}",

					event.user_id,

					event.operation,

					event.details
				);
			},

			SecurityEventType::SecurityViolation => {

				dev_log!(
					"ipc",

					"error: [SecurityEvent] Security violation - User: {}, Operation: {}, Details: {:?}",

					event.user_id,

					event.operation,

					event.details
				);
			},

			SecurityEventType::AccessGranted => {

				dev_log!(
					"ipc",

					"[SecurityEvent] Access granted - User: {}, Operation: {}",

					event.user_id,

					event.operation
				);
			},

			_ => {

				dev_log!(
					"ipc",

					"[SecurityEvent] {:?} - User: {}, Operation: {}",

					event.event_type,

					event.user_id,

					event.operation
				);
			},
		}
	}

	pub async fn get_audit_log(&self, limit:usize) -> Vec<SecurityEvent> {

		let audit_log = self.audit_log.read().await;

		audit_log.iter().rev().take(limit).cloned().collect()
	}

	pub async fn initialize_defaults(&self) {

		dev_log!("ipc", "[PermissionManager] Initializing default roles and permissions");

		let mut permissions = self.permissions.write().await;

		let mut roles = self.roles.write().await;

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
					name:name.to_string(),
					description:description.to_string(),
					category:"standard".to_string(),
				},
			);
		}

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
					name:name.to_string(),
					permissions:role_permissions.iter().map(|p| p.to_string()).collect(),
					description:format!("{} role with standard permissions", name),
				},
			);
		}

		dev_log!(
			"ipc",

			"[PermissionManager] Initialized {} permissions and {} roles",

			permissions.len(),

			roles.len()
		);
	}

	pub async fn add_role(&self, role:Role) {

		let role_name = role.name.clone();

		let mut roles = self.roles.write().await;

		roles.insert(role_name.clone(), role);

		dev_log!("ipc", "[PermissionManager] Added role: {}", role_name);
	}

	pub async fn add_permission(&self, permission:Permission) {

		let permission_name = permission.name.clone();

		let mut permissions = self.permissions.write().await;

		permissions.insert(permission_name.clone(), permission);

		dev_log!("ipc", "[PermissionManager] Added permission: {}", permission_name);
	}

	pub async fn clear_audit_log(&self) {

		let mut audit_log = self.audit_log.write().await;

		audit_log.clear();

		dev_log!("ipc", "[PermissionManager] Audit log cleared");
	}

	pub async fn get_audit_log_stats(&self) -> (usize, Vec<(&'static str, usize)>) {

		let audit_log = self.audit_log.read().await;

		let mut type_counts:Vec<(&'static str, usize)> = vec![
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
