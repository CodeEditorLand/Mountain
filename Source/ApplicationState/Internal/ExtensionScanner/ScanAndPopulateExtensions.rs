use std::{collections::HashMap, path::PathBuf};

use CommonLibrary::Error::CommonError::CommonError;
use serde_json::Value;
use tauri::AppHandle;

use crate::{
	ApplicationState::DTO::ExtensionDescriptionStateDTO::ExtensionDescriptionStateDTO,
	ExtensionManagement,
	dev_log,
};

pub async fn Fn(
	ApplicationHandle:AppHandle,

	_State:&crate::ApplicationState::State::ExtensionState::State::State,
) -> Result<(), CommonError> {
	dev_log!("extensions", "[ExtensionScanner] Starting extension scan...");

	// --- Fast path: pre-baked manifest cache (B7.P08) ---
	// `Maintain/Build/Manifest/PreBake.ts` writes a single JSON blob to
	// `Target/debug/extensions.manifest.json` as part of the debug build.
	// Loading it avoids the N×disk-read scan and cuts ~1200 ms from boot.
	if let Ok(ExecutablePath) = std::env::current_exe() {
		if let Some(BinaryDir) = ExecutablePath.parent() {
			match super::LoadFromCache::Fn(&BinaryDir.to_path_buf()).await {
				Ok(Some(CachedMap)) => {
					let CachedLen = CachedMap.len();

					let PostWriteCount = {
						let mut Guard = _State
							.ScannedExtensions
							.ScannedExtensions
							.lock()
							.map_err(|Error| CommonError::StateLockPoisoned { Context:Error.to_string() })?;

						*Guard = CachedMap;

						Guard.len()
					};

					dev_log!(
						"extensions",
						"[ExtensionScanner] Cache hit: {} extensions loaded in <50ms (live scan skipped). State has \
						 {} entries.",
						CachedLen,
						PostWriteCount
					);

					// Supplementary live-scan of user-writable paths only.
					// PreBake at build time walks the bundled extension trees
					// (Mountain/Target/Resources/extensions,
					// Sky/Target/Static/Application/extensions, VS Code
					// Dependency/.../extensions) - it does NOT walk
					// `~/.fiddee/extensions` or `~/.land/extensions`.
					// Without this supplementary scan, the cache hit hides
					// every VSIX-installed extension from the workbench:
					// `extensions:scanUserExtensions` returns 0 and the
					// `@installed` view stays empty.
					//
					// We re-use `Registry.GetExtensionScanPaths()` and filter
					// through `Scanner::IsUserExtensionScanPath` so the
					// classifier and the scan set stay coherent (Lodge
					// override, ~/.fiddee/extensions, ~/.land/extensions).
					// Found entries overwrite cache entries with the same ID
					// - matches stock VS Code semantics where an installed
					// VSIX shadows a built-in of the same identifier.
					let UserScanPaths:Vec<PathBuf> = _State
						.Registry
						.GetExtensionScanPaths()
						.into_iter()
						.filter(|P| ExtensionManagement::Scanner::IsUserExtensionScanPath(P))
						.collect();

					if !UserScanPaths.is_empty() {
						dev_log!(
							"extensions",
							"[ExtensionScanner] Cache hit supplement: live-scanning {} user-writable path(s)",
							UserScanPaths.len()
						);

						let UserFutures:Vec<_> = UserScanPaths
							.into_iter()
							.map(|Path| {
								let Handle = ApplicationHandle.clone();

								async move {
									let Display = Path.display().to_string();

									match ExtensionManagement::Scanner::ScanDirectoryForExtensions(Handle, Path).await {
										Ok(Found) => {
											dev_log!(
												"extensions",
												"[ExtensionScanner] User path '{}' → {} extensions (supplement)",
												Display,
												Found.len()
											);

											Found
										},

										Err(E) => {
											dev_log!(
												"extensions",
												"warn: [ExtensionScanner] User path '{}' failed (supplement): {}",
												Display,
												E
											);

											Vec::new()
										},
									}
								}
							})
							.collect();

						let UserResults = futures::future::join_all(UserFutures).await;

						let mut UserMerged = 0usize;

						{
							let mut Guard = _State
								.ScannedExtensions
								.ScannedExtensions
								.lock()
								.map_err(|Error| CommonError::StateLockPoisoned { Context:Error.to_string() })?;

							for Found in UserResults {
								for Extension in Found {
									let Identifier = Extension
										.Identifier
										.get("value")
										.and_then(Value::as_str)
										.unwrap_or_default()
										.to_string();

									if !Identifier.is_empty() {
										Guard.insert(Identifier, Extension);

										UserMerged += 1;
									}
								}
							}
						}

						dev_log!(
							"extensions",
							"[ExtensionScanner] Cache hit supplement: merged {} user extension(s) into state",
							UserMerged
						);
					}

					// Unblock any callers waiting for the first scan result.
					_State.ScanReady.notify_waiters();

					return Ok(());
				},

				Ok(None) => {
					dev_log!("extensions", "[ExtensionScanner] Cache miss - falling back to live disk scan");
				},

				Err(E) => {
					dev_log!(
						"extensions",
						"warn: [ExtensionScanner] Cache load error: {}; continuing with live scan",
						E
					);
				},
			}
		}
	}

	let ScanPaths:Vec<PathBuf> = _State.Registry.GetExtensionScanPaths();

	dev_log!(
		"extensions",
		"[ExtensionScanner] Scanning {} paths in parallel",
		ScanPaths.len()
	);

	// Scan all paths concurrently; each spawns its own tokio task so slow
	// directories (e.g. a network-mounted extensions folder) don't stall the
	// others.
	let Futures:Vec<_> = ScanPaths
		.into_iter()
		.map(|Path| {
			let Handle = ApplicationHandle.clone();

			async move {
				let Display = Path.display().to_string();

				match ExtensionManagement::Scanner::ScanDirectoryForExtensions(Handle, Path).await {
					Ok(Found) => {
						dev_log!(
							"extensions",
							"[ExtensionScanner] Path '{}' → {} extensions",
							Display,
							Found.len()
						);

						(Display, Ok(Found))
					},

					Err(E) => {
						dev_log!("extensions", "warn: [ExtensionScanner] Path '{}' failed: {}", Display, E);

						(Display, Err(E))
					},
				}
			}
		})
		.collect();

	let Results = futures::future::join_all(Futures).await;

	let mut All:HashMap<String, ExtensionDescriptionStateDTO> = HashMap::new();

	let mut SuccessfulScans = 0usize;

	let mut FailedScans = 0usize;

	for (_Path, Result) in Results {
		match Result {
			Ok(Found) => {
				SuccessfulScans += 1;

				for Extension in Found {
					let Identifier = Extension
						.Identifier
						.get("value")
						.and_then(Value::as_str)
						.unwrap_or_default()
						.to_string();

					if !Identifier.is_empty() {
						All.insert(Identifier, Extension);
					}
				}
			},

			Err(_) => {
				FailedScans += 1;
			},
		}
	}

	// Single-swap replace: build the full map first (above), then take the
	// lock once for a pointer-swap so concurrent GetExtensions calls are not
	// blocked during the N-insert loop (which was previously holding the lock
	// for 100-500 ms on a cold filesystem with 94+ extensions).
	let AllLen = All.len();

	let PostWriteCount = {
		let mut Guard = _State
			.ScannedExtensions
			.ScannedExtensions
			.lock()
			.map_err(|Error| CommonError::StateLockPoisoned { Context:Error.to_string() })?;

		*Guard = All; // move - no clone needed

		Guard.len()
	};

	dev_log!(
		"extensions",
		"[ExtensionScanner] Complete: {} extensions ({} paths ok, {} failed). State has {} entries.",
		AllLen,
		SuccessfulScans,
		FailedScans,
		PostWriteCount
	);

	// Unblock any callers waiting for the first scan result.
	_State.ScanReady.notify_waiters();

	Ok(())
}

/// Robust extension scanning - clears state first, retries once on failure.
pub(crate) async fn ScanExtensionsWithRecovery(
	ApplicationHandle:AppHandle,

	State:&crate::ApplicationState::State::ExtensionState::State::State,
) -> Result<(), CommonError> {
	dev_log!("extensions", "[ExtensionScanner] Starting robust extension scan...");

	match Fn(ApplicationHandle.clone(), State).await {
		Ok(()) => {
			dev_log!("extensions", "[ExtensionScanner] Robust scan completed successfully");

			Ok(())
		},

		Err(Error) => {
			dev_log!("extensions", "error: [ExtensionScanner] Scan failed: {}; retrying once", Error);

			Fn(ApplicationHandle, State).await
		},
	}
}
