pub mod New;
pub mod CreateSecurityContext;
pub mod ValidatePermission;
pub mod RegisterRole;
pub mod RegisterPermission;
pub mod GetRolePermissions;
pub mod HasPermission;
pub mod InitializeDefaults;

use std::{
	collections::HashMap,
	sync::Arc,
	time::{Duration, SystemTime},
};

use tokio::sync::RwLock;

use crate::{
	IPC::Permission::{
		Role::ManageRole::{Permission::Struct as Permission, Role::Struct as Role},
		Validate::ValidatePermission::SecurityContext::Struct as SecurityContext,
	},
	dev_log,
};

pub struct Struct {
	pub(super) Roles:Arc<RwLock<HashMap<String, Role>>>,

	pub(super) Permissions:Arc<RwLock<HashMap<String, Permission>>>,

	pub(super) OperationPermissions:HashMap<String, Vec<String>>,

	pub(super) ValidationTimeoutMillis:u64,
}
