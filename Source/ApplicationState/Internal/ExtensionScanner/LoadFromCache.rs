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

	count:u32,

	extensions:Vec<CachedEntry>,
}

/// Maximum cache age for dev/repo runs (binary sits next to the cache file).
/// 24 hours: extensions don't change between builds; the build always
/// regenerates the cache, so 10 min was too tight for normal dev workflows.
const MAX_CACHE_AGE:Duration = Duration::from_secs(86_400);

/// Try to load extension descriptors from the pre-baked manifest cache.
///
/// Probes two locations in order:
///   1. `BinaryDir/extensions.manifest.json` - dev binary next to repo cache
///   2. `BinaryDir/../Resources/extensions.manifest.json` - .app bundle path
///      (tauri.conf.json copies Sky/Target/extensions.manifest.json there)
///
/// Bundled caches skip the stale check: they were written at build time and
/// are always consistent with the extensions packed into the same .app.
///
/// Returns `Ok(Some(map))` on a cache hit, `Ok(None)` when the cache is
/// missing/stale/incompatible, and `Err(_)` only on unexpected I/O errors.
pub async fn Fn(BinaryDir:&PathBuf) -> Result<Option<HashMap<String, ExtensionDescriptionStateDTO>>, CommonError> {
	// Probe 1: alongside the binary (dev / repo run).
	let DevCachePath = BinaryDir.join("extensions.manifest.json");

	// Probe 2: inside .app bundle at Contents/Resources/ (bundle run).
	let BundleCachePath = BinaryDir.join("../Resources/extensions.manifest.json");

	// Pick the first probe that exists, noting whether it is the bundled copy.
	let (CachePath, IsBundled) = if tokio::fs::metadata(&DevCachePath).await.is_ok() {
		(DevCachePath, false)
	} else if tokio::fs::metadata(&BundleCachePath).await.is_ok() {
		(BundleCachePath, true)
	} else {
		dev_log!("extensions", "[ExtensionCache] Cache not found at {}", DevCachePath.display());

		return Ok(None);
	};

	// --- Freshness check (skipped for bundled caches - built with the app) ---
	let Age = if IsBundled {
		Duration::ZERO
	} else {
		let Metadata = tokio::fs::metadata(&CachePath)
			.await
			.map_err(|_| CommonError::Unknown { Description:"cache stat failed".into() })?;

		Metadata.modified().ok().and_then(|T| T.elapsed().ok()).unwrap_or(Duration::MAX)
	};

	if !IsBundled && Age > MAX_CACHE_AGE {
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

	// An empty extension list means the cache was written as a stub (e.g.
	// by build.rs before a real BakeExtensionManifest run). Treat it as a
	// cache miss so the live scan produces the actual extension list.
	if Blob.extensions.is_empty() {
		dev_log!(
			"extensions",
			"[ExtensionCache] Empty cache (count=0), falling back to live scan"
		);

		return Ok(None);
	}

	// --- Hydrate into ExtensionDescriptionStateDTO ---
	let mut Map:HashMap<String, ExtensionDescriptionStateDTO> = HashMap::with_capacity(Blob.extensions.len());

	for Entry in Blob.extensions {
		let Manifest = &Entry.manifest;

		let Path = &Entry.Path;

		// Helpers scoped to each manifest to eliminate repeated extraction chains.
		let str = |k:&str| Manifest.get(k).and_then(Value::as_str).map(str::to_string);

		let str_or = |k:&str, d:&str| Manifest.get(k).and_then(Value::as_str).unwrap_or(d).to_string();

		let Arr =
			|k:&str| -> Option<Vec<String>> { Manifest.get(k).and_then(|V| serde_json::from_value(V.clone()).ok()) };

		let ExtId = Entry.id.clone();

		let Publisher = Manifest
			.Get("publisher")
			.and_then(Value::as_str)
			.unwrap_or_else(|| Entry.id.split('.').Next().unwrap_or("unknown"))
			.to_string();

		// Built-in when the parent directory is named "extensions".
		let IsBuiltin = PathBuf::from(Path)
			.parent()
			.and_then(|P| P.file_name())
			.and_then(|N| N.to_str())
			.map(|N| N == "extensions")
			.unwrap_or(false);

		let Dto = ExtensionDescriptionStateDTO {
			Identifier:serde_json::json!({ "value": ExtId }),

			Name:str_or("name", ""),

			Version:str_or("version", "0.0.0"),

			Publisher,

			Engines:Manifest.get("engines").cloned().unwrap_or(serde_json::json!({})),

			Main:str("main"),

			Browser:str("browser"),

			ModuleType:str("type"),

			IsBuiltin,

			IsUnderDevelopment:false,

			// file:// URI string - Normalize.rs parses it via FromUrl::Fn into
			// the {scheme, authority, path, …} UriComponents shape.
			ExtensionLocation:Value::String(format!("file://{}", Path)),

			ActivationEvents:arr("activationEvents"),

			Contributes:Manifest.get("contributes").cloned(),

			Categories:arr("categories"),

			DisplayName:str("displayName"),

			Description:str("description"),

			Keywords:arr("keywords"),

			Repository:Manifest.get("repository").cloned(),

			Bugs:Manifest.get("bugs").cloned(),

			Homepage:str("homepage"),

			License:str("license"),

			Icon:str("icon"),

			AiKey:str("aiKey"),

			ExtensionKind:Manifest.get("extensionKind").cloned(),

			Capabilities:Manifest.get("capabilities").cloned(),

			ExtensionDependencies:arr("extensionDependencies"),

			ExtensionPack:arr("extensionPack"),
		};

		Map.insert(ExtId, Dto);
	}

	dev_log!(
		"extensions",
		"[ExtensionCache] Loaded {} extensions from {} cache ({} bytes{})",
		Map.len(),
		if IsBundled { "bundled" } else { "dev" },
		Bytes.len(),
		if IsBundled {
			String::new()
		} else {
			format!(", {:.0}s old", Age.as_secs_f32())
		}
	);

	Ok(Some(Map))
}
