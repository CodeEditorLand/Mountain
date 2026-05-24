//! `Manager::New`

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

pub fn Fn() -> Struct {
		dev_log!("ipc", "[PermissionManager] Creating new PermissionManager instance");

		Self {
			roles:Arc::new(RwLock::new(HashMap::new())),

			permissions:Arc::new(RwLock::new(HashMap::new())),

			audit_log:Arc::new(RwLock::new(Vec::new())),
		}
	}
