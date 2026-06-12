//! Extension commands router - delegates all `extensions:*` IPC commands.

use std::sync::Arc;

use serde_json::{Value, json};

use super::*;
use crate::{
	IPC::WindServiceHandlers::{
		Extension::{ExtensionGetManifest, ExtensionInstall, ExtensionUninstall},
		Extensions::{
			ExtensionsGet::Fn as ExtensionsGet,
			ExtensionsGetAll::Fn as ExtensionsGetAll,
			ExtensionsGetInstalled::Fn as ExtensionsGetInstalled,
			ExtensionsIsActive::Fn as ExtensionsIsActive,
		},
		Utilities::JsonValueHelpers::arg_string,
	},
	RunTime::ApplicationRunTime::ApplicationRunTime,
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

			// `scanSystemExtensions` → `getInstalled(type=ExtensionType.System)`.
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
			dev_log!("extensions", "{} (forwarded to getInstalled with type=User)", command);

			let mut UserArgs = Arguments.clone();

			if UserArgs.is_empty() {
				UserArgs.push(Value::Null);
			}

			UserArgs[0] = json!(1);

			Some(ExtensionsGetInstalled(RunTime.clone(), UserArgs).await)
		},

		"extensions:getUninstalled" => {
			dev_log!("extensions", "{} (returning [])", command);

			Some(Ok(Value::Array(Vec::new())))
		},

		// Gallery is offline: Mountain has no marketplace backend.
		"extensions:query" | "extensions:getExtensions" | "extensions:getRecommendations" => {
			dev_log!("extensions", "{} (offline gallery - returning [])", command);

			Some(Ok(Value::Array(Vec::new())))
		},

		"extensions:search" => {
			dev_log!("extensions", "extensions:search (offline gallery - returning empty)");

			Some(Ok(json!({ "galleryExtensions": [], "total": 0 })))
		},

		"extensions:getCoreTranslation" => Some(Ok(Value::Null)),

		"extensions:download" => Some(Err("Marketplace download unavailable in offline mode".to_string())),

		"extensions:getExtensionsControlManifest" => {
			dev_log!("extensions", "{} (offline gallery - empty manifest)", command);

			Some(Ok(json!({
				"malicious": [],
				"deprecated": {},
				"search": [],
				"autoUpdate": {},
			})))
		},

		"extensions:resetPinnedStateForAllUserExtensions" => {
			dev_log!("extensions", "{} (no-op, pin state is UI-local)", command);

			Some(Ok(Value::Null))
		},

		// Local VSIX install.
		"extensions:install" => Some(ExtensionInstall::Fn(ApplicationHandle.clone(), RunTime.clone(), Arguments).await),

		"extensions:uninstall" => {
			Some(ExtensionUninstall::Fn(ApplicationHandle.clone(), RunTime.clone(), Arguments).await)
		},

		// Reads `extension/package.json` from a `.vsix` archive.
		"extensions:getManifest" => Some(ExtensionGetManifest::Fn(Arguments).await),

		// `extensions:reinstall` - no gallery, return minimal envelope.
		"extensions:reinstall" => {
			let ExtId = arg_string(&Arguments, 0);

			dev_log!("extensions", "extensions:reinstall {} (no-op: no gallery)", ExtId);

			Some(Ok(
				serde_json::json!({ "identifier": { "id": ExtId }, "version": "0.0.0", "type": 0 }),
			))
		},

		// Metadata update only matters for ratings/icons/readme.
		"extensions:updateMetadata" => {
			dev_log!("extensions", "{} (no-op: no gallery backend)", command);

			Some(Ok(Value::Null))
		},

		_ => None,
	}
}
