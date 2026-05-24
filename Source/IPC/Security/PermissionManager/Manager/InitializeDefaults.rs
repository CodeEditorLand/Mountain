//! `Manager::InitializeDefaults`

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

pub fn Fn(This:&Struct) {
		dev_log!("ipc", "[PermissionManager] Initializing default roles and permissions");

		let mut permissions = This.permissions.write().await;

		let mut roles = This.roles.write().await;

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
