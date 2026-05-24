//! `Role::New`

use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use super::Struct;
use crate::dev_log;

pub fn Fn(Name:String, Permissions:Vec<String>, Description:String) -> Struct {
	let UniquePermissions:Vec<String> = Permissions.into_iter().collect::<HashSet<String>>().into_iter().collect();

	Self { Name, Permissions:UniquePermissions, Description, ParentRole:None, Priority:0 }
}
