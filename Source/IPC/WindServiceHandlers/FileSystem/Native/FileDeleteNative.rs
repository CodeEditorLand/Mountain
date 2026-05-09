#![allow(non_snake_case, unused_variables, dead_code, unused_imports)]

//! Wire method `file:delete`. Honours `{ recursive }` option for
//! directories; `useTrash` is accepted but not yet implemented (future
//! atom: trash.rs on macOS/Linux via `trash-rs`, Windows via SHFileOp).

use serde_json::Value;

use crate::IPC::WindServiceHandlers::Utilities::PathExtraction::extract_path_from_arg;

pub async fn FileDeleteNative(Arguments:Vec<Value>) -> Result<Value, String> {
	let Path = extract_path_from_arg(Arguments.get(0).ok_or("Missing file path")?)?;

	let Recursive = Arguments
		.get(1)
		.and_then(|V| V.as_object())
		.and_then(|O| O.get("recursive"))
		.and_then(|V| V.as_bool())
		.unwrap_or(false);

	let PathBuf = std::path::Path::new(&Path);

	if PathBuf.is_dir() {
		if Recursive {
			tokio::fs::remove_dir_all(&Path).await
		} else {
			tokio::fs::remove_dir(&Path).await
		}
	} else {
		tokio::fs::remove_file(&Path).await
	}
	.map_err(|E| format!("Failed to delete: {} ({})", Path, E))?;

	Ok(Value::Null)
}
