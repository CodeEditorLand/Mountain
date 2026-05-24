//! `Manager::LogSecurityEvent`

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

pub fn Fn(This:&Struct, event:SecurityEvent) {
		let mut audit_log = This.audit_log.write().await;

		audit_log.push(event.clone());

		if audit_log.len() > 1000 {
			audit_log.remove(0);
		}

		match event.event_type {
			SecurityEventType::PermissionDenied => {
				dev_log!(
					"ipc",
					"warn: [SecurityEvent] Permission denied - User: {}, Operation: {}, Details: {:?}",
					event.user_id,
					event.operation,
					event.details
				);
			},

			SecurityEventType::SecurityViolation => {
				dev_log!(
					"ipc",
					"error: [SecurityEvent] Security violation - User: {}, Operation: {}, Details: {:?}",
					event.user_id,
					event.operation,
					event.details
				);
			},

			SecurityEventType::AccessGranted => {
				dev_log!(
					"ipc",
					"[SecurityEvent] Access granted - User: {}, Operation: {}",
					event.user_id,
					event.operation
				);
			},

			_ => {
				dev_log!(
					"ipc",
					"[SecurityEvent] {:?} - User: {}, Operation: {}",
					event.event_type,
					event.user_id,
					event.operation
				);
			},
		}
	}
