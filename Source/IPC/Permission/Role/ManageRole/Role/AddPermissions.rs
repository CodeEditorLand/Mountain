//! `Role::AddPermissions`

use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use super::Struct;
use crate::dev_log;

pub fn Fn(mut self, Permissions:impl IntoIterator<Item = String>) -> Struct {
	for Permission in Permissions {
		if !self.Permissions.contains(&Permission) {
			self.Permissions.push(Permission.clone());

			dev_log!("ipc", "[Role] Added permission '{}' to role '{}'", Permission, self.Name);
		}
	}

	self
}
