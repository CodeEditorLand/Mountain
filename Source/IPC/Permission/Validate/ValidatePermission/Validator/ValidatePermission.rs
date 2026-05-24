//! `Validator::ValidatePermission`

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

pub fn Fn(This:&Struct, Operation:&str, Context:&SecurityContext) -> Result<(), String> {
	let timeout_duration = Duration::from_millis(This.ValidationTimeoutMillis);

	let result = tokio::time::timeout(timeout_duration, async {
		This.ValidatePermissionInternal(Operation, Context).await
	})
	.await;

	match result {
		Ok(validation_result) => validation_result,

		Err(_) => {
			dev_log!(
				"ipc",
				"error: [PermissionValidator] Permission validation timed out for operation: {}",
				Operation
			);

			Err("Permission validation timeout".to_string())
		},
	}
}
