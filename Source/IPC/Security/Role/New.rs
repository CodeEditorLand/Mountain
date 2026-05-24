//! `Role::New`

use super::Struct;
use serde::{Deserialize, Serialize};

pub fn Fn(name:String, permissions:Vec<String>, description:String) -> Struct {
		Self { name, permissions, description }
	}
