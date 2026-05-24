//! `Permission::GetCategory`

use serde::{Deserialize, Serialize};

use super::Struct;

pub fn Fn(This:&Struct) -> String {
	if let Some(pos) = This.Name.rfind('.') {
		This.Name[..pos].to_string()
	} else {
		"unknown".to_string()
	}
}
