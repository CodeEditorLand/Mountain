//! `Manager::GetAuditLog`

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

pub fn Fn(This:&Struct, limit:usize) -> Vec<SecurityEvent> {
		let audit_log = This.audit_log.read().await;

		audit_log.iter().rev().take(limit).cloned().collect()
	}
