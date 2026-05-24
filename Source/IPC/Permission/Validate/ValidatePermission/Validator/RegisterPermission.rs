//! `Validator::RegisterPermission`

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

pub fn Fn(This:&Struct, Permission:Permission) -> Result<(), String> {
	if Permission.Name.is_empty() {
		return Err("Permission name cannot be empty".to_string());
	}

	if Permission.Description.is_empty() {
		return Err("Permission description cannot be empty".to_string());
	}

	let mut permissions = This.Permissions.write().await;

	let PermissionName = Permission.Name.clone();

	permissions.insert(PermissionName.clone(), Permission);

	dev_log!("ipc", "[PermissionValidator] Permission registered: {}", PermissionName);

	Ok(())
}
