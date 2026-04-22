#![allow(non_snake_case)]

//! Extension host domain handlers for Wind IPC.

use std::{path::PathBuf, sync::Arc};

use serde_json::{Value, json};
use tauri::AppHandle;
use CommonLibrary::ExtensionManagement::ExtensionManagementService::ExtensionManagementService;

use crate::{ExtensionManagement::VsixInstaller, RunTime::ApplicationRunTime::ApplicationRunTime, Vine, dev_log};

/// Cocoon sidecar id (matches `CocoonManagement::COCOON_SIDE_CAR_IDENTIFIER`).
const COCOON_SIDE_CAR_IDENTIFIER:&str = "cocoon-main";

/// Timeout for fire-and-forget `$deltaExtensions` notifications; long enough
/// to survive a busy Cocoon but short enough that install feedback isn't
/// blocked on a stalled extension host.
const COCOON_DELTA_TIMEOUT_MS:u64 = 10_000;

/// Tell Cocoon to diff the extension registry by the provided descriptors.
/// Fire-and-forget: a missing Cocoon (LAND_SPAWN_COCOON=false) or a transient
/// RPC failure is logged but does not fail the install/uninstall IPC call.
fn NotifyCocoonDeltaExtensions(ToAdd:Vec<Value>, ToRemove:Vec<Value>) {
	tokio::spawn(async move {
		let Parameters = json!({
			"toAdd": ToAdd,
			"toRemove": ToRemove,
		});

		match Vine::Client::SendRequest(
			&COCOON_SIDE_CAR_IDENTIFIER.to_string(),
			"$deltaExtensions".to_string(),
			Parameters,
			COCOON_DELTA_TIMEOUT_MS,
		)
		.await
		{
			Ok(Response) => {
				dev_log!("extensions", "$deltaExtensions applied: {}", Response);
			},
			Err(Error) => {
				// Non-fatal - most commonly hit when Cocoon is intentionally
				// off (LAND_SPAWN_COCOON=false) or still booting.
				dev_log!("extensions", "warn: $deltaExtensions failed (non-fatal): {}", Error);
			},
		}
	});
}

/// Return metadata for all scanned extensions.
pub async fn handle_extensions_get_all(Runtime:Arc<ApplicationRunTime>) -> Result<Value, String> {
	let Extensions = Runtime
		.Environment
		.GetExtensions()
		.await
		.map_err(|Error| format!("extensions:getAll failed: {}", Error))?;

	let ExtensionCount = Extensions.len();
	let Response = json!(Extensions);
	let PayloadBytes = serde_json::to_string(&Response).map(|S| S.len()).unwrap_or(0);

	dev_log!(
		"extensions",
		"extensions:getAll returning {} extensions ({} bytes serialized)",
		ExtensionCount,
		PayloadBytes
	);
	if let Some(First) = Extensions.first() {
		dev_log!(
			"extensions",
			"extensions:getAll sample: {}",
			serde_json::to_string(First)
				.unwrap_or_default()
				.chars()
				.take(300)
				.collect::<String>()
		);
	} else if ExtensionCount == 0 {
		dev_log!(
			"extensions",
			"warn: extensions:getAll returning EMPTY - scan has not populated ScannedExtensions, or all inserts were \
			 rejected"
		);
	}
	Ok(Response)
}

/// Return metadata for a single extension by ID.
pub async fn handle_extensions_get(Runtime:Arc<ApplicationRunTime>, Args:Vec<Value>) -> Result<Value, String> {
	let Id = Args
		.first()
		.and_then(|V| V.as_str())
		.ok_or_else(|| "extensions:get requires string id as first argument".to_string())?
		.to_string();

	let Extension = Runtime
		.Environment
		.GetExtension(Id)
		.await
		.map_err(|Error| format!("extensions:get failed: {}", Error))?;

	Ok(Extension.unwrap_or(Value::Null))
}

/// Check whether an extension is currently active (scanned and present).
pub async fn handle_extensions_is_active(Runtime:Arc<ApplicationRunTime>, Args:Vec<Value>) -> Result<Value, String> {
	let Id = Args
		.first()
		.and_then(|V| V.as_str())
		.ok_or_else(|| "extensions:isActive requires string id as first argument".to_string())?
		.to_string();

	let Extension = Runtime
		.Environment
		.GetExtension(Id)
		.await
		.map_err(|Error| format!("extensions:isActive failed: {}", Error))?;

	Ok(json!(Extension.is_some()))
}

// ---------------------------------------------------------------------------
// Atom K2 / K3 - local VSIX install & uninstall
// ---------------------------------------------------------------------------
//
// Wind ships two kinds of install call:
//   1. from the "Install from VSIX…" dialog - args[0] is a URI string pointing
//      at a local `.vsix`.
//   2. from the Extensions sidebar row - args[0] is a gallery identifier, which
//      we do not support (no marketplace backend) and which returns null with a
//      clear log line.
//
// Uninstall always receives an identifier string (args[0]) matching one of
// the entries we previously returned from `extensions:getInstalled`.
//
// Every successful install / uninstall:
//   - mutates `ApplicationState.Extension.ScannedExtensions`,
//   - emits `sky://extensions/installed` or `…/uninstalled` so Wind re-fetches
//     the sidebar,
//   - logs one summary line under the `extensions` tag.

