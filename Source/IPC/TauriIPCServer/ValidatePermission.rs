//! RBAC permission check for an IPC operation against a security
//! context. Body of `PermissionManager::validate_permission`.

use super::{PermissionManager, SecurityContext, SecurityEvent, SecurityEventType};

pub(crate) async fn Fn(
	Manager:&PermissionManager,

	operation:&str,

	context:&SecurityContext,
) -> Result<(), String> {
	// Check if operation requires specific permissions
	let required_permissions = Manager.get_required_permissions(operation).await;

	if required_permissions.is_empty() {
		return Ok(()); // No specific permissions required
	}

	// Check if user has required permissions
	let mut user_permissions:Vec<String> = context.permissions.iter().cloned().collect();

	for role in context.roles.iter() {
		let role_perms = Manager.get_role_permissions(role).await;

		user_permissions.extend(role_perms);
	}

	for required in required_permissions {
		if !user_permissions.contains(&required) {
			return Err(format!("Missing permission: {}", required));
		}
	}

	// Log successful access
	Manager
		.log_security_event(SecurityEvent {
			event_type:SecurityEventType::AccessGranted,
			user_id:context.user_id.clone(),
			operation:operation.to_string(),
			timestamp:std::time::SystemTime::now(),
			details:Some(format!("Access granted for operation: {}", operation)),
		})
		.await;

	Ok(())
}
