//! `extensions:install` IPC handler - local VSIX only. Gallery installs are
//! declined (Land has no marketplace backend) and return `null`.
//!
//! Sequence:
//!   1. Resolve the VSIX path from `Arguments[0]` (string or UriComponents).
//!   2. Reject non-`.vsix` files.
//!   3. Unpack into the user-scope extension directory via
//!      `VsixInstaller::InstallVsix`.
//!   4. Register with `ScannedExtensions` so `GetExtensions()` reflects the
//!      install on the next read.
//!   5. Fire-and-forget `$deltaExtensions` + `$activateByEvent` to Cocoon so
//!      the extension activates without a workbench reload.
//!   6. Emit `sky://extensions/installed` so Wind refreshes the sidebar.
//!   7. Return an `ILocalExtension` envelope shaped for VS Code's
//!      ExtensionEnablementService sidebar merge path.

use std::sync::Arc;

use serde_json::{Value, json};
use tauri::{AppHandle, Emitter};

use crate::{
	ExtensionManagement::VsixInstaller,
	IPC::{
		UriComponents::FromFilePath::Fn as UriFromFilePath,
		WindServiceHandlers::Extension::{
			NotifyCocoonDeltaExtensions::Fn as NotifyCocoonDeltaExtensions,
			UserExtensionDirectory::Fn as UserExtensionDirectory,
			VsixPathFromArgs::Fn as VsixPathFromArgs,
		},
	},
	RunTime::ApplicationRunTime::ApplicationRunTime,
	dev_log,
};

pub async fn Fn(
	ApplicationHandle:AppHandle,

	Runtime:Arc<ApplicationRunTime>,

	Args:Vec<Value>,
) -> Result<Value, String> {
	let OTELStart = crate::IPC::DevLog::NowNano::Fn();

	let VsixPath = match VsixPathFromArgs(&Args) {
		Some(Path) => Path,

		None => {
			dev_log!("extensions", "extensions:install no-op: Arguments[0] missing or non-file URI");

			crate::otel_span!("extensions:install:noop-missing-arg", OTELStart);

			return Ok(Value::Null);
		},
	};

	if VsixPath.extension().and_then(|Value| Value.to_str()) != Some("vsix") {
		dev_log!("extensions", "extensions:install no-op: {} is not a .vsix", VsixPath.display());

		crate::otel_span!("extensions:install:noop-not-vsix", OTELStart);

		return Ok(Value::Null);
	}

	let InstallRoot = UserExtensionDirectory();

	let Outcome = tokio::task::spawn_blocking(move || VsixInstaller::InstallVsix(&VsixPath, &InstallRoot))
		.await
		.map_err(|Error| format!("extensions:install join error: {}", Error))?
		.map_err(|Error| format!("extensions:install failed: {}", Error))?;

	Runtime
		.Environment
		.ApplicationState
		.Extension
		.ScannedExtensions
		.AddOrUpdate(Outcome.Identifier.clone(), Outcome.Description.clone());

	let Descriptor = serde_json::to_value(&Outcome.Description).unwrap_or(Value::Null);

	NotifyCocoonDeltaExtensions(vec![Descriptor.clone()], Vec::new());

	if let Err(Error) = ApplicationHandle.emit(
		"sky://extensions/installed",
		json!({
			"identifier": Outcome.Identifier,
			"version": Outcome.Version,
			"location": Outcome.InstalledAt.to_string_lossy(),
		}),
	) {
		dev_log!("extensions", "warn: failed to emit sky://extensions/installed: {}", Error);
	}

	dev_log!(
		"extensions",
		"extensions:install succeeded: {} v{} at {}",
		Outcome.Identifier,
		Outcome.Version,
		Outcome.InstalledAt.display()
	);

	crate::otel_span!(
		"extensions:install:ok",
		OTELStart,
		&[
			("extension.identifier", Outcome.Identifier.as_str()),
			("extension.version", Outcome.Version.as_str()),
		]
	);

	// ILocalExtension envelope - matches `ExtensionsGetInstalled`
	// so VS Code's ExtensionEnablementService merges it into the sidebar.
	// `location` must carry `$mid: 1` so the renderer's `URI.revive()`
	// runs; otherwise `resources.joinPath(local.location, …)` hits
	// `uri.with is not a function`. Routed through `UriFromFilePath` so
	// the marker never drops off.
	Ok(json!({
		"type": 1,
		"isBuiltin": false,
		"identifier": { "id": Outcome.Identifier },
		"manifest": Descriptor,
		"location": UriFromFilePath(Outcome.InstalledAt.to_string_lossy()),
		"targetPlatform": "undefined",
		"isValid": true,
		"validations": [],
		"preRelease": false,
		"isWorkspaceScoped": false,
		"isMachineScoped": false,
		"isApplicationScoped": false,
		"publisherId": null,
		"isPreReleaseVersion": false,
		"hasPreReleaseVersion": false,
		"private": false,
		"updated": false,
		"pinned": false,
		"forceAutoUpdate": false,
		"source": "vsix",
		"size": 0,
	}))
}
