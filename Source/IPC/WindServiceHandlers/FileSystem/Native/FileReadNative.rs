#![allow(non_snake_case, unused_variables, dead_code, unused_imports)]

//! Wire method `file:read` / `file:readFile`.
//!
//! Returns `{ buffer: number[] }`. VS Code's `DiskFileSystemProviderClient`
//! wraps the payload with `VSBuffer.wrap()`. The explicit byte array is
//! required because `FileProtocolShim` used to return a struct with a
//! `number[]` field and any change to that shape breaks the Blob worker
//! round-trip - see `feedback_ipc_binary_fetch.md`.

use serde_json::{Value, json};

use crate::{IPC::WindServiceHandlers::Utilities::PathExtraction::extract_path_from_arg, dev_log};

pub async fn handle_file_read_native(args:Vec<Value>) -> Result<Value, String> {
	let Path = extract_path_from_arg(args.get(0).ok_or("Missing file path")?)?;

	dev_log!("vfs", "readFile: {}", Path);

	let Bytes = tokio::fs::read(&Path)
		.await
		.map_err(|E| format!("Failed to read file: {} (path: {})", E, Path))?;

	dev_log!("vfs", "readFile OK: {} ({} bytes)", Path, Bytes.len());

	let ByteArray:Vec<Value> = Bytes.iter().map(|B| json!(*B)).collect();
	Ok(json!({ "buffer": ByteArray }))
}
