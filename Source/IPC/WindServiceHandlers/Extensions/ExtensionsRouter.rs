//! Extension commands router - delegates all `extensions:*` IPC commands.

use std::sync::Arc;

use serde_json::{json, Value};

use super::*;
use crate::{
	IPC::WindServiceHandlers::Extensions::{
		ExtensionsGet::Fn as ExtensionsGet,
		ExtensionsGetAll::Fn as ExtensionsGetAll,
		ExtensionsGetInstalled::Fn as ExtensionsGetInstalled,
		ExtensionsIsActive::Fn as ExtensionsIsActive,
	},
	RunTime::ApplicationRunTime::ApplicationRunTime,
	Utilities::arg_string,
	dev_log,
};

/// Routes extensions commands. Returns Some(result) for handled commands,
/// None otherwise (caller falls through to next dispatch arm).
pub(crate) async fn route(
	ApplicationHandle:tauri::AppHandle,

	RunTime:Arc<ApplicationRunTime>,

	command:&str,

	Arguments:Vec<Value>,
) -> Option<Result<Value, String>> {
	match command {
		"extensions:getAll" => Some(ExtensionsGetAll(RunTime.clone()).await),

		"extensions:get" => Some(ExtensionsGet(RunTime.clone(), Arguments).await),

		"extensions:isActive" => Some(ExtensionsIsActive(RunTime.clone(), Arguments).await),

		// `extensions:activate(extensionId)` - send `$activateByEvent`
		// to Cocoon so the extension host starts the extension.
		"extensions:activate" => {
			let ExtensionId = arg_string(&Arguments, 0);

			dev_log!("extensions", "extensions:activate id={}", ExtensionId);

			if ExtensionId.is_empty() {
				Some(Ok(Value::Null))
			} else {
				let Notification = json!({
					"event": format!("onCustom:{}", ExtensionId),
					"extensionId": ExtensionId,
				});

				let _ = crate::Vine::Client::SendNotification::Fn(
					"cocoon-main".to_string(),
					"$activateByEvent".to_string(),
					Notification,
				)
				.await;

				Some(Ok(Value::Null))
			}
		},

		// `ExtensionManagementChannelClient.getInstalled` →
		// `extensions:getInstalled`. `ExtensionsGetInstalled` builds the
		// `ILocalExtension[]` wrapper; `ExtensionsGetAll` returns raw
		// manifests. Do NOT alias - payload shapes differ.
		"extensions:getInstalled" | "extensions:scanSystemExtensions" => {
			// Atom H1a: Arguments[0]=type, Arguments[1]=profileLocation URI,
			// Arguments[2]=productVersion, Arguments[3]=??? (VS Code canonical is
			// 3; shim appears to add a 4th). Dump to find out what it
			// contains on post-nav page reloads where the sidebar
			// renders 0 entries despite Mountain returning 94.
			let ArgsSummary = Arguments
				.iter()
				.enumerate()
				.map(|(Idx, V)| {
					let Preview = serde_json::to_string(V).unwrap_or_default();

					// Char-aware truncation - same UTF-8 hazard as
					// the diagnostic-tag formatter above.
					let Trimmed = if Preview.len() > 180 {
						let CutAt = Preview
							.char_indices()
							.map(|(Index, _)| Index)
							.take_while(|Index| *Index <= 180)
							.last()
							.unwrap_or(0);

						format!("{}…", &Preview[..CutAt])
					} else {
						Preview
					};

					format!("[{}]={}", Idx, Trimmed)
				})
				.collect::<Vec<_>>()
				.join(" ");

			dev_log!("extensions", "{} Arguments={}", command, ArgsSummary);

			// `scanSystemExtensions` is conceptually
			// `getInstalled(type=ExtensionType.System)`, so override
			// `Arguments[0]` to `0` before forwarding.
			let EffectiveArgs = if *command == *"extensions:scanSystemExtensions" {
				let mut Overridden = Arguments.clone();

				if Overridden.is_empty() {
					Overridden.push(Value::Null);
				}

				Overridden[0] = json!(0);

				Overridden
			} else {
				Arguments.clone()
			};

			Some(ExtensionsGetInstalled(RunTime.clone(), EffectiveArgs).await)
		},

		"extensions:scanUserExtensions" => {
			// User-scope scan. Forward to the unified handler with
			// `type=ExtensionType.User (1)` so VSIX-installed
			// extensions under `~/.fiddee/extensions/*` come back
			// even when the caller didn't pass an explicit type
			// filter.
			dev_log!("extensions", "{} (forwarded to getInstalled with type=User)", command);

			let mut UserArgs = Arguments.clone();

			if UserArgs.is_empty() {
				UserArgs.push(Value::Null);
			}

			UserArgs[0] = json!(1);

			Some(ExtensionsGetInstalled(RunTime.clone(), UserArgs).await)
		},

		"extensions:getUninstalled" => {
			// Uninstalled state (extensions soft-deleted but kept in
			// the profile) isn't tracked yet; an empty array is the
			// correct "nothing pending uninstall" response.
			dev_log!("extensions", "{} (returning [])", command);

			Some(Ok(Value::Array(Vec::new())))
		},

		// Gallery is offline: Mountain has no marketplace backend.
		"extensions:query" | "extensions:getExtensions" | "extensions:getRecommendations" => {
			dev_log!("extensions", "{} (offline gallery - returning [])", command);

			Some(Ok(Value::Array(Vec::new())))
		},

		// `ExtensionGalleryService.query()` - called when the user types
		// in the Extensions search box. Returns `IGalleryQueryResult`.
		"extensions:search" => {
			dev_log!("extensions", "extensions:search (offline gallery - returning empty)");

			Some(Ok(json!({ "galleryExtensions": [], "total": 0 })))
		},

		// `ExtensionGalleryService.getCoreTranslation()` - locale bundles.
		"extensions:getCoreTranslation" => Some(Ok(Value::Null)),

		// `ExtensionGalleryService.download()` - no gallery backend.
		"extensions:download" => Some(Err("Marketplace download unavailable in offline mode".to_string())),

		// `IExtensionsControlManifest` - consulted by the Extensions sidebar.
		"extensions:getExtensionsControlManifest" => {
			dev_log!("extensions", "{} (offline gallery - empty manifest)", command);

			Some(Ok(json!({
				"malicious": [],
				"deprecated": {},
				"search": [],
				"autoUpdate": {},
			})))
		},

		// Pin state is Wind-owned (Cocoon never sees it).
		"extensions:resetPinnedStateForAllUserExtensions" => {
			dev_log!("extensions", "{} (no-op, pin state is UI-local)", command);

			Some(Ok(Value::Null))
		},

		// Local VSIX install.
		"extensions:install" => {
			Some(Extension::ExtensionInstall::Fn(ApplicationHandle.clone(), RunTime.clone(), Arguments).await)
		},

		"extensions:uninstall" => {
			Some(Extension::ExtensionUninstall::Fn(ApplicationHandle.clone(), RunTime.clone(), Arguments).await)
		},

		// Reads `extension/package.json` from a `.vsix` archive.
		"extensions:getManifest" => {
			Some(crate::IPC::WindServiceHandlers::Extension::ExtensionGetManifest::Fn(Arguments).await)
		},

		// `extensions:reinstall` - no gallery, return minimal envelope.
		"extensions:reinstall" => {
			let ExtId = arg_string(&Arguments, 0);

			dev_log!("extensions", "extensions:reinstall {} (no-op: no gallery)", ExtId);

			Some(Ok(serde_json::json!({ "identifier": { "id": ExtId }, "version": "0.0.0", "type": 0 })))
		},

		// Metadata update only matters for ratings/icons/readme.
		"extensions:updateMetadata" => {
			dev_log!("extensions", "{} (no-op: no gallery backend)", command);

			Some(Ok(Value::Null))
		},

		_ => None,
	}
}
