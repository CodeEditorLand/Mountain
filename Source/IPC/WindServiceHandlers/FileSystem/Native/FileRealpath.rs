#![allow(non_snake_case, unused_variables, dead_code, unused_imports)]

//! Wire method `file:realpath`. Emits a VS Code `UriComponents` (`$mid: 1`)
//! so the renderer reviver promotes it to a real `URI` with `.fsPath` /
//! `.with`. Plain string would be treated as a relative path.

use serde_json::Value;

use crate::IPC::{
	UriComponents::FromFilePath::Fn as UriFromFilePath,
	WindServiceHandlers::Utilities::PathExtraction::extract_path_from_arg,
};

pub async fn FileRealpath(Arguments:Vec<Value>) -> Result<Value, String> {
	let Path = extract_path_from_arg(Arguments.get(0).ok_or("Missing path")?)?;

	let Canonical = tokio::fs::canonicalize(&Path)
		.await
		.map_err(|E| format!("Failed to realpath: {} ({})", Path, E))?;

	Ok(UriFromFilePath(Canonical.to_string_lossy()))
}
