//! `Manager::ClearAuditLog`

use super::Struct;
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

pub fn Fn(This:&Struct) {
		let mut audit_log = This.audit_log.write().await;

		audit_log.clear();

		dev_log!("ipc", "[PermissionManager] Audit log cleared");
	}