/// User-scope install destination for VSIX unpacks. Matches the user-scope
/// scan path in `Binary/Extension/ScanPathConfigure.rs` so VSIX-installed
/// extensions are discovered on the next Mountain boot without a sync step.
///
/// Atom V1: honours `LAND_USER_EXTENSIONS_DIR` (from `.env.Land.Extensions`).
/// Resolution order:
///   1. `$LAND_USER_EXTENSIONS_DIR` — explicit per-operator override.
///      Leading `~/` expands against `$HOME`.
///   2. `$HOME/.land/extensions` — VS Code-style user-scope default.
///   3. `./extensions` — fallback when `$HOME` is unavailable (container,
///      restricted environment). `fs::create_dir_all` runs on install so
///      this works even if the cwd is read-only at scan time.
fn UserExtensionDirectory() -> PathBuf {
	if let Ok(Override) = std::env::var("LAND_USER_EXTENSIONS_DIR") {
		if let Some(Stripped) = Override.strip_prefix("~/") {
			if let Some(HomeDirectory) = dirs::home_dir() {
				return HomeDirectory.join(Stripped);
			}
		}
		return PathBuf::from(Override);
	}

	if let Some(HomeDirectory) = dirs::home_dir() {
		return HomeDirectory.join(".land/extensions");
	}

	PathBuf::from("extensions")
}

fn VsixPathFromArgs(Args:&[Value]) -> Option<PathBuf> {
	let Raw = Args.first()?;

	let RawString = if let Some(AsStr) = Raw.as_str() {
		AsStr.to_string()
	} else if let Some(AsObject) = Raw.as_object() {
		// Wind can pass a UriComponents object; pull the conventional fields.
		if let Some(External) = AsObject.get("external").and_then(|V| V.as_str()) {
			External.to_string()
		} else if let Some(Path) = AsObject.get("path").and_then(|V| V.as_str()) {
			Path.to_string()
		} else {
			return None;
		}
	} else {
		return None;
	};

	if let Ok(Parsed) = url::Url::parse(&RawString) {
		if Parsed.scheme() == "file" {
			return Some(Parsed.to_file_path().unwrap_or_else(|_| PathBuf::from(Parsed.path())));
		}
	}

	Some(PathBuf::from(RawString))
}

/// `extensions:install` - local VSIX only. Gallery installs are declined.
pub async fn handle_extensions_install(
	ApplicationHandle:AppHandle,
	Runtime:Arc<ApplicationRunTime>,
	Args:Vec<Value>,
) -> Result<Value, String> {
	let OTELStart = crate::IPC::DevLog::NowNano();

	let VsixPath = match VsixPathFromArgs(&Args) {
		Some(Path) => Path,
		None => {
			dev_log!("extensions", "extensions:install no-op: args[0] missing or non-file URI");
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

	// Register in ScannedExtensions so `GetExtensions()` returns the new
	// extension on the very next read, without a disk rescan.
	Runtime
		.Environment
		.ApplicationState
		.Extension
		.ScannedExtensions
		.AddOrUpdate(Outcome.Identifier.clone(), Outcome.Description.clone());

	let Descriptor = serde_json::to_value(&Outcome.Description).unwrap_or(Value::Null);

	// Atom K4: tell Cocoon to hot-activate the new extension - no window
	// reload required. Cocoon's `$deltaExtensions` handler adds the
	// descriptor to its registry, indexes `activationEvents`, and fires its
	// own `deltaExtensions` event so any live consumers (ExtensionsNamespace
	// subscribers) refresh.
	NotifyCocoonDeltaExtensions(vec![Descriptor.clone()], Vec::new());

	// Broadcast so Wind re-renders the sidebar without a workbench reload.
	{
		use tauri::Emitter;

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

	// ILocalExtension envelope - matches `handle_extensions_get_installed`
	// so VS Code's ExtensionEnablementService merges it into the sidebar.
	Ok(json!({
		"type": 1,
		"isBuiltin": false,
		"identifier": { "id": Outcome.Identifier },
		"manifest": Descriptor,
		"location": {
			"scheme": "file",
			"path": Outcome.InstalledAt.to_string_lossy(),
			"authority": "",
		},
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

/// `extensions:uninstall` - remove install dir, clear registry entry.
pub async fn handle_extensions_uninstall(
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

	// Atom K4: symmetric with install - tell Cocoon to drop the extension
	// from its registry so live consumers (`onDidChangeExtensions`) refresh.
	if !RemovedDescriptor.is_null() {
		NotifyCocoonDeltaExtensions(Vec::new(), vec![RemovedDescriptor]);
	}

	{
		use tauri::Emitter;

		if let Err(Error) = ApplicationHandle.emit(
			"sky://extensions/uninstalled",
			json!({
				"identifier": Identifier,
				"location": InstallDirectory.as_ref().map(|Value| Value.to_string_lossy().to_string()),
			}),
		) {
			dev_log!("extensions", "warn: failed to emit sky://extensions/uninstalled: {}", Error);
		}
	}

	dev_log!("extensions", "extensions:uninstall succeeded: {}", Identifier);

	crate::otel_span!(
		"extensions:uninstall:ok",
		OTELStart,
		&[("extension.identifier", Identifier.as_str())]
	);

	Ok(Value::Bool(true))
}
