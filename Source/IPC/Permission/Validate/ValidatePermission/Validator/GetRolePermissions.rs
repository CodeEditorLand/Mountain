//! `Validator::GetRolePermissions`

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

pub fn Fn(This:&Struct, RoleName:&str) -> Vec<String> {
	let roles = This.Roles.read().await;

	roles.get(RoleName).map(|role| role.Permissions.clone()).unwrap_or_default()
}
