#![allow(non_snake_case)]

//! `file:watch` - register a native filesystem watcher for a URI.
//!
//! VS Code's `DiskFileSystemProvider.watch(resource, opts)` sends:
//!   Arguments[0] = resource URI (object or path string)
//!   Arguments[1] = `{ recursive: boolean, excludes: string[] }`
//!
//! Returns a numeric token that the caller passes to `file:unwatch`.
//! The token is stored as the string key in `WatcherProvider::RegisterWatcher`.
//!
//! Events flow through `FileWatcherProvider` → `$fileWatcher:event` gRPC
//! notification → Cocoon → `onDidChangeFile` extension API.

use std::{
	path::PathBuf,
	sync::{
		Arc,
		atomic::{AtomicU64, Ordering},
	},
};

use CommonLibrary::FileSystem::FileWatcherProvider::FileWatcherProvider;
use serde_json::{Value, json};

use crate::{
	IPC::WindServiceHandlers::Utilities::PathExtraction::Fn as extract_path_from_arg,
	RunTime::ApplicationRunTime::ApplicationRunTime,
	dev_log,
};

static WATCH_HANDLE_SEQ:AtomicU64 = AtomicU64::new(1);

pub async fn Fn(RunTime:Arc<ApplicationRunTime>, Arguments:Vec<Value>) -> Result<Value, String> {
	let ResourceArg = Arguments.get(0).ok_or("file:watch: missing resource")?;

	let Path = extract_path_from_arg(ResourceArg)?;

	let Opts = Arguments.get(1).and_then(Value::as_object);

	let IsRecursive = Opts.and_then(|O| O.get("recursive")).and_then(Value::as_bool).unwrap_or(false);

	let Pattern:Option<String> = Opts.and_then(|O| O.get("excludes")).and_then(Value::as_array).and_then(|Arr| {
		let Globs:Vec<&str> = Arr.iter().filter_map(Value::as_str).collect();

		if Globs.is_empty() { None } else { Some(Globs.join("|")) }
	});

	let Handle = WATCH_HANDLE_SEQ.fetch_add(1, Ordering::Relaxed).to_string();

	dev_log!(
		"filewatcher",
		"file:watch handle={} path={} recursive={} pattern={:?}",
		Handle,
		Path,
		IsRecursive,
		Pattern
	);

	let Root = PathBuf::from(&Path);

	RunTime
		.Environment
		.RegisterWatcher(Handle.clone(), Root, IsRecursive, Pattern)
		.await
		.map_err(|E| format!("file:watch: {E}"))?;

	Ok(json!(Handle))
}
