//! `Validator::InitializeDefaults`

use std::{
	collections::HashMap,
	sync::Arc,
	time::{Duration, SystemTime},
};

use tokio::sync::RwLock;

use super::Struct;
use crate::{
	IPC::Permission::{
		Role::ManageRole::{Permission::Struct as Permission, Role::Struct as Role},
		Validate::ValidatePermission::SecurityContext::Struct as SecurityContext,
	},
	dev_log,
};

pub fn Fn(This:&Struct) -> Result<(), String> {
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
		This.RegisterPermission(Permission).await?;
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
		This.RegisterRole(Role).await?;
	}

	dev_log!(
		"ipc",
		"[PermissionValidator] Default roles and permissions initialized successfully"
	);

	Ok(())
}
