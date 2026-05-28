//! # Workspace Effect (CreateEffectForRequest)
//!
//! Effect constructors for workspace-level RPC methods. Handles:
//! - `applyEdit` and `showTextDocument` via round-trip to Sky through
//!   `UserInterfaceProvider::SendUserInterfaceRequest` (resolves when Sky has
//!   actually applied the edit or shown the document).
//! - `Workspace.RequestResourceTrust` and `Workspace.IsResourceTrusted` return
//!   a permissive `true` heuristic so `vscode.git` proceeds; single- window dev
//!   runtime stays trust-by-default.
//! - `$updateWorkspaceFolders` applies workspace folder additions/removals to
//!   `ApplicationState.Workspace` and broadcasts the delta.

use std::sync::Arc;

use serde_json::{Value, json};
use tauri::Runtime;

use crate::{
	RunTime::ApplicationRunTime::ApplicationRunTime,
	Track::Effect::{
		CreateEffectForRequest::Utilities::Params::{array_unwrap, uri_from_params},
		MappedEffectType::MappedEffect,
	},
	dev_log,
};

pub fn CreateEffect<R:Runtime>(MethodName:&str, Parameters:Value) -> Option<Result<MappedEffect, String>> {
	match MethodName {
		"applyEdit" => {
			crate::effect!(run_time, {
				// Atom T1: round-trip via Mountain's request/reply plumbing so the
				// extension's `await workspace.applyEdit(…)` resolves when Sky has
				// actually applied the edit (or refused). Previously a synthetic
				// `true` returned before the edit ran, racing listeners that
				// expected post-apply state.
				let Payload = if Parameters.is_array() {
					Parameters.get(0).cloned().unwrap_or_default()
				} else {
					Parameters
				};

				crate::Environment::UserInterfaceProvider::SendUserInterfaceRequest(
					&run_time.Environment,
					"sky://workspace/applyEdit",
					Payload,
				)
				.await
				.map_err(|Error| {
					dev_log!("ipc", "error: [applyEdit] Sky did not answer ({:?})", Error);

					Error.to_string()
				})
			})
		},

		"showTextDocument" => {
			crate::effect!(run_time, {
				// Atom T1: same round-trip as applyEdit. The canonical vscode
				// return shape is a `TextEditor` - today Sky resolves with a
				// thin `{ uri, viewColumn }` stub. Extensions that chain
				// editor ops may still see undefined properties; that's a
				// Sky-side enrichment task (T2 follow-up).
				match crate::Environment::UserInterfaceProvider::SendUserInterfaceRequest(
					&run_time.Environment,
					"sky://window/showTextDocument",
					Parameters,
				)
				.await
				{
					Ok(Value) => Ok(Value),
					Err(Error) => {
						dev_log!(
							"ipc",
							"warn: [showTextDocument] Sky did not answer ({:?}); returning null",
							Error
						);

						Ok(json!(null))
					},
				}
			})
		},

		// `editor.revealRange(range, revealType)` - scroll the Monaco editor to
		// bring a range into view. Extensions use this for go-to-definition
		// "reveal cursor", reference highlights, error navigation, etc.
		// Routes to Sky's ICodeEditorService so Monaco scrolls its viewport.
		"window.revealRange" => {
			crate::effect!(run_time, {
				match crate::Environment::UserInterfaceProvider::SendUserInterfaceRequest(
					&run_time.Environment,
					"sky://editor/revealRange",
					Parameters,
				)
				.await
				{
					Ok(V) => Ok(V),
					Err(_) => Ok(json!(null)),
				}
			})
		},

		// Workspace-trust family. vscode.git's `Model.openRepository` calls
		// `await workspace.requestResourceTrust({uri, message})` and
		// `await workspace.isResourceTrusted(uri)` before constructing the
		// Repository. The Cocoon `WrapWorkspaceNamespace` Proxy fallback
		// already returns a permissive `true` heuristic so vscode.git
		// proceeds; routing the same method names through Mountain here
		// gives the canonical handler a place to live (and makes
		// `MountainMethods` see them via `GenerateRouteManifest.sh`'s grep,
		// which switches the Cocoon shim from heuristic-default to
		// gRPC-routed automatically on the next manifest regeneration). A
		// future round can replace the unconditional `true` with a real
		// per-OS trust query (Gatekeeper / SmartScreen / xattrs); single-
		// window dev runtime stays trust-by-default.
		"Workspace.RequestResourceTrust" | "Workspace.IsResourceTrusted" => {
			crate::effect!(_run_time, { Ok(json!({ "trusted": true })) })
		},

		"$updateWorkspaceFolders" => {
			crate::effect!(run_time, {
				let Payload = array_unwrap(Parameters);

				let Additions:Vec<(String, String)> = Payload
					.get("additions")
					.and_then(Value::as_array)
					.map(|Array| {
						Array
							.iter()
							.filter_map(|Entry| {
								let Uri = Entry
									.get("uri")
									.and_then(|U| U.get("value").and_then(Value::as_str).or_else(|| U.as_str()))
									.map(str::to_string)?;

								let Name = Entry.get("name").and_then(Value::as_str).unwrap_or("").to_string();

								Some((Uri, Name))
							})
							.collect()
					})
					.unwrap_or_default();

				let Removals:Vec<String> = Payload
					.get("removals")
					.and_then(Value::as_array)
					.map(|Array| {
						Array
							.iter()
							.filter_map(|Entry| {
								Entry
									.get("uri")
									.and_then(|U| U.get("value").and_then(Value::as_str).or_else(|| U.as_str()))
									.map(str::to_string)
							})
							.collect()
					})
					.unwrap_or_default();

				let Workspace = &run_time.Environment.ApplicationState.Workspace;

				let mut Folders = Workspace.GetWorkspaceFolders();

				Folders.retain(|F| !Removals.contains(&F.URI.to_string()));

				let Base = Folders.len();

				for (Index, (UriStr, Name)) in Additions.iter().enumerate() {
					if let Ok(Url) = url::Url::parse(UriStr) {
						if let Ok(Dto) =
							crate::ApplicationState::DTO::WorkspaceFolderStateDTO::WorkspaceFolderStateDTO::New(
								Url,
								Name.clone(),
								Base + Index,
							) {
							Folders.push(Dto);
						}
					}
				}

				crate::ApplicationState::State::WorkspaceState::WorkspaceDelta::UpdateWorkspaceFoldersAndNotify(
					Workspace, Folders,
				);

				Ok(json!(null))
			})
		},

		// `workspace.save(uri)` - Cocoon's shim calls this when an extension
		// calls `vscode.workspace.save(uri)`. Route through Sky so the workbench's
		// `ITextFileService.save(uri)` can flush the dirty working copy to disk.
		// Returns the URI on success so the caller can confirm the file was saved.
		"Workspace.Save" => {
			crate::effect!(run_time, {
				let UriVal = uri_from_params(Parameters);

				// Fire `document.willSave` to Cocoon BEFORE writing to disk.
				// This gives `onWillSaveTextDocument` listeners a chance to
				// apply last-minute edits (format-on-save, organize-imports,
				// trailing-whitespace strippers, etc.).
				// Fire-and-forget with a short grace period so slow listeners
				// don't stall the save for more than 1.5 s.
				let WillSavePayload = serde_json::json!({
					"uri": UriVal,
					"reason": 1, // TextDocumentSaveReason.Manual
				});

				let _ = tokio::time::timeout(
					std::time::Duration::from_millis(1500),
					::Vine::Client::SendNotification::Fn(
						"cocoon-main".to_string(),
						"document.willSave".to_string(),
						WillSavePayload,
					),
				)
				.await;

				let SaveResult = match crate::Environment::UserInterfaceProvider::SendUserInterfaceRequest(
					&run_time.Environment,
					"sky://workspace/save",
					UriVal.clone(),
				)
				.await
				{
					Ok(Result) => {
						if Result.is_null() {
							UriVal.clone()
						} else {
							Result
						}
					},
					Err(Error) => {
						dev_log!("ipc", "warn: [Workspace.Save] Sky did not answer ({:?}); ok", Error);

						UriVal.clone()
					},
				};

				// Notify Cocoon that the file was saved so `onDidSaveTextDocument`
				// fires for extension-triggered saves (format-on-save, etc.).
				let _ = ::Vine::Client::SendNotification::Fn(
					"cocoon-main".to_string(),
					"$acceptModelSaved".to_string(),
					serde_json::json!({ "uri": UriVal }),
				)
				.await;

				Ok(SaveResult)
			})
		},

		// `workspace.saveAs(uri)` - same as Save but opens a Save-As dialog.
		// Currently delegates to the same Save path; a future Sky-side handler
		// can drive the dialog independently.
		"Workspace.SaveAs" => {
			crate::effect!(run_time, {
				let UriVal = uri_from_params(Parameters);

				match crate::Environment::UserInterfaceProvider::SendUserInterfaceRequest(
					&run_time.Environment,
					"sky://workspace/saveAs",
					UriVal.clone(),
				)
				.await
				{
					Ok(Result) => Ok(if Result.is_null() { UriVal } else { Result }),
					Err(_) => Ok(UriVal),
				}
			})
		},

		// `saveAll` - Cocoon's older API surface calls this from `gRPC/Client.ts`
		// when the workbench wants to flush all dirty working copies. Routes to
		// Sky's `sky://workspace/saveAll` handler which delegates to VS Code's
		// `ITextFileService.save({ saveReason: AutoSave })` for all dirty models.
		"saveAll" | "Workspace.SaveAll" => {
			crate::effect!(run_time, {
				match crate::Environment::UserInterfaceProvider::SendUserInterfaceRequest(
					&run_time.Environment,
					"sky://workspace/saveAll",
					serde_json::json!({}),
				)
				.await
				{
					Ok(Result) => Ok(Result),
					Err(Error) => {
						dev_log!("ipc", "warn: [saveAll] Sky did not answer ({:?}); ok", Error);

						Ok(serde_json::json!(null))
					},
				}
			})
		},

		_ => None,
	}
}
