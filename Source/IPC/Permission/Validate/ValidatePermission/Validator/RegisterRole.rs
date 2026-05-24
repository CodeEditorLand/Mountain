//! `Validator::RegisterRole`

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

pub fn Fn(This:&Struct, Role:Role) -> Result<(), String> {
	if Role.Name.is_empty() {
		return Err("Role name cannot be empty".to_string());
	}

	let mut roles = This.Roles.write().await;

	let permissions_read = This.Permissions.read().await;

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
