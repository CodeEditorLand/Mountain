//! `Role::Validate`

use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use super::Struct;
use crate::dev_log;

pub fn Fn(This:&Struct) -> Result<(), String> {
	if This.Name.is_empty() {
		return Err("Role name cannot be empty".to_string());
	}

	if This.Name.contains(|c:char| c.is_whitespace()) {
		return Err("Role name cannot contain whitespace".to_string());
	}

	if This.Description.is_empty() {
		return Err("Role description cannot be empty".to_string());
	}

	for Permission in &This.Permissions {
		if Permission.is_empty() {
			return Err("Permission name cannot be empty".to_string());
		}

		if !Permission.contains('.') {
			return Err(format!(
				"Permission '{}' must contain a dot separating category and action",
				Permission
			));
		}

		if Permission.contains(|c:char| c.is_whitespace()) {
			return Err(format!("Permission '{}' cannot contain whitespace", Permission));
		}
	}

	Ok(())
}
