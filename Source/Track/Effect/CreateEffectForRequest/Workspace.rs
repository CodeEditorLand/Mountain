#![allow(non_snake_case, unused_variables, dead_code, unused_imports)]

use std::{future::Future, pin::Pin, sync::Arc};

use serde_json::{Value, json};
use tauri::Runtime;

use crate::{RunTime::ApplicationRunTime::ApplicationRunTime, Track::Effect::MappedEffectType::MappedEffect, dev_log};

pub fn CreateEffect<R:Runtime>(MethodName:&str, Parameters:Value) -> Option<Result<MappedEffect, String>> {
	match MethodName {
		"applyEdit" => {
			let effect =
				move |run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
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
							dev_log!(
								"ipc",
								"error: [applyEdit] Sky did not answer ({:?})",
								Error
							);
							Error.to_string()
						})
					})
				};

			Some(Ok(Box::new(effect)))
		},

		"showTextDocument" => {
			let effect =
				move |run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
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
				};

			Some(Ok(Box::new(effect)))
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
			let effect =
				move |_run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {

					Box::pin(async move {
						Ok(json!({ "trusted": true }))
					})
				};

			Some(Ok(Box::new(effect)))
		},

		"$updateWorkspaceFolders" => {
			let effect =
				move |run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						let Payload = if Parameters.is_array() {
							Parameters.get(0).cloned().unwrap_or_default()
						} else {
							Parameters
						};
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
				};

			Some(Ok(Box::new(effect)))
		},

		_ => None,
	}
}
