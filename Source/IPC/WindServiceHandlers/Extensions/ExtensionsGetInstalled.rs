#![allow(non_snake_case)]

//! `extensions:getInstalled(type?)` - return scanned extensions
//! reshaped as VS Code's `ILocalExtension[]` so
//! `ExtensionManagementChannelClient.getInstalled` can
//! destructure `extension.identifier.id`,
//! `extension.manifest.*`, and `extension.location` without
//! blowing up.
//!
//! ## Argument contract
//!
//! `Arguments[0]` is the optional `ExtensionType` filter VS
//! Code passes:
//!
//! - `0` (System) → only built-ins.
//! - `1` (User) → only VSIX-installed.
//! - `null` / missing → every known extension.
//!
//! Without the filter the trusted-publishers boot migration
//! iterates User-typed extensions over System manifests and
//! crashes on `manifest.publisher.toLowerCase()`.
//!
//! ## Boot-time race
//!
//! The workbench fires `getInstalled` ~13 times within the
//! first second. `ExtensionPopulate` runs in parallel and only
//! writes to ScannedExtensions ~250-500 ms in. If we returned
//! `[]` early, the workbench cached it forever and the activity
//! bar lost every extension-contributed icon. We poll for ≤5 s
//! before returning empty.
//!
//! ## Manifest skeleton
//!
//! VS Code unconditionally calls
//! `manifest.publisher.toLowerCase()`. A `null` or non-object
//! manifest crashes the webview before its first paint. We
//! coerce to `{}` and inject `publisher`/`name`/`version`
//! defaults so the renderer always has shape.

use std::sync::Arc;

use CommonLibrary::ExtensionManagement::ExtensionManagementService::ExtensionManagementService;
use serde_json::{Value, json};

use crate::{
	IPC::UriComponents::Normalize::Fn as NormalizeUri,
	RunTime::ApplicationRunTime::ApplicationRunTime,
	dev_log,
};

const EXTENSION_TYPE_SYSTEM:u8 = 0;
const EXTENSION_TYPE_USER:u8 = 1;

pub async fn ExtensionsGetInstalled(RunTime:Arc<ApplicationRunTime>, Arguments:Vec<Value>) -> Result<Value, String> {
	let TypeFilter:Option<u8> = Arguments.first().and_then(|V| V.as_u64()).map(|N| N as u8);

	let mut Extensions = RunTime
		.Environment
		.GetExtensions()
		.await
		.map_err(|Error| format!("extensions:getInstalled failed: {}", Error))?;

	if Extensions.is_empty() {
		const POLL_INTERVAL_MS:u64 = 50;
		const MAX_WAIT_MS:u64 = 5000;
		let mut Elapsed:u64 = 0;
		while Extensions.is_empty() && Elapsed < MAX_WAIT_MS {
			tokio::time::sleep(std::time::Duration::from_millis(POLL_INTERVAL_MS)).await;
			Elapsed += POLL_INTERVAL_MS;
			Extensions = RunTime
				.Environment
				.GetExtensions()
				.await
				.map_err(|Error| format!("extensions:getInstalled failed: {}", Error))?;
		}
		if !Extensions.is_empty() {
			dev_log!(
				"extensions",
				"extensions:getInstalled awaited scan completion ({}ms) - now has {} entries",
				Elapsed,
				Extensions.len()
			);
		} else {
			dev_log!(
				"extensions",
				"warn: extensions:getInstalled timed out after {}ms; returning empty list",
				Elapsed
			);
		}
	}

	let Wrapped:Vec<Value> = Extensions
		.into_iter()
		.filter_map(|Manifest| {
			let IsBuiltin = Manifest.get("isBuiltin").and_then(Value::as_bool).unwrap_or(true);
			let ExtensionType = if IsBuiltin { EXTENSION_TYPE_SYSTEM } else { EXTENSION_TYPE_USER };

			if let Some(Wanted) = TypeFilter
				&& Wanted != ExtensionType
			{
				return None;
			}

			let Publisher = Manifest
				.get("publisher")
				.and_then(Value::as_str)
				.filter(|S| !S.is_empty())
				.unwrap_or("unknown")
				.to_string();
			let Name = Manifest
				.get("name")
				.and_then(Value::as_str)
				.filter(|S| !S.is_empty())
				.unwrap_or("unknown")
				.to_string();
			let Id = format!("{}.{}", Publisher, Name);

			let Location = NormalizeUri(Manifest.get("extensionLocation"));

			let mut Manifest = match Manifest {
				Value::Object(_) => Manifest,
				_ => json!({}),
			};
			if let Value::Object(ref mut Map) = Manifest {
				Map.insert("extensionLocation".to_string(), Location.clone());
				Map.entry("publisher".to_string()).or_insert_with(|| json!(Publisher.clone()));
				Map.entry("name".to_string()).or_insert_with(|| json!(Name.clone()));
				Map.entry("version".to_string()).or_insert_with(|| json!("0.0.0"));
			}

			Some(json!({
				"type": ExtensionType,
				"isBuiltin": IsBuiltin,
				"identifier": { "id": Id },
				"manifest": Manifest,
				"location": Location,
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
				"source": if IsBuiltin { "system" } else { "vsix" },
				"size": 0,
			}))
		})
		.collect();

	dev_log!(
		"extensions",
		"extensions:getInstalled type={:?} returning {} ILocalExtension-shaped entries",
		TypeFilter,
		Wrapped.len()
	);

	Ok(json!(Wrapped))
}
