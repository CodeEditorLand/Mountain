#![allow(non_snake_case, unused_variables, dead_code, unused_imports)]

//! Wire method `file:mkdir`. `create_dir_all` is recursive; matches the
//! Electron default VS Code expects.

use serde_json::Value;

use crate::IPC::WindServiceHandlers::Utilities::PathExtraction::extract_path_from_arg;

pub async fn handle_file_mkdir_native(args:Vec<Value>) -> Result<Value, String> {
	let Path = extract_path_from_arg(args.get(0).ok_or("Missing directory path")?)?;

	tokio::fs::create_dir_all(&Path)
		.await
		.map_err(|E| format!("Failed to mkdir: {} ({})", Path, E))?;

	Ok(Value::Null)
}
