
//! Standard `admin` role - full access including system /
//! external / execute and `role.manage` for changing role
//! definitions at runtime.

use crate::IPC::Permission::Role::ManageRole::Role;

pub fn Fn() -> Role::Struct {
	Role::Struct::New(
		"admin".to_string(),
		vec![
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
		"Administrator with full access".to_string(),
	)
	.AddPermission("role.manage".to_string())
}
