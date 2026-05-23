#![allow(non_snake_case)]

//! `file:open` - open a file and return an integer file descriptor.
//!
//! VS Code's `DiskFileSystemProvider.open(resource, opts)` uses fd-based
//! access for large binary files and write operations. The fd table is a
//! process-global `HashMap<u32, File>` guarded by a `Mutex`. FDs start
//! from 1 and increment with each successful open.
//!
//! Arguments[0] = resource URI or path string
//! Arguments[1] = options: `{ create?: boolean, unlock?: boolean }`
//!
//! Returns: integer fd number, or 0 on error (VS Code ignores the error
//! for read-only opens and falls back to the full-read path).

use std::{
	collections::HashMap,
	sync::{
		Mutex,
		OnceLock,
		atomic::{AtomicU32, Ordering},
	},
};

use serde_json::{Value, json};
use tokio::fs::File;

use crate::{IPC::WindServiceHandlers::Utilities::PathExtraction::Fn as extract_path_from_arg, dev_log};

static NEXT_FD:AtomicU32 = AtomicU32::new(1);

pub struct FdTable {
	pub Files:Mutex<HashMap<u32, File>>,
}

static FD_TABLE:OnceLock<FdTable> = OnceLock::new();

pub(crate) fn GetFdTable() -> &'static FdTable { FD_TABLE.get_or_init(|| FdTable { Files:Mutex::new(HashMap::new()) }) }

pub async fn Fn(Arguments:Vec<Value>) -> Result<Value, String> {
	let ResourceArg = Arguments.first().ok_or("file:open: missing resource")?;

	let Path = extract_path_from_arg(ResourceArg)?;

	let Opts = Arguments.get(1).and_then(Value::as_object);

	let Create = Opts.and_then(|O| O.get("create")).and_then(Value::as_bool).unwrap_or(false);

	let Truncate = Opts.and_then(|O| O.get("truncate")).and_then(Value::as_bool).unwrap_or(false);

	let F = if Create {
		let mut OpenOpts = tokio::fs::OpenOptions::new();

		OpenOpts.write(true).create(true);

		if Truncate {
			OpenOpts.truncate(true);
		}

		OpenOpts
			.open(&Path)
			.await
			.map_err(|E| format!("file:open create '{}': {}", Path, E))?
	} else {
		tokio::fs::File::open(&Path)
			.await
			.map_err(|E| format!("file:open '{}': {}", Path, E))?
	};

	let Fd = NEXT_FD.fetch_add(1, Ordering::Relaxed);

	if let Ok(mut Table) = GetFdTable().Files.lock() {
		Table.insert(Fd, F);
	}

	dev_log!("vfs", "file:open fd={} path={} create={}", Fd, Path, Create);

	Ok(json!(Fd))
}
