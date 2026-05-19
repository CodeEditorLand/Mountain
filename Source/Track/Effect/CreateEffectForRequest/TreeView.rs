#![allow(non_snake_case, unused_variables, dead_code, unused_imports)]

//! Effect constructors for tree-view registration and disposal from the
//! Cocoon extension host. Delegates to `TreeViewProvider` on
//! `MountainEnvironment` and emits `SkyEvent` notifications to keep the
//! Sky workbench's `ITreeView` instances in sync.

use std::{future::Future, pin::Pin, sync::Arc};

use CommonLibrary::{
	Environment::Requires::Requires,
	IPC::SkyEvent::SkyEvent,
	TreeView::TreeViewProvider::TreeViewProvider,
};
use serde_json::{Value, json};
use tauri::Runtime;

use crate::{
	IPC::SkyEmit::LogSkyEmit,
	RunTime::ApplicationRunTime::ApplicationRunTime,
	Track::Effect::MappedEffectType::MappedEffect,
	dev_log,
};

pub fn CreateEffect<R:Runtime>(MethodName:&str, Parameters:Value) -> Option<Result<MappedEffect, String>> {
	match MethodName {
		"$tree:register" | "tree.register" => {
			let effect =
				move |run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						let provider:Arc<dyn TreeViewProvider> = run_time.Environment.Require();

						let first = Parameters.get(0).and_then(Value::as_str).unwrap_or("");

						let (ViewId, Options) = if Parameters.get(2).is_some() {
							let vid = Parameters.get(1).and_then(Value::as_str).unwrap_or(first).to_string();
							let opts = Parameters.get(2).cloned().unwrap_or_default();
							(vid, opts)
						} else {
							let vid = first.to_string();
							let opts = Parameters.get(1).cloned().unwrap_or_default();
							(vid, opts)
						};

						dev_log!("tree-view", "[TreeView] register view={}", ViewId);

						let Result = provider.RegisterTreeDataProvider(ViewId.clone(), Options.clone()).await;

						dev_log!(
							"tree-view",
							"[TreeView] register view={} result={}",
							ViewId,
							if Result.is_ok() { "ok" } else { "err" }
						);

						if Result.is_ok() {
							if let Err(Error) = LogSkyEmit(
								&run_time.Environment.ApplicationHandle,
								SkyEvent::TreeViewCreate.AsStr(),
								json!({ "viewId": ViewId, "options": Options }),
							) {
								dev_log!("tree-view", "warn: [TreeView] emit failed view={}: {}", ViewId, Error);
							}
						}

						Result.map(|_| json!(null)).map_err(|E| E.to_string())
					})
				};

			Some(Ok(Box::new(effect)))
		},

		"tree.unregister" | "tree.dispose" => {
			let effect =
				move |run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						let ViewId = Parameters.get(0).and_then(Value::as_str).unwrap_or("").to_string();

						if ViewId.is_empty() {
							return Ok(json!(null));
						}

						dev_log!("tree-view", "[TreeView] unregister view={}", ViewId);

						let provider:Arc<dyn TreeViewProvider> = run_time.Environment.Require();

						let Result = provider.UnregisterTreeDataProvider(ViewId.clone()).await;

						if Result.is_ok() {
							if let Err(Error) = LogSkyEmit(
								&run_time.Environment.ApplicationHandle,
								SkyEvent::TreeViewDispose.AsStr(),
								json!({ "viewId": ViewId }),
							) {
								dev_log!(
									"tree-view",
									"warn: [TreeView] dispose emit failed view={}: {}",
									ViewId,
									Error
								);
							}
						}

						Result.map(|_| json!(null)).map_err(|E| E.to_string())
					})
				};

			Some(Ok(Box::new(effect)))
		},

		_ => None,
	}
}
