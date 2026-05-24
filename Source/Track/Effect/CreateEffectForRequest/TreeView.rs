//! Effect constructors for tree-view registration and disposal from the
//! Cocoon extension host. Delegates to `TreeViewProvider` on
//! `MountainEnvironment` and emits `SkyEvent` notifications to keep the
//! Sky workbench's `ITreeView` instances in sync.

use std::sync::Arc;

use CommonLibrary::{
	Environment::Requires::Requires,
	IPC::SkyEvent::SkyEvent,
	TreeView::TreeViewProvider::TreeViewProvider,
};
use serde_json::{Value, json};
use tauri::Runtime;

use crate::{
	IPC::SkyEmit::Fn,
	RunTime::ApplicationRunTime::ApplicationRunTime,
	Track::Effect::{
		CreateEffectForRequest::Utilities::Params::{StrAt, StringAt, ValAt},
		MappedEffectType::MappedEffect,
	},
	dev_log,
};

pub fn Fn<R:Runtime>(MethodName:&str, Parameters:Value) -> Option<Result<MappedEffect, String>> {
	match MethodName {
		"$tree:register" | "tree.Register" => {
			crate::effect!(RunTime, {
				let Provider:Arc<dyn TreeViewProvider> = RunTime.Environment.Require();

				let first = StrAt(&Parameters, 0);

				let (ViewId, Options) = if Parameters.get(2).is_some() {
					let vid = Parameters.get(1).and_then(Value::as_str).unwrap_or(first).to_string();
					let opts = ValAt(&Parameters, 2);
					(vid, opts)
				} else {
					let vid = first.to_string();
					let opts = ValAt(&Parameters, 1);
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
						&RunTime.Environment.ApplicationHandle,
						SkyEvent::TreeViewCreate.AsStr(),
						json!({ "viewId": ViewId, "options": Options }),
					) {
						dev_log!("tree-view", "warn: [TreeView] emit failed view={}: {}", ViewId, Error);
					}
				}

				Result.map(|_| json!(null)).map_err(|E| E.to_string())
			})
		},

		"tree.Unregister" | "tree.Dispose" => {
			crate::effect!(RunTime, {
				let ViewId = StringAt(&Parameters, 0);

				if ViewId.is_empty() {
					return Ok(json!(null));
				}

				dev_log!("tree-view", "[TreeView] unregister view={}", ViewId);

				let Provider:Arc<dyn TreeViewProvider> = RunTime.Environment.Require();

				let Result = provider.UnregisterTreeDataProvider(ViewId.clone()).await;

				if Result.is_ok() {
					if let Err(Error) = LogSkyEmit(
						&RunTime.Environment.ApplicationHandle,
						SkyEvent::TreeViewDispose.AsStr(),
						json!({ "viewId": ViewId }),
					) {
						dev_log!("tree-view", "warn: [TreeView] dispose emit failed view={}: {}", ViewId, Error);
					}
				}

				Result.map(|_| json!(null)).map_err(|E| E.to_string())
			})
		},

		// `treeView.reveal(element, options)` - extension asks Mountain to
		// scroll the native panel to a specific tree item. Previously only
		// existed in the Tauri IPC path (mod.rs), so Cocoon's gRPC
		// sendRequest("tree.reveal", ...) fell through to "Unknown method".
		"tree.reveal" => {
			crate::effect!(RunTime, {
				let Payload = if Parameters.is_object() {
					Parameters.clone()
				} else {
					json!({
						"viewId": ValAt(&Parameters, 0),
						"element": ValAt(&Parameters, 1),
						"options": ValAt(&Parameters, 2),
					})
				};
				if let Err(Error) =
					LogSkyEmit(&RunTime.Environment.ApplicationHandle, "sky://tree-view/reveal", Payload)
				{
					dev_log!("tree-view", "warn: [TreeView] reveal emit failed: {}", Error);
				}
				Ok(json!(null))
			})
		},

		_ => None,
	}
}
