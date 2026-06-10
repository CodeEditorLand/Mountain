//! Wire method: `tree:getChildren`.
//! Renderer-side tree-view child lookup. Mirrors the Cocoon→Mountain
//! `GetTreeChildren` gRPC path but is invoked directly by the Wind/Sky
//! tree-view bridge so the UI can request children without waiting for
//! Cocoon to ask first.
//!
//! Waits up to 5 s for the Cocoon gRPC connection (boot-race guard) then
//! dispatches `$provideTreeChildren` with a 5 s timeout. On timeout or
//! extension rejection the workbench receives `{ items: [] }` and schedules
//! its own retry when the view scrolls back into view.

use std::sync::Arc;

use serde_json::{Value, json};
use tauri::AppHandle;

use crate::RunTime::ApplicationRunTime::ApplicationRunTime;

pub async fn Fn(
	ApplicationHandle:AppHandle,

	RunTime:Arc<ApplicationRunTime>,

	Arguments:Vec<Value>,
) -> Result<Value, String> {
	let ViewId = Arguments
		.first()
		.and_then(|V| V.get("viewId").or_else(|| V.get(0)))
		.and_then(Value::as_str)
		.unwrap_or("")
		.to_string();

	let ItemHandle = Arguments
		.first()
		.and_then(|V| V.get("treeItemHandle").or_else(|| V.get(1)))
		.and_then(Value::as_str)
		.unwrap_or("")
		.to_string();

	crate::dev_log!(
		"tree-view",
		"[TreeView] invoke:getChildren view={} parent={}",
		ViewId,
		ItemHandle
	);

	if ViewId.is_empty() {
		return Err("tree:getChildren requires viewId".to_string());
	}

	let Parameters = json!({
		"viewId": ViewId,
		"treeItemHandle": ItemHandle,
	});

	// Boot-race: the workbench's Explorer view fires `tree:getChildren` ~700
	// log lines before Cocoon's gRPC client finishes handshaking. Without
	// this wait the first call returns `ClientNotConnected`, the workbench
	// caches an empty list, and the Explorer never recovers without a manual
	// refresh. 5000 ms chosen from boot-trace observation.
	let _ = ::Vine::Client::WaitForClientConnection::Fn("cocoon-main", 5000).await;

	match ::Vine::Client::SendRequest::Fn(
		"cocoon-main",
		"$provideTreeChildren".to_string(),
		Parameters,
		// 15000 ms: warm tree calls take 1-50 ms but cold scans can
		// blow well past 5 s on first activation (`npm` walks every
		// `package.json` in the workspace; `vscode.git` enumerates
		// every nested submodule). 5 s repeatedly tripped on multi-
		// repo workspaces like Land itself, so the panel rendered
		// empty until the user manually refreshed. Bumping to 15 s
		// keeps the upper bound bounded (renderer still receives
		// `{items:[]}` on real hang) while letting genuine cold
		// scans complete.
		15000,
	)
	.await
	{
		Ok(Value_) => {
			match &Value_ {
				Value::Object(_) | Value::Array(_) => Ok(Value_),

				// Non-conforming shape: force to {items:[]} so the renderer
				// always has iterable data and avoids TypeError crashes.
				_ => Ok(json!({ "items": [] })),
			}
		},

		Err(Error) => {
			// Log first failure per view; silence repeats so the dev log
			// doesn't fill with identical lines while the user browses
			// nodes from a misbehaving extension.
			crate::IPC::DevLog::DebugOnce::Fn(
				"tree-view",
				&format!("get-children-error:{}", ViewId),
				&format!(
					"[TreeView] invoke:getChildren error view={} err={:?} (further occurrences silenced)",
					ViewId, Error
				),
			);

			Ok(json!({ "items": [] }))
		},
	}
}
