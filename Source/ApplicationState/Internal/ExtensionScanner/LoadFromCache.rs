//! # Extension Manifest Cache Loader (B7.P08)
//!
//! Loads the pre-baked extension manifest from
//! `Target/debug/extensions.manifest.json` (written by
//! `Maintain/Build/Manifest/PreBake.ts` as part of the debug build step).
//!
//! ## Why this exists
//!
//! Mountain's `ScanAndPopulateExtensions` currently reads 113+ `package.json`
//! files sequentially from disk during boot, taking ~1200 ms on cold storage.
//! After the build step runs `PreBake.ts`, the manifests are pre-merged into a
//! single JSON blob. `LoadFromCache` reads that blob with a single `fs::read`
//! and deserializes with `serde_json::from_slice`, reducing boot cost to <50
//! ms.
//!
//! ## Fallback
//!
//! If the cache file is missing, stale (older than 10 min), or corrupt, the
//! caller falls back to the normal `ScanAndPopulateExtensions` path.
//!
//! ## Cache format (written by PreBake.ts)
//!
//! ```json
//! {
//!   "version": 1,
//!   "count": 113,
//!   "extensions": [
//!     { "id": "publisher.name", "path": "/abs/path/to/ext", "manifest": { … } }
//!   ]
//! }
//! ```

use std::{collections::HashMap, path::PathBuf, time::Duration};

use CommonLibrary::Error::CommonError::CommonError;
use serde::Deserialize;
use serde_json::Value;

use crate::{ApplicationState::DTO::ExtensionDescriptionStateDTO::ExtensionDescriptionStateDTO, dev_log};

/// One entry in the pre-baked cache file.
#[derive(Debug, Deserialize)]
struct CachedEntry {
	id:String,
	path:String,
	manifest:Value,
}

/// Top-level cache blob.
#[derive(Debug, Deserialize)]
struct CacheBlob {
	version:u32,
	#[allow(dead_code)]
	count:u32,
	extensions:Vec<CachedEntry>,
}

/// Maximum cache age before we consider it stale and fall back to a live scan.
const MAX_CACHE_AGE:Duration = Duration::from_secs(600); // 10 minutes

