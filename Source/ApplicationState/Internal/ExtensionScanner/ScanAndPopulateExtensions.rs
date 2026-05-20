use std::{collections::HashMap, path::PathBuf};

use CommonLibrary::Error::CommonError::CommonError;
use serde_json::Value;
use tauri::AppHandle;

use crate::{
	ApplicationState::DTO::ExtensionDescriptionStateDTO::ExtensionDescriptionStateDTO,
	ExtensionManagement,
	dev_log,
};

pub async fn ScanAndPopulateExtensions(
	ApplicationHandle:AppHandle,

	_State:&crate::ApplicationState::State::ExtensionState::State::State,
) -> Result<(), CommonError> {
	dev_log!("extensions", "[ExtensionScanner] Starting extension scan...");

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
pub async fn ScanExtensionsWithRecovery(
	ApplicationHandle:AppHandle,

	State:&crate::ApplicationState::State::ExtensionState::State::State,
) -> Result<(), CommonError> {
	dev_log!("extensions", "[ExtensionScanner] Starting robust extension scan...");

	match ScanAndPopulateExtensions(ApplicationHandle.clone(), State).await {
		Ok(()) => {
			dev_log!("extensions", "[ExtensionScanner] Robust scan completed successfully");

			Ok(())
		},

		Err(Error) => {
			dev_log!("extensions", "error: [ExtensionScanner] Scan failed: {}; retrying once", Error);

			ScanAndPopulateExtensions(ApplicationHandle, State).await
		},
	}
}
