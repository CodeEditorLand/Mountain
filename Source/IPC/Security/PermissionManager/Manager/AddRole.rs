//! `Manager::AddRole`

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

pub fn Fn(This:&Struct, role:Role) {
		let role_name = role.name.clone();

		let mut roles = This.roles.write().await;

		roles.insert(role_name.clone(), role);

		dev_log!("ipc", "[PermissionManager] Added role: {}", role_name);
	}
