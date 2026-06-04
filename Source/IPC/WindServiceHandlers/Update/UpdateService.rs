//! Wire methods: `update:*`.
//! Land has no update server yet; all methods are acknowledged no-ops that
//! match the shape VS Code's `IUpdateService` channel expects.
//! `_getInitialState` returns `{ type: "idle" }` which the workbench
//! renders as "up to date". `isLatestVersion` returns `true`.

use serde_json::{Value, json};

pub async fn UpdateGetInitialState() -> Result<Value, String> {

	crate::dev_log!("update", "update:_getInitialState");

	Ok(json!({ "type": "idle", "updateType": 0 }))
}

pub async fn UpdateIsLatestVersion() -> Result<Value, String> {

	crate::dev_log!("update", "update:isLatestVersion");

	Ok(json!(true))
}

pub async fn UpdateCheckForUpdates() -> Result<Value, String> {

	crate::dev_log!("update", "update:checkForUpdates");

	Ok(Value::Null)
}

pub async fn UpdateDownloadUpdate() -> Result<Value, String> {

	crate::dev_log!("update", "update:downloadUpdate");

	Ok(Value::Null)
}

pub async fn UpdateApplyUpdate() -> Result<Value, String> {

	crate::dev_log!("update", "update:applyUpdate");

	Ok(Value::Null)
}

pub async fn UpdateQuitAndInstall() -> Result<Value, String> {

	crate::dev_log!("update", "update:quitAndInstall");

	Ok(Value::Null)
}
