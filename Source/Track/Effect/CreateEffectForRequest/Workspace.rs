#![allow(non_snake_case, unused_variables, dead_code, unused_imports)]

use std::{future::Future, pin::Pin, sync::Arc};

use serde_json::{Value, json};
use tauri::Runtime;

use crate::{RunTime::ApplicationRunTime::ApplicationRunTime, Track::Effect::MappedEffectType::MappedEffect, dev_log};

pub fn CreateEffect<R:Runtime>(
	MethodName:&str,
	Parameters:Value,
) -> Option<Result<MappedEffect, String>> {
	match MethodName {
		"applyEdit" => {
			let effect =
				move |run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						use tauri::Emitter;
						let AppHandle = run_time.Environment.ApplicationHandle.clone();
						let Payload = if Parameters.is_array() {
							Parameters.get(0).cloned().unwrap_or_default()
						} else {
							Parameters
						};
						let _ = AppHandle.emit("sky://workspace/applyEdit", Payload);
						Ok(json!(true))
					})
				};
			Some(Ok(Box::new(effect)))
		},

		"showTextDocument" => {
			let effect =
				move |run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						use tauri::Emitter;
						let AppHandle = run_time.Environment.ApplicationHandle.clone();
						let _ = AppHandle.emit("sky://window/showTextDocument", &Parameters);
						Ok(json!(null))
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
											.and_then(|U| {
												U.get("value")
													.and_then(Value::as_str)
													.or_else(|| U.as_str())
											})
											.map(str::to_string)?;
										let Name = Entry
											.get("name")
											.and_then(Value::as_str)
											.unwrap_or("")
											.to_string();
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
											.and_then(|U| {
												U.get("value")
													.and_then(Value::as_str)
													.or_else(|| U.as_str())
											})
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
								if let Ok(Dto) = crate::ApplicationState::DTO::WorkspaceFolderStateDTO::WorkspaceFolderStateDTO::New(
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
