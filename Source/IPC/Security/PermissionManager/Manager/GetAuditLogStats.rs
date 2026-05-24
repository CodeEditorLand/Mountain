//! `Manager::GetAuditLogStats`

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

pub fn Fn(This:&Struct) -> (usize, Vec<(&'static str, usize)>) {
		let audit_log = This.audit_log.read().await;

		let mut type_counts:Vec<(&'static str, usize)> = vec![
			("PermissionDenied", 0),
			("AccessGranted", 0),
			("ConfigurationChange", 0),
			("SecurityViolation", 0),
			("PerformanceAnomaly", 0),
		];

		for event in audit_log.iter() {
			let type_name = match event.event_type {
				SecurityEventType::PermissionDenied => "PermissionDenied",

				SecurityEventType::AccessGranted => "AccessGranted",

				SecurityEventType::ConfigurationChange => "ConfigurationChange",

				SecurityEventType::SecurityViolation => "SecurityViolation",

				SecurityEventType::PerformanceAnomaly => "PerformanceAnomaly",
			};

			if let Some((_, count)) = type_counts.iter_mut().find(|(name, _)| *name == type_name) {
				*count += 1;
			}
		}

		(audit_log.len(), type_counts)
	}
