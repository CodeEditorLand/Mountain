#![allow(non_snake_case)]

//! Standard `user` role - read-only access to file, config,
//! and storage subsystems. The default role assigned when no
//! roles are supplied in a `SecurityContext`.

use crate::IPC::Permission::Role::ManageRole::Role;

pub fn Fn() -> Role::Struct {
	Role::Struct::New(
		"user".to_string(),
		vec!["file.read".to_string(), "config.read".to_string(), "storage.read".to_string()],
		"Standard user with read access".to_string(),
	)
}
