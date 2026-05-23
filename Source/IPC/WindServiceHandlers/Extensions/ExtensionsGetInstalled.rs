//! `extensions:getInstalled(type?)` - return scanned extensions reshaped as
//! VS Code's `ILocalExtension[]` so `ExtensionManagementChannelClient
//! .getInstalled` can destructure `extension.identifier.id`,
//! `extension.manifest.*`, and `extension.location` without blowing up.
//!
//! ## Argument contract
//!
//! `Arguments[0]` is the optional `ExtensionType` filter VS Code passes:
//! - `0` (System) → only built-ins.
//! - `1` (User) → only VSIX-installed.
//! - `null` / missing → every known extension.
//!
//! Without the filter the trusted-publishers boot migration iterates
//! User-typed extensions over System manifests and crashes on
//! `manifest.publisher.toLowerCase()`.
//!
//! ## Boot-time race
//!
//! The workbench fires `getInstalled` ~13 times within the first second.
//! `ExtensionPopulate` runs in parallel and writes to `ScannedExtensions`
//! 250-500 ms in. We await `ExtensionState.ScanReady` (a `tokio::sync::Notify`
//! fired once the scan commits its results) with a 5 s hard cap, then return
//! whatever is available. No 50 ms polling loop - we wake exactly when data
//! arrives.
//!
//! ## Manifest skeleton
//!
//! VS Code unconditionally calls `manifest.publisher.toLowerCase()`. A `null`
//! or non-object manifest crashes the webview before its first paint. We
//! coerce to `{}` and inject `publisher`/`name`/`version` defaults.

use std::{
	sync::{Arc, OnceLock},
	time::Duration,
};

use CommonLibrary::ExtensionManagement::ExtensionManagementService::ExtensionManagementService;
use serde_json::{Value, json};

use crate::{
	IPC::UriComponents::Normalize::Fn as NormalizeUri,
	RunTime::ApplicationRunTime::ApplicationRunTime,
	dev_log,
};

const EXTENSION_TYPE_SYSTEM:u8 = 0;

const EXTENSION_TYPE_USER:u8 = 1;

const SCAN_WAIT_CAP_MS:u64 = 5000;

// Per-type cached responses. Extensions don't change during a session so
// building the ILocalExtension[] once per type and returning the cached Value
// on subsequent calls avoids re-serializing ~1.8 MB on every getInstalled call.
// Keyed by TypeFilter: index 0=None(all), 1=System(0), 2=User(1).
static INSTALLED_CACHE:[OnceLock<Value>; 3] = [OnceLock::new(), OnceLock::new(), OnceLock::new()];

fn CacheIndex(TypeFilter:Option<u8>) -> usize {
	match TypeFilter {
		None => 0,

		Some(EXTENSION_TYPE_SYSTEM) => 1,

		Some(EXTENSION_TYPE_USER) => 2,

		Some(_) => 0,
	}
}

pub async fn Fn(RunTime:Arc<ApplicationRunTime>, Arguments:Vec<Value>) -> Result<Value, String> {
	let TypeFilter:Option<u8> = Arguments.first().and_then(|V| V.as_u64()).map(|N| N as u8);

	// Fast path: return cached response if available (built on first call per
	// type).
	let CacheSlot = CacheIndex(TypeFilter);

	if let Some(Cached) = INSTALLED_CACHE[CacheSlot].get() {
		let Count = Cached.as_array().map(|A| A.len()).unwrap_or(0);

		dev_log!(
			"extensions",
			"extensions:getInstalled type={:?} returning {} entries (cache hit)",
			TypeFilter,
			Count
		);

		return Ok(Cached.clone());
	}

	let mut Extensions = RunTime
		.Environment
		.GetExtensions()
		.await
		.map_err(|Error| format!("extensions:getInstalled failed: {}", Error))?;

	if Extensions.is_empty() {
		let ScanReady = RunTime.Environment.ApplicationState.Extension.ScanReady.clone();

		let Notified = tokio::time::timeout(Duration::from_millis(SCAN_WAIT_CAP_MS), ScanReady.notified()).await;

		Extensions = RunTime
			.Environment
			.GetExtensions()
			.await
			.map_err(|Error| format!("extensions:getInstalled failed: {}", Error))?;

		match Notified {
			Ok(()) => {
				dev_log!(
					"extensions",
					"extensions:getInstalled: scan-ready signal received, {} entries available",
					Extensions.len()
				);
			},

			Err(_) => {
				dev_log!(
					"extensions",
					"warn: extensions:getInstalled: scan-ready timed out after {}ms; {} entries available",
					SCAN_WAIT_CAP_MS,
					Extensions.len()
				);
			},
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

	let Response = json!(Wrapped);

	// Only cache non-empty responses - an empty response on first call (timeout)
	// shouldn't poison the cache for subsequent calls that would get real data.
	if !Wrapped.is_empty() {
		let _ = INSTALLED_CACHE[CacheSlot].set(Response.clone());
	}

	Ok(Response)
}
