#![allow(unused_variables, dead_code, unused_imports)]

//! Wire method `file:move` / `file:rename`.

use serde_json::{Value, json};

use crate::{IPC::WindServiceHandlers::Utilities::PathExtraction::Fn as extract_path_from_arg, dev_log};

pub async fn Fn(Arguments:Vec<Value>) -> Result<Value, String> {
	let Source = extract_path_from_arg(Arguments.get(0).ok_or("Missing source path")?)?;

	let Target = extract_path_from_arg(Arguments.get(1).ok_or("Missing target path")?)?;

	tokio::fs::rename(&Source, &Target)
		.await
		.map_err(|E| format!("Failed to rename: {} -> {} ({})", Source, Target, E))?;

	// Notify Cocoon so `onDidRenameFiles` fires for extensions (GitLens, etc.)
	let OldUri = format!("file://{}", Source);
	let NewUri = format!("file://{}", Target);
	dev_log!("vfs", "file:rename ok {} -> {}", Source, Target);
	tokio::spawn(async move {
		if let Err(Error) = crate::Vine::Client::SendNotification::Fn(
			"cocoon-main".to_string(),
			"$acceptDidRenameFiles".to_string(),
			json!({ "files": [{ "oldUri": OldUri, "newUri": NewUri }] }),
		)
		.await
		{
			dev_log!(
				"vfs",
				"warn: [FileRenameNative] $acceptDidRenameFiles notify failed: {:?}",
				Error
			);
		}
	});

	Ok(Value::Null)
}
