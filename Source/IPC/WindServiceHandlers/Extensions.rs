#![allow(non_snake_case, unused_variables, dead_code, unused_imports)]

//! Extension management handlers - list, get, query, install/uninstall stubs.

use std::sync::Arc;

use CommonLibrary::ExtensionManagement::ExtensionManagementService::ExtensionManagementService;
use serde_json::{Value, json};

use crate::{
	IPC::UriComponents::Normalize::Fn as NormalizeUri,
	RunTime::ApplicationRunTime::ApplicationRunTime,
	dev_log,
};

/// VS Code's `ExtensionType` enum - mirror the numeric values used by the
/// renderer's `getInstalled(type?)` IPC so the filter in `GetInstalledArgs`
/// matches what the channel client sends.
///
/// `src/vs/platform/extensions/common/extensions.ts` in the pinned VS Code
/// dependency:
/// ```ts
/// export const enum ExtensionType { System = 0, User = 1 }
/// ```
const EXTENSION_TYPE_SYSTEM:u8 = 0;
const EXTENSION_TYPE_USER:u8 = 1;

/// Return scanned extensions reshaped as VS Code's `ILocalExtension[]`
/// so `ExtensionManagementChannelClient.getInstalled` can destructure
/// `extension.identifier.id`, `extension.manifest.*`, and
/// `extension.location` without blowing up.
///
/// # Argument contract
///
/// `Arguments[0]` is the optional `ExtensionType` filter VS Code passes in:
/// - `0` (System) → only return built-in extensions.
/// - `1` (User) → only return VSIX-installed extensions.
/// - `null` / missing → return every known extension.
///
/// Previously this filter was silently dropped and every call returned the
/// full list hardcoded as `type: 0, isBuiltin: true`. That produced three
/// cascading symptoms:
///   1. VSIX-installed extensions (e.g. `Anthropic.claude-code`) showed up
///      under "Built-in" in the Extensions sidebar and had no Uninstall action
///      because the UI keys off `type === User`.
///   2. The trusted-publishers boot migration iterated every extension as User
///      and attempted `manifest.publisher.toLowerCase()` against System
///      manifests.
///   3. `extensions:scanUserExtensions` (which shares the user-only semantic)
///      returned zero, making the "Install from VSIX…" refresh appear to do
///      nothing even when the install itself succeeded.
pub async fn ExtensionsGetInstalled(RunTime:Arc<ApplicationRunTime>, Arguments:Vec<Value>) -> Result<Value, String> {
	let TypeFilter:Option<u8> = Arguments.first().and_then(|V| V.as_u64()).map(|N| N as u8);

	// Boot-time race fix: the workbench's `IExtensionService` calls
	// `extensions:getInstalled` ~13 times during the first second of
	// boot - reading an empty list because `ExtensionPopulate` runs
	// in a parallel async task and only writes to ScannedExtensions
	// AFTER its multi-path scan completes (~250-500ms in). The
	// workbench caches that empty list, runs `viewsContainersExtension
	// Point.setHandler([])`, and never re-processes contributions
	// when the scan finishes - so the activity bar has zero
	// extension-contributed icons (no Roo, Claude, gitlens panels)
	// even though 113 extensions are scanned.
	//
	// Poll the map up to ~5 seconds before returning empty. 3s was
	// previously the ceiling but cold-cache runs over 113 extensions
	// across 6 paths regularly land at ~2950ms - the previous limit
	// hit the wall and returned `[]`, poisoning the workbench's
	// `IExtensionService` cache. 5s gives slow I/O (cold NVM,
	// network-mounted user dirs) headroom while keeping the visible
	// worst case bounded.
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
			// `isBuiltin` is authored by the scanner; default to `true` for
			// safety when the field is missing (matches the pre-filter
			// hardcoded behaviour so we never drop an extension the renderer
			// used to see).
			let IsBuiltin = Manifest.get("isBuiltin").and_then(Value::as_bool).unwrap_or(true);
			let ExtensionType = if IsBuiltin { EXTENSION_TYPE_SYSTEM } else { EXTENSION_TYPE_USER };

			if let Some(Wanted) = TypeFilter {
				if Wanted != ExtensionType {
					return None;
				}
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

			// VS Code's `URI.revive()` is a no-op on strings, so the scanner's
			// `file://…` raw URL has to be reshaped into `UriComponents` here -
			// otherwise every `location.fsPath` / `location.scheme` access in
			// the sidebar silently returns `undefined` and the whole batch is
			// filtered out. Normalize once, reuse for both the top-level
			// `location` and the mirror inside `manifest.extensionLocation` so
			// callers that read either field get the same shape.
			let Location = NormalizeUri(Manifest.get("extensionLocation"));
			// Guarantee the manifest is an object with non-empty `publisher`,
			// `name` and `version` fields before it reaches the renderer. VS
			// Code runs a trusted-publishers migration at first-boot
			// (`extensions.contribution.ts`) that unconditionally calls
			// `extension.manifest.publisher.toLowerCase()`; any missing
			// `manifest` object, or a manifest with `publisher === undefined`,
			// crashes the webview with
			// `TypeError: undefined is not an object (evaluating 'manifest.publisher')`
			// before the workbench can render a single pixel. A non-object
			// value here (null / Value::Null from upstream scan failures) is
			// replaced with a bare skeleton so the renderer always has shape.
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
				// IExtension (base)
				"type": ExtensionType,
				"isBuiltin": IsBuiltin,
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
				// `source` distinguishes the disk origin: built-ins ship with
				// the bundle ("system"); VSIX-installed extensions live under
				// `~/.land/extensions/*` ("vsix"). The sidebar keys off this
				// for the "Uninstall" gesture.
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

/// Return metadata for all scanned extensions.
pub async fn ExtensionsGetAll(RunTime:Arc<ApplicationRunTime>) -> Result<Value, String> {
	let Extensions = RunTime
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
pub async fn ExtensionsGet(RunTime:Arc<ApplicationRunTime>, Arguments:Vec<Value>) -> Result<Value, String> {
	let Id = Arguments
		.first()
		.and_then(|V| V.as_str())
		.ok_or_else(|| "extensions:get requires string id as first argument".to_string())?
		.to_string();

	let Extension = RunTime
		.Environment
		.GetExtension(Id)
		.await
		.map_err(|Error| format!("extensions:get failed: {}", Error))?;

	Ok(Extension.unwrap_or(Value::Null))
}

/// Check whether an extension is currently active (scanned and present).
pub async fn ExtensionsIsActive(RunTime:Arc<ApplicationRunTime>, Arguments:Vec<Value>) -> Result<Value, String> {
	let Id = Arguments
		.first()
		.and_then(|V| V.as_str())
		.ok_or_else(|| "extensions:isActive requires string id as first argument".to_string())?
		.to_string();

	let Extension = RunTime
		.Environment
		.GetExtension(Id)
		.await
		.map_err(|Error| format!("extensions:isActive failed: {}", Error))?;

	Ok(json!(Extension.is_some()))
}
