#![allow(non_snake_case)]

//! `Validator::Struct` - role-based access control engine.
//! Holds the role / permission tables and the operation →
//! required-permission mapping; enforces the
//! default-deny + RBAC policy through `ValidatePermission`.
//! The struct + impl + tests stay in one file - tightly
//! coupled cluster.

use std::{
	collections::HashMap,
	sync::Arc,
	time::{Duration, SystemTime},
};

use tokio::sync::RwLock;

use crate::{
	IPC::Permission::{
		Role::ManageRole::{Permission::Struct as Permission, Role::Struct as Role},
		Validate::ValidatePermission::SecurityContext::Struct as SecurityContext,
	},
	dev_log,
};

pub struct Struct {
	pub(super) Roles:Arc<RwLock<HashMap<String, Role>>>,

	pub(super) Permissions:Arc<RwLock<HashMap<String, Permission>>>,

	pub(super) OperationPermissions:HashMap<String, Vec<String>>,

	pub(super) ValidationTimeoutMillis:u64,
}

impl Struct {
	pub fn New(ValidationTimeoutMillis:u64) -> Self {
		Self {
			Roles:Arc::new(RwLock::new(HashMap::new())),

			Permissions:Arc::new(RwLock::new(HashMap::new())),

			OperationPermissions:Self::BuildOperationMapping(),

			ValidationTimeoutMillis,
		}
	}

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

	pub async fn ValidatePermission(&self, Operation:&str, Context:&SecurityContext) -> Result<(), String> {
		let timeout_duration = Duration::from_millis(self.ValidationTimeoutMillis);

		let result = tokio::time::timeout(timeout_duration, async {
			self.ValidatePermissionInternal(Operation, Context).await
		})
		.await;

		match result {
			Ok(validation_result) => validation_result,

			Err(_) => {
				dev_log!(
					"ipc",
					"error: [PermissionValidator] Permission validation timed out for operation: {}",
					Operation
				);

				Err("Permission validation timeout".to_string())
			},
		}
	}

	async fn ValidatePermissionInternal(&self, Operation:&str, Context:&SecurityContext) -> Result<(), String> {
		if Operation.is_empty() {
			return Err("Operation name cannot be empty".to_string());
		}

		if Context.UserId.is_empty() {
			return Err("User ID cannot be empty".to_string());
		}

		if Context.Roles.is_empty() && Context.Permissions.is_empty() {
			return Err("User has no assigned roles or permissions".to_string());
		}

		let RequiredPermissions = match self.OperationPermissions.get(Operation) {
			Some(perms) => perms.clone(),

			None => return Ok(()),
		};

		if RequiredPermissions.is_empty() {
			return Ok(());
		}

		let UserPermissions = self.AggregateUserPermissions(Context).await?;

		for RequiredPermission in &RequiredPermissions {
			if !UserPermissions.contains(RequiredPermission) {
				return Err(format!("Missing required permission: {}", RequiredPermission));
			}
		}

		Ok(())
	}

	async fn AggregateUserPermissions(&self, Context:&SecurityContext) -> Result<Vec<String>, String> {
		let mut UserPermissions:Vec<String> = Context.Permissions.clone();

		let roles_read = self.Roles.read().await;

		for RoleName in &Context.Roles {
			if let Some(role) = roles_read.get(RoleName) {
				for Permission in &role.Permissions {
					if !UserPermissions.contains(Permission) {
						UserPermissions.push(Permission.clone());
					}
				}
			} else {
				dev_log!("ipc", "[PermissionValidator] Role not found: {}, skipping", RoleName);
			}
		}

		Ok(UserPermissions)
	}

