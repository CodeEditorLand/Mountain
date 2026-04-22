#![allow(non_snake_case, unused_variables, dead_code, unused_imports)]

//! Extension management handlers - list, get, query, install/uninstall stubs.

use std::sync::Arc;

use CommonLibrary::ExtensionManagement::ExtensionManagementService::ExtensionManagementService;
use serde_json::{Value, json};

use crate::{dev_log, RunTime::ApplicationRunTime::ApplicationRunTime};

/// `MarshalledId.Uri` from VS Code's `src/vs/base/common/marshallingIds.ts`.
/// VS Code's `transformAndReviveIncomingURIs` walks every property of every
/// object returned from the IPC channel and only calls `URI.revive()` on
/// nested objects that carry `$mid === 1`. Without this marker the extension
/// location surfaces as a plain `UriComponents` bag; downstream
/// `resources.joinPath(local.location, …)` then trips on
/// `uri.with is not a function` the moment the sidebar tries to build an
/// icon URL. Every UriComponents we produce has to carry `$mid: 1`.
const MID_URI:u64 = 1;

/// Reshape an `extensionLocation` field into VS Code's `UriComponents` object
/// form. The extension scanner stores it as a raw `file://` URL string; the
/// renderer's URI reviver is keyed off the `$mid` marshalling marker and only
/// revives objects that carry it, so the emitted shape must always include
/// `$mid: 1` alongside the usual `scheme / authority / path / query /
/// fragment` fields - otherwise the sidebar silently filters the whole batch
/// out (no `$mid` ⇒ no revive ⇒ `.fsPath` / `.with` undefined).
fn Normalize_Location_To_UriComponents(Raw:Option<&Value>) -> Value {
	let Base = match Raw {
		Some(Value::Object(Map)) if Map.contains_key("scheme") => Value::Object(Map.clone()),
		Some(Value::String(Url)) => Parse_File_Url_To_UriComponents(Url),
		_ => json!({
			"scheme": "file",
			"authority": "",
			"path": "/extensions/unknown",
			"query": "",
			"fragment": "",
		}),
	};
	Stamp_Mid_Uri(Base)
}

/// Attach `$mid: 1` to a `UriComponents` object if it isn't already present.
/// Guards against both missing markers (the common case; Mountain built the
/// object ourselves) and pre-marked payloads (passthrough from an upstream
/// service that already tagged it). Returns the input unchanged for any
/// non-object value - callers are expected to have already produced a shape.
fn Stamp_Mid_Uri(Input:Value) -> Value {
	match Input {
		Value::Object(mut Map) => {
			Map.entry("$mid".to_string()).or_insert(json!(MID_URI));
			Value::Object(Map)
		}
		Other => Other,
	}
}

/// Minimal `file://` URL → `UriComponents` parser. Accepts
/// `file:///absolute/path` (authority empty) and preserves the path verbatim.
/// Non-`file:` schemes are parsed generically (scheme: path).
fn Parse_File_Url_To_UriComponents(Url:&str) -> Value {
	if let Some(Rest) = Url.strip_prefix("file://") {
		// `file:///Volumes/...` → authority="", path="/Volumes/..."
		let (Authority, Path) = match Rest.find('/') {
			Some(0) => ("", Rest),
			Some(Index) => (&Rest[..Index], &Rest[Index..]),
			None => ("", ""),
		};
		return json!({
			"scheme": "file",
			"authority": Authority,
			"path": Path,
			"query": "",
			"fragment": "",
		});
	}
	if let Some((Scheme, PathPart)) = Url.split_once(':') {
		let Trimmed = PathPart.trim_start_matches("//");
		let (Authority, Path) = match Trimmed.find('/') {
			Some(0) => ("", Trimmed),
			Some(Index) => (&Trimmed[..Index], &Trimmed[Index..]),
			None => ("", Trimmed),
		};
		return json!({
			"scheme": Scheme,
			"authority": Authority,
			"path": Path,
			"query": "",
			"fragment": "",
		});
	}
	json!({
		"scheme": "file",
		"authority": "",
		"path": Url,
		"query": "",
		"fragment": "",
	})
}

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
			let Location = Normalize_Location_To_UriComponents(Manifest.get("extensionLocation"));
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
