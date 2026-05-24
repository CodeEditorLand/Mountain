//! `Role::AddPermission`

use super::Struct;
use serde::{Deserialize, Serialize};

pub fn Fn(This:&mut Struct, permission:String) {
		if !This.HasPermission(&permission) {
			This.permissions.push(permission);
		}
	}
