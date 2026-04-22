#![allow(non_snake_case, unused_variables, dead_code, unused_imports)]

//! Extension management handlers - list, get, query, install/uninstall stubs.

use std::sync::Arc;

use CommonLibrary::ExtensionManagement::ExtensionManagementService::ExtensionManagementService;
use serde_json::{Value, json};

use crate::{
	IPC::UriComponents::{Normalize as NormalizeUri, StampMidUri},
	RunTime::ApplicationRunTime::ApplicationRunTime,
	dev_log,
};

/// Return scanned extensions reshaped as VS Code's `ILocalExtension[]`
/// so `ExtensionManagementChannelClient.getInstalled` can destructure
/// `extension.identifier.id`, `extension.manifest.*`, and
/// `extension.location` without blowing up.
pub async fn handle_extensions_get_installed(runtime:Arc<ApplicationRunTime>) -> Result<Value, String> {
	let Extensions = runtime
		.Environment
		.GetExtensions()
		.await
		.map_err(|Error| format!("extensions:getInstalled failed: {}", Error))?;

	let Wrapped:Vec<Value> = Extensions
		.into_iter()
		.map(|Manifest| {
			let Publisher = Manifest
				.get("publisher")
				.and_then(Value::as_str)
				.unwrap_or("unknown")
				.to_string();
			let Name = Manifest.get("name").and_then(Value::as_str).unwrap_or("unknown").to_string();
			let Id = format!("{}.{}", Publisher, Name);

			// VS Code's `URI.revive()` is a no-op on strings, so the scanner's
			// `file://…` raw URL has to be reshaped into `UriComponents` here -
			// otherwise every `location.fsPath` / `location.scheme` access in
			// the sidebar silently returns `undefined` and the whole batch is
			// filtered out. Normalize once, reuse for both the top-level
			// `location` and the mirror inside `manifest.extensionLocation` so
			// callers that read either field get the same shape.
			let Location = NormalizeUri(Manifest.get("extensionLocation"));
			let mut Manifest = Manifest;
			if let Value::Object(ref mut Map) = Manifest {
				Map.insert("extensionLocation".to_string(), Location.clone());
			}

			json!({
				// IExtension (base)
				"type": 0, // ExtensionType.System
				"isBuiltin": true,
				"identifier": { "id": Id },
				"manifest": Manifest,
				"location": Location,
				"targetPlatform": "undefined",
				"isValid": true,
				"validations": [],
				"preRelease": false,
				// ILocalExtension (extras)
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
				"source": "system",
				"size": 0,
			})
		})
		.collect();

	dev_log!(
		"extensions",
		"extensions:getInstalled returning {} ILocalExtension-shaped entries",
		Wrapped.len()
	);

	Ok(json!(Wrapped))
}

/// Return metadata for all scanned extensions.
pub async fn handle_extensions_get_all(runtime:Arc<ApplicationRunTime>) -> Result<Value, String> {
	let Extensions = runtime
		.Environment
		.GetExtensions()
		.await
		.map_err(|Error| format!("extensions:getAll failed: {}", Error))?;

	dev_log!("extensions", "extensions:getAll returning {} extensions", Extensions.len());
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
	}
	Ok(json!(Extensions))
}

/// Return metadata for a single extension by ID.
pub async fn handle_extensions_get(runtime:Arc<ApplicationRunTime>, args:Vec<Value>) -> Result<Value, String> {
	let Id = args
		.first()
		.and_then(|V| V.as_str())
		.ok_or_else(|| "extensions:get requires string id as first argument".to_string())?
		.to_string();

	let Extension = runtime
		.Environment
		.GetExtension(Id)
		.await
		.map_err(|Error| format!("extensions:get failed: {}", Error))?;

	Ok(Extension.unwrap_or(Value::Null))
}

/// Check whether an extension is currently active (scanned and present).
pub async fn handle_extensions_is_active(
	runtime:Arc<ApplicationRunTime>,
	args:Vec<Value>,
) -> Result<Value, String> {
	let Id = args
		.first()
		.and_then(|V| V.as_str())
		.ok_or_else(|| "extensions:isActive requires string id as first argument".to_string())?
		.to_string();

	let Extension = runtime
		.Environment
		.GetExtension(Id)
		.await
		.map_err(|Error| format!("extensions:isActive failed: {}", Error))?;

	Ok(json!(Extension.is_some()))
}
