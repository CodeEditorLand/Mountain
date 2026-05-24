//! Standard `developer` role - read + write across files and
//! storage; read-only on config.

use crate::IPC::Permission::Role::ManageRole::Struct;

pub fn Fn() -> Role::Struct {
	Role::Struct::New(
		"developer".to_string(),
		vec![
			"file.read".to_string(),
			"file.write".to_string(),
			"config.read".to_string(),
			"storage.read".to_string(),
			"storage.write".to_string(),
		],
		"Developer with read/write access".to_string(),
	)
}
