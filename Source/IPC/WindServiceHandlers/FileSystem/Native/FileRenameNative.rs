#![allow(non_snake_case, unused_variables, dead_code, unused_imports)]

//! Wire method `file:move` / `file:rename`.

use serde_json::Value;

use crate::IPC::WindServiceHandlers::Utilities::PathExtraction::extract_path_from_arg;

pub async fn handle_file_rename_native(args:Vec<Value>) -> Result<Value, String> {
	let Source = extract_path_from_arg(args.get(0).ok_or("Missing source path")?)?;
	let Target = extract_path_from_arg(args.get(1).ok_or("Missing target path")?)?;

	tokio::fs::rename(&Source, &Target)
		.await
		.map_err(|E| format!("Failed to rename: {} -> {} ({})", Source, Target, E))?;

	Ok(Value::Null)
}
