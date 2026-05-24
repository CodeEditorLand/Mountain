//! `Permission::SetSensitive`

use serde::{Deserialize, Serialize};

use super::Struct;

pub fn Fn(mut self) -> Struct {
	self.IsSensitive = true;

	self
}
