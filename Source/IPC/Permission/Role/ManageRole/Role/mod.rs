pub mod New;
pub mod NewWithParent;
pub mod AddPermission;
pub mod AddPermissions;
pub mod HasPermission;
pub mod PermissionCount;
pub mod Validate;

use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::dev_log;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Struct {
	pub Name:String,

	pub Permissions:Vec<String>,

	pub Description:String,

	pub ParentRole:Option<String>,

	pub Priority:u32,
}
