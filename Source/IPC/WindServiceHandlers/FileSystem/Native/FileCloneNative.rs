//! Wire method `file:copy` / `file:cloneFile`. `tokio::fs::copy`
//! preserves content but not xattrs/acls; callers that need metadata
//! should use an OS-specific clone atom (future work).

use serde_json::{Value, json};

use crate::{IPC::WindServiceHandlers::Utilities::PathExtraction::Fn as extract_path_from_arg, dev_log};

pub async fn Fn(Arguments:Vec<Value>) -> Result<Value, String> {
	let Source = extract_path_from_arg(Arguments.get(0).ok_or("Missing source path")?)?;

	let Target = extract_path_from_arg(Arguments.get(1).ok_or("Missing target path")?)?;

	// Ensure the target parent directory exists (VS Code local history creates
	// per-file history dirs that may not exist yet).
	if let Some(Parent) = std::path::Path::new(&Target).parent() {
		if !Parent.as_os_str().is_empty() {
			tokio::fs::create_dir_all(Parent)
				.await
				.map_err(|E| format!("Failed to create target dir {}: {}", Parent.display(), E))?;
		}
	}

	tokio::fs::copy(&Source, &Target)
		.await
		.map_err(|E| format!("Failed to clone: {} -> {} ({})", Source, Target, E))?;

	// Notify Cocoon so `onDidCreateFiles` fires for the newly copied file.
	let NewUri = format!("file://{}", Target);

	dev_log!("vfs", "file:clone ok {} -> {}", Source, Target);

	tokio::spawn(async move {
		if let Err(Error) = crate::Vine::Client::SendNotification::Fn(
			"cocoon-main".to_string(),
			"$acceptDidCreateFiles".to_string(),
			json!({ "files": [{ "uri": NewUri }] }),
		)
		.await
		{
			dev_log!(
				"vfs",
				"warn: [FileCloneNative] $acceptDidCreateFiles notify failed: {:?}",
				Error
			);
		}
	});

	Ok(Value::Null)
}
