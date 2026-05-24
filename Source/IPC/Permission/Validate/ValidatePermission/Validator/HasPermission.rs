//! `Validator::HasPermission`

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

pub fn Fn(This:&Struct, Context:&SecurityContext, PermissionName:&str) -> bool {
	if Context.Permissions.contains(&PermissionName.to_string()) {
		return true;
	}

	let roles = This.Roles.read().await;

	for RoleName in &Context.Roles {
		if let Some(role) = roles.get(RoleName) {
			if role.Permissions.contains(&PermissionName.to_string()) {
				return true;
			}
		}
	}

	false
}
