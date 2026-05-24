//! Wire method `file:mkdir`. `create_dir_all` is recursive; matches the
//! Electron default VS Code expects.

use serde_json::{Value, json};

use crate::{IPC::WindServiceHandlers::Utilities::PathExtraction::Fn as ExtractPathFromArg, dev_log};

pub async fn Fn(Arguments:Vec<Value>) -> Result<Value, String> {
	let Path = ExtractPathFromArg(Arguments.get(0).ok_or("Missing directory path")?)?;

	tokio::fs::create_dir_all(&Path)
		.await
		.map_err(|E| format!("Failed to mkdir: {} ({})", Path, E))?;

	dev_log!("vfs", "file:mkdir ok path={}", Path);

	let FileUri = format!("file://{}", Path);

	tokio::spawn(async move {
		if let Err(Error) = crate::Vine::Client::SendNotification::Fn(
			"cocoon-main".to_string(),
			"$acceptDidCreateFiles".to_string(),
			json!({ "files": [{ "uri": FileUri }] }),
		)
		.await
		{
			dev_log!(
				"vfs",
				"warn: [FileMkdirNative] $acceptDidCreateFiles notify failed: {:?}",
				Error
			);
		}
	});

	Ok(Value::Null)
}
