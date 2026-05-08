#![allow(non_snake_case)]

//! Build the standard `Permission::Struct` set covering file,
//! config, storage, system, and admin categories. Sensitive
//! permissions (`config.update`, `system.*`, `admin.*`,
//! `role.manage`) are flagged so audit logging picks them up.

use crate::{IPC::Permission::Role::ManageRole::Permission, dev_log};

pub fn Fn() -> Vec<Permission::Struct> {
	dev_log!("ipc", "[ManageRole] Creating standard permissions");

	vec![
		Permission::Struct::New("file.read".to_string(), "Read file operations".to_string(), "file".to_string()),
		Permission::Struct::New(
			"file.write".to_string(),
			"Write file operations".to_string(),
			"file".to_string(),
		),
		Permission::Struct::New(
			"file.delete".to_string(),
			"Delete file operations".to_string(),
			"file".to_string(),
		),
		Permission::Struct::New(
			"config.read".to_string(),
			"Read configuration".to_string(),
			"config".to_string(),
		),
		Permission::Struct::NewSensitive(
			"config.update".to_string(),
			"Update configuration".to_string(),
			"config".to_string(),
		)
		.SetSensitive(),
		Permission::Struct::New("storage.read".to_string(), "Read storage".to_string(), "storage".to_string()),
		Permission::Struct::New("storage.write".to_string(), "Write storage".to_string(), "storage".to_string()),
		Permission::Struct::New(
			"storage.delete".to_string(),
			"Delete from storage".to_string(),
			"storage".to_string(),
		),
		Permission::Struct::NewSensitive(
			"system.external".to_string(),
			"Access external system resources".to_string(),
			"system".to_string(),
		)
		.SetSensitive(),
		Permission::Struct::NewSensitive(
			"system.execute".to_string(),
			"Execute system commands".to_string(),
			"system".to_string(),
		)
		.SetSensitive(),
		Permission::Struct::NewSensitive(
			"admin.manage".to_string(),
			"Administrative management operations".to_string(),
			"admin".to_string(),
		)
		.SetSensitive(),
		Permission::Struct::NewSensitive(
			"role.manage".to_string(),
			"Manage roles and permissions".to_string(),
			"admin".to_string(),
		)
		.SetSensitive(),
	]
}
