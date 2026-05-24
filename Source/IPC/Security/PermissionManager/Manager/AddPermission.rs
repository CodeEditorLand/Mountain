//! `Manager::AddPermission`

use super::Struct;
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

pub fn Fn(This:&Struct, permission:Permission) {
		let permission_name = permission.name.clone();

		let mut permissions = This.permissions.write().await;

		permissions.insert(permission_name.clone(), permission);

		dev_log!("ipc", "[PermissionManager] Added permission: {}", permission_name);
	}
