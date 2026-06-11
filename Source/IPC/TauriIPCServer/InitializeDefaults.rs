//! Seeds the standard permission and role tables for RBAC. Body of
//! `PermissionManager::initialize_defaults`.

use super::{Permission, PermissionManager, Role};

pub(crate) async fn Fn(Manager:&PermissionManager) {
	let mut permissions = Manager.permissions.write().await;

	let mut roles = Manager.roles.write().await;

	// Define standard permissions
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

	// Define standard roles
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
}
