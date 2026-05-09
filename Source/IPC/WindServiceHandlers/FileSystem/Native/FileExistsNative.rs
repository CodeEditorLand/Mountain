#![allow(non_snake_case, unused_variables, dead_code, unused_imports)]

//! Wire method `file:exists`. Boolean probe via `tokio::fs::try_exists`.

use serde_json::{Value, json};

use crate::IPC::WindServiceHandlers::Utilities::PathExtraction::extract_path_from_arg;

pub async fn FileExistsNative(Arguments:Vec<Value>) -> Result<Value, String> {
	let Path = extract_path_from_arg(Arguments.get(0).ok_or("Missing file path")?)?;

	Ok(json!(tokio::fs::try_exists(&Path).await.unwrap_or(false)))
}
