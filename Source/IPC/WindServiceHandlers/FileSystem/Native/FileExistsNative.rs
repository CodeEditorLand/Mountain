//! Wire method `file:exists`. Boolean probe via `tokio::fs::try_exists`.

use serde_json::{Value, json};

use crate::IPC::WindServiceHandlers::Utilities::PathExtraction::Fn as ExtractPathFromArg;

pub async fn Fn(Arguments:Vec<Value>) -> Result<Value, String> {
	let Path = ExtractPathFromArg(Arguments.get(0).ok_or("Missing file path")?)?;

	// Propagate I/O errors (permission denied, broken symlink) rather than
	// returning false. unwrap_or(false) would make errors look like "not found",
	// causing VS Code to overwrite existing files it cannot read.
	let Exists = tokio::fs::try_exists(&Path)
		.await
		.map_err(|E| format!("file:exists I/O error for {}: {}", Path, E))?;

	Ok(json!(Exists))
}
