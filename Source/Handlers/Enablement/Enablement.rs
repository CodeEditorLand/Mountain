
// Defines the logic handlers for managing extension enablement states.

#![allow(non_snake_case, non_camel_case_types)]

use Common::IpcEffect::ProxyConfiguration as ProxyTarget;
use log::{debug, error, info, warn};
use serde_json::{Value, json};
use tauri::{AppHandle, Manager, Runtime};
use vs_platform_extensions_common_extensions::{
	EnablementState as VsEnablementState,
	ExtensionIdentifier as VsExtensionIdentifier,
};

use crate::Handlers::ErrorUtils;
use crate::Vine; // The gRPC communication module

/// Logic to retrieve the enablement state for a given extension.
/// Currently, this is a STUB and always returns `EnabledGlobally`.
pub async fn HandleGetEnablementStateLogic<R:Runtime>(
	_ApplicationHandle:AppHandle<R>,
	ExtensionIdentifier:VsExtensionIdentifier,
) -> Result<Value, String> {
	debug!(
		"[EnablementHandler Logic] GetState request for extension ID: '{}'",
		ExtensionIdentifier.Value
	);

	// This is where the actual logic to check persistent storage or in-memory cache
	// would go. For now, it's stubbed to always return enabled.
	let CurrentEnablementStateEnum = VsEnablementState::EnabledGlobally;
	let ResponseStateNumber = CurrentEnablementStateEnum as u32;

	warn!(
		"[EnablementHandler Logic] STUB: Returning mock enablement state ({:?}) for extension '{}'.",
		CurrentEnablementStateEnum, ExtensionIdentifier.Value
	);

	Ok(json!(ResponseStateNumber))
}

/// Logic to set the enablement state for one or more extensions.
/// This function simulates the update and notifies the Cocoon sidecar of the
/// change.
pub async fn HandleSetEnablementLogic<R:Runtime>(
	ApplicationHandle:AppHandle<R>,
	ExtensionsToUpdate:Vec<VsExtensionIdentifier>,
	NewStateEnum:VsEnablementState,
) -> Result<Value, String> {
	info!(
		"[EnablementHandler Logic] SetEnablement for {} extensions to state: {:?}",
		ExtensionsToUpdate.len(),
		NewStateEnum
	);

	let mut Results:Vec<bool> = Vec::new();
	let mut ChangedExtensionsForNotification:Vec<Value> = Vec::new();

	for ExtensionId in ExtensionsToUpdate {
		// Here would be the logic to persist the new state.
		// For the stub, we just assume it was successful.
		info!(
			"[EnablementHandler Logic] STUB: Successfully processed enablement state for '{}' to {:?}.",
			ExtensionId.Value, NewStateEnum
		);
		Results.push(true);

		// Prepare the DTO for the notification to Cocoon.
		ChangedExtensionsForNotification.push(json!({
			"id": { "value": ExtensionId.Value, "uuid": ExtensionId.Uuid },
			"state": NewStateEnum as u32
		}));
	}

	if !ChangedExtensionsForNotification.is_empty() {
		let NotificationMethod = format!(
			"{}.$acceptEnablementChanged",
			ProxyTarget::ExtHostExtensionEnablement.GetTargetPrefix()
		);
		let NotificationParameters = json!([ChangedExtensionsForNotification]);

		info!(
			"[EnablementHandler Logic] Notifying Cocoon of enablement changes for {} extensions.",
			notification_params[0].as_array().map_or(0, |a| a.len())
		);
		if let Err(Error) =
			Vine::SendNotification("cocoon-main".to_string(), NotificationMethod, NotificationParameters).await
		{
			error!(
				"[EnablementHandler Logic] Failed to send $acceptEnablementChanged to Cocoon: {}",
				Error
			);
			// This failure might be surfaced to the user or handled in some
			// other way.
		}
	}

	Ok(json!(Results))
}
