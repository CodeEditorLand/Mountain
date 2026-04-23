#![allow(non_snake_case)]
//! `extensions:uninstall` IPC handler - removes the install directory,
//! clears the registry entry, and notifies Cocoon + Wind. Symmetric with
//! `ExtensionInstall`.

use std::{path::PathBuf, sync::Arc};

use serde_json::{Value, json};
use tauri::{AppHandle, Emitter};

use crate::{
	ExtensionManagement::VsixInstaller,
	IPC::WindServiceHandlers::Extension::NotifyCocoonDeltaExtensions::NotifyCocoonDeltaExtensions,
	RunTime::ApplicationRunTime::ApplicationRunTime,
	dev_log,
};

pub async fn ExtensionUninstall(
	ApplicationHandle:AppHandle,
	Runtime:Arc<ApplicationRunTime>,
	Args:Vec<Value>,
) -> Result<Value, String> {
	let OTELStart = crate::IPC::DevLog::NowNano();

	let Identifier = match Args.first().and_then(|Value| {
		Value
			.as_str()
			.map(str::to_owned)
			.or_else(|| Value.get("id").and_then(|Inner| Inner.as_str()).map(str::to_owned))
	}) {
		Some(Value) => Value,
		None => {
			dev_log!("extensions", "extensions:uninstall no-op: args[0] missing identifier");
			crate::otel_span!("extensions:uninstall:noop-missing-id", OTELStart);
			return Ok(Value::Null);
		},
	};

	let Descriptor = Runtime
		.Environment
		.ApplicationState
		.Extension
		.ScannedExtensions
		.Get(&Identifier);

	let InstallDirectory = Descriptor
		.as_ref()
		.and_then(|Description| Description.ExtensionLocation.get("path").and_then(|V| V.as_str()))
		.map(PathBuf::from);

	if let Some(Directory) = InstallDirectory.clone() {
		let DirectoryForBlocking = Directory.clone();

		tokio::task::spawn_blocking(move || VsixInstaller::UninstallExtension(&DirectoryForBlocking))
			.await
			.map_err(|Error| format!("extensions:uninstall join error: {}", Error))?
			.map_err(|Error| format!("extensions:uninstall failed: {}", Error))?;
	}

	let RemovedDescriptor = Descriptor
		.as_ref()
		.map(|Description| serde_json::to_value(Description).unwrap_or(Value::Null))
		.unwrap_or(Value::Null);

	Runtime
		.Environment
		.ApplicationState
		.Extension
		.ScannedExtensions
		.Remove(&Identifier);

	if !RemovedDescriptor.is_null() {
		NotifyCocoonDeltaExtensions(Vec::new(), vec![RemovedDescriptor]);
	}

	if let Err(Error) = ApplicationHandle.emit(
		"sky://extensions/uninstalled",
		json!({
			"identifier": Identifier,
			"location": InstallDirectory.as_ref().map(|Value| Value.to_string_lossy().to_string()),
		}),
	) {
		dev_log!("extensions", "warn: failed to emit sky://extensions/uninstalled: {}", Error);
	}

	dev_log!("extensions", "extensions:uninstall succeeded: {}", Identifier);

	crate::otel_span!(
		"extensions:uninstall:ok",
		OTELStart,
		&[("extension.identifier", Identifier.as_str())]
	);

	Ok(Value::Bool(true))
}
