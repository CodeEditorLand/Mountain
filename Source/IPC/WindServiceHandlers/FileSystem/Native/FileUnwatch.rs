#![allow(non_snake_case)]

//! `file:unwatch` - unregister a native filesystem watcher by its token.
//!
//! VS Code's `DiskFileSystemProvider` calls this when an extension disposes
//! its `FileSystemWatcher` or when the workspace is closed.
//!
//! Arguments[0] = the numeric token string returned by `file:watch`.

use std::sync::Arc;

use CommonLibrary::FileSystem::FileWatcherProvider::FileWatcherProvider;
use serde_json::Value;

use crate::{RunTime::ApplicationRunTime::ApplicationRunTime, dev_log};

pub async fn FileUnwatch(RunTime:Arc<ApplicationRunTime>, Arguments:Vec<Value>) -> Result<Value, String> {
	let Token = Arguments
		.first()
		.and_then(|V| {
			// Handle both numeric and string tokens.
			V.as_str().map(str::to_owned).or_else(|| V.as_u64().map(|N| N.to_string()))
		})
		.unwrap_or_default();

	if Token.is_empty() {
		return Ok(Value::Null);
	}

	dev_log!("filewatcher", "file:unwatch handle={}", Token);

	RunTime
		.Environment
		.UnregisterWatcher(Token)
		.await
		.map_err(|E| format!("file:unwatch: {E}"))?;

	Ok(Value::Null)
}
