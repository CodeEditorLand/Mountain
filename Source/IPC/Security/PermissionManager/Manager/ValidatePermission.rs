//! `Manager::ValidatePermission`

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

pub fn Fn(This:&Struct, operation:&str, context:&SecurityContext) -> Result<(), String> {
		let required_permissions = This.get_required_permissions(operation).await;

		if required_permissions.is_empty() {
			dev_log!(
				"ipc",
				"[PermissionManager] Operation '{}' requires no special permissions",
				operation
			);

			return Ok(());
		}

		let mut user_permissions:Vec<String> = context.permissions.iter().cloned().collect();

		for role in context.roles.iter() {
			let role_perms = This.get_role_permissions(role).await;

			user_permissions.extend(role_perms);
		}

		for required in &required_permissions {
			if !user_permissions.contains(required) {
				let error = format!("Missing permission: {}", required);

				dev_log!(
					"ipc",
					"[PermissionManager] Permission denied for user '{}' on operation '{}': {}",
					context.user_id,
					operation,
					error
				);

				This.LogSecurityEvent(SecurityEvent {
					event_type:SecurityEventType::PermissionDenied,
					user_id:context.user_id.clone(),
					operation:operation.to_string(),
					timestamp:std::time::SystemTime::now(),
					details:Some(format!("Permission denied: {}", error)),
				})
				.await;

				return Err(error);
			}
		}

		This.LogSecurityEvent(SecurityEvent {
			event_type:SecurityEventType::AccessGranted,
			user_id:context.user_id.clone(),
			operation:operation.to_string(),
			timestamp:std::time::SystemTime::now(),
			details:Some(format!("Access granted for operation: {}", operation)),
		})
		.await;

		dev_log!(
			"ipc",
			"[PermissionManager] Access granted for user '{}' on operation '{}'",
			context.user_id,
			operation
		);

		Ok(())
	}
