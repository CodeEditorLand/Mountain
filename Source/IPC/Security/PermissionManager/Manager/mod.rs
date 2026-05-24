pub mod New;
pub mod ValidatePermission;
pub mod LogSecurityEvent;
pub mod GetAuditLog;
pub mod InitializeDefaults;
pub mod AddRole;
pub mod AddPermission;
pub mod ClearAuditLog;
pub mod GetAuditLogStats;

use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;
use crate::{
	IPC::Security::{
		Permission::Permission,
		PermissionManager::{
			SecurityContext::Struct as SecurityContext,
			SecurityEvent::Struct as SecurityEvent,
			SecurityEventType::Enum as SecurityEventType,
		},
		Role::Role,
	},
	dev_log,
};

pub struct Struct {
	pub(super) roles:Arc<RwLock<HashMap<String, Role>>>,

	pub(super) permissions:Arc<RwLock<HashMap<String, Permission>>>,

	pub(super) audit_log:Arc<RwLock<Vec<SecurityEvent>>>,
}