/// Try to load extension descriptors from the pre-baked manifest cache.
///
/// Returns `Ok(Some(map))` on a cache hit, `Ok(None)` when the cache is
/// missing/stale/incompatible, and `Err(_)` only on unexpected I/O errors.
pub async fn TryLoadFromCache(
	BinaryDir:&PathBuf,
) -> Result<Option<HashMap<String, ExtensionDescriptionStateDTO>>, CommonError> {
	let CachePath = BinaryDir.join("extensions.manifest.json");

	// --- Existence + freshness check ---
	let Metadata = match tokio::fs::metadata(&CachePath).await {
		Ok(M) => M,
		Err(_) => {
			dev_log!("extensions", "[ExtensionCache] Cache not found at {}", CachePath.display());
			return Ok(None);
		},
	};

	let Age = Metadata.modified().ok().and_then(|T| T.elapsed().ok()).unwrap_or(Duration::MAX);

	if Age > MAX_CACHE_AGE {
		dev_log!(
			"extensions",
			"[ExtensionCache] Cache is stale ({:.0}s > {:.0}s), falling back to live scan",
			Age.as_secs_f32(),
			MAX_CACHE_AGE.as_secs_f32()
		);
		return Ok(None);
	}

	// --- Read + parse ---
	let Bytes = match tokio::fs::read(&CachePath).await {
		Ok(B) => B,
		Err(E) => {
			dev_log!(
				"extensions",
				"warn: [ExtensionCache] Read failed: {}; falling back to live scan",
				E
			);
			return Ok(None);
		},
	};

	let Blob:CacheBlob = match serde_json::from_slice(&Bytes) {
		Ok(B) => B,
		Err(E) => {
			dev_log!(
				"extensions",
				"warn: [ExtensionCache] Parse error: {}; falling back to live scan",
				E
			);
			return Ok(None);
		},
	};

	if Blob.version != 1 {
		dev_log!(
			"extensions",
			"[ExtensionCache] Unsupported cache version {}; falling back to live scan",
			Blob.version
		);
		return Ok(None);
	}

	// --- Hydrate into ExtensionDescriptionStateDTO ---
	let mut Map:HashMap<String, ExtensionDescriptionStateDTO> = HashMap::with_capacity(Blob.extensions.len());

	for Entry in Blob.extensions {
		let Manifest = &Entry.manifest;
		let Path = &Entry.path;

		let Name = Manifest.get("name").and_then(Value::as_str).unwrap_or("").to_string();
		let Version = Manifest.get("version").and_then(Value::as_str).unwrap_or("0.0.0").to_string();
		let Publisher = Manifest
			.get("publisher")
			.and_then(Value::as_str)
			.unwrap_or(Entry.id.split('.').next().unwrap_or("unknown"))
			.to_string();

		// Build ExtensionLocation as a plain file:// URI string.
		// Normalize.rs handles `Value::String` via `FromUrl::Fn` which parses
		// it into the `{scheme, authority, path, ...}` UriComponents shape.
		// Using `{"value": url}` (the Identifier wrapper shape) would NOT match
		// the `scheme` key check and would fall through to `/extensions/unknown`.
		let LocationUri = format!("file://{}", Path);

		// Activation events
		let ActivationEvents:Option<Vec<String>> = Manifest
			.get("activationEvents")
			.and_then(|V| serde_json::from_value(V.clone()).ok());

		// Contributes
		let Contributes = Manifest.get("contributes").cloned();

		// Categories
		let Categories:Option<Vec<String>> =
			Manifest.get("categories").and_then(|V| serde_json::from_value(V.clone()).ok());

		let Main = Manifest.get("main").and_then(Value::as_str).map(str::to_string);
		let Browser = Manifest.get("browser").and_then(Value::as_str).map(str::to_string);
		let ModuleType = Manifest.get("type").and_then(Value::as_str).map(str::to_string);

		let DisplayName = Manifest.get("displayName").and_then(Value::as_str).map(str::to_string);

		let ExtId = Entry.id.clone();

		// Determine if this is a built-in extension (lives under the
		// binary's sibling `extensions/` directory).
		let IsBuiltin = PathBuf::from(Path)
			.parent()
			.and_then(|P| P.file_name())
			.and_then(|N| N.to_str())
			.map(|N| N == "extensions")
			.unwrap_or(false);

		let Description = Manifest.get("description").and_then(Value::as_str).map(str::to_string);
		let Keywords:Option<Vec<String>> =
			Manifest.get("keywords").and_then(|V| serde_json::from_value(V.clone()).ok());
		let Icon = Manifest.get("icon").and_then(Value::as_str).map(str::to_string);
		let AiKey = Manifest.get("aiKey").and_then(Value::as_str).map(str::to_string);
		let ExtensionKind = Manifest.get("extensionKind").cloned();
		let Capabilities = Manifest.get("capabilities").cloned();
		let ExtensionDependencies:Option<Vec<String>> = Manifest
			.get("extensionDependencies")
			.and_then(|V| serde_json::from_value(V.clone()).ok());
		let ExtensionPack:Option<Vec<String>> = Manifest
			.get("extensionPack")
			.and_then(|V| serde_json::from_value(V.clone()).ok());

		let Dto = ExtensionDescriptionStateDTO {
			Identifier:serde_json::json!({ "value": ExtId }),
			Name,
			Version,
			Publisher,
			Engines:Manifest.get("engines").cloned().unwrap_or(serde_json::json!({})),
			Main,
			Browser,
			ModuleType,
			IsBuiltin,
			IsUnderDevelopment:false,
			ExtensionLocation:Value::String(LocationUri),
			ActivationEvents,
			Contributes,
			Categories,
			DisplayName,
			Description,
			Keywords,
			Repository:Manifest.get("repository").cloned(),
			Bugs:Manifest.get("bugs").cloned(),
			Homepage:Manifest.get("homepage").and_then(Value::as_str).map(str::to_string),
			License:Manifest.get("license").and_then(Value::as_str).map(str::to_string),
			Icon,
			AiKey,
			ExtensionKind,
			Capabilities,
			ExtensionDependencies,
			ExtensionPack,
		};

		Map.insert(ExtId, Dto);
	}

	dev_log!(
		"extensions",
		"[ExtensionCache] Loaded {} extensions from cache ({} bytes, {:.0}s old)",
		Map.len(),
		Bytes.len(),
		Age.as_secs_f32()
	);

	Ok(Some(Map))
}
