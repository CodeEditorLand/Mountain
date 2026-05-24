//! `Validator::New`

use std::{
	collections::HashMap,
	sync::Arc,
	time::{Duration, SystemTime},
};

use tokio::sync::RwLock;

use super::Struct;
use crate::{
	IPC::Permission::{
		Role::ManageRole::{Permission::Struct as Permission, Role::Struct as Role},
		Validate::ValidatePermission::SecurityContext::Struct as SecurityContext,
	},
	dev_log,
};

pub fn Fn(ValidationTimeoutMillis:u64) -> Struct {
	Self {
		Roles:Arc::new(RwLock::new(HashMap::new())),

		Permissions:Arc::new(RwLock::new(HashMap::new())),

		OperationPermissions:Struct::BuildOperationMapping(),

		ValidationTimeoutMillis,
	}
}
