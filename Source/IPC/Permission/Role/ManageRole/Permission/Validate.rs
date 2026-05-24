//! `Permission::Validate`

use serde::{Deserialize, Serialize};

use super::Struct;

pub fn Fn(This:&Struct) -> Result<(), String> {
	if This.Name.is_empty() {
		return Err("Permission name cannot be empty".to_string());
	}

	if This.Name.contains(|c:char| c.is_whitespace()) {
		return Err("Permission name cannot contain whitespace".to_string());
	}

	if !This.Name.contains('.') {
		return Err("Permission name must contain a dot separating category and action".to_string());
	}

	if This.Description.is_empty() {
		return Err("Permission description cannot be empty".to_string());
	}

	if This.Category.is_empty() {
		return Err("Permission category cannot be empty".to_string());
	}

	Ok(())
}