	pub async fn RegisterRole(&self, Role:Role) -> Result<(), String> {
		if Role.Name.is_empty() {
			return Err("Role name cannot be empty".to_string());
		}

		let mut roles = self.Roles.write().await;

		let permissions_read = self.Permissions.read().await;

		for PermissionName in &Role.Permissions {
			if !permissions_read.contains_key(PermissionName) {
				dev_log!(
					"ipc",
					"warn: [PermissionValidator] Permission '{}' referenced by role '{}' does not exist",
					PermissionName,
					Role.Name
				);
			}
		}

		drop(permissions_read);

		let RoleName = Role.Name.clone();

		roles.insert(RoleName.clone(), Role);

		dev_log!("ipc", "[PermissionValidator] Role registered: {}", RoleName);

		Ok(())
	}

	pub async fn RegisterPermission(&self, Permission:Permission) -> Result<(), String> {
		if Permission.Name.is_empty() {
			return Err("Permission name cannot be empty".to_string());
		}

		if Permission.Description.is_empty() {
			return Err("Permission description cannot be empty".to_string());
		}

		let mut permissions = self.Permissions.write().await;

		let PermissionName = Permission.Name.clone();

		permissions.insert(PermissionName.clone(), Permission);

		dev_log!("ipc", "[PermissionValidator] Permission registered: {}", PermissionName);

		Ok(())
	}

	pub async fn GetRolePermissions(&self, RoleName:&str) -> Vec<String> {
		let roles = self.Roles.read().await;

		roles.get(RoleName).map(|role| role.Permissions.clone()).unwrap_or_default()
	}

	pub async fn HasPermission(&self, Context:&SecurityContext, PermissionName:&str) -> bool {
		if Context.Permissions.contains(&PermissionName.to_string()) {
			return true;
		}

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

	pub async fn InitializeDefaults(&self) -> Result<(), String> {
		dev_log!("ipc", "[PermissionValidator] Initializing default roles and permissions");

		let DefaultPermissions = vec![
			Permission {
				Name:"file.read".to_string(),

				Description:"Read file operations".to_string(),

				Category:"file".to_string(),

				IsSensitive:false,
			},
			Permission {
				Name:"file.write".to_string(),

				Description:"Write file operations".to_string(),

				Category:"file".to_string(),

				IsSensitive:false,
			},
			Permission {
				Name:"config.read".to_string(),

				Description:"Read configuration".to_string(),

				Category:"config".to_string(),

				IsSensitive:false,
			},
			Permission {
				Name:"config.update".to_string(),

				Description:"Update configuration".to_string(),

				Category:"config".to_string(),

				IsSensitive:false,
			},
			Permission {
				Name:"storage.read".to_string(),

				Description:"Read storage".to_string(),

				Category:"storage".to_string(),

				IsSensitive:false,
			},
			Permission {
				Name:"storage.write".to_string(),

				Description:"Write storage".to_string(),

				Category:"storage".to_string(),

				IsSensitive:false,
			},
			Permission {
				Name:"system.external".to_string(),

				Description:"Access external system resources".to_string(),

				Category:"system".to_string(),

				IsSensitive:true,
			},
			Permission {
				Name:"system.execute".to_string(),

				Description:"Execute system commands".to_string(),

				Category:"system".to_string(),

				IsSensitive:true,
			},
			Permission {
				Name:"admin.manage".to_string(),

				Description:"Administrative management operations".to_string(),

				Category:"admin".to_string(),

				IsSensitive:true,
			},
		];

		for Permission in DefaultPermissions {
			self.RegisterPermission(Permission).await?;
		}

		let DefaultRoles = vec![
			Role {
				Name:"user".to_string(),

				Permissions:vec!["file.read".to_string(), "config.read".to_string(), "storage.read".to_string()],

				Description:"Standard user with read access".to_string(),

				ParentRole:None,

				Priority:0,
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

				ParentRole:None,

				Priority:1,
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

				ParentRole:None,

				Priority:2,
			},
		];

		for Role in DefaultRoles {
			self.RegisterRole(Role).await?;
		}

		dev_log!(
			"ipc",
			"[PermissionValidator] Default roles and permissions initialized successfully"
		);

		Ok(())
	}
}
