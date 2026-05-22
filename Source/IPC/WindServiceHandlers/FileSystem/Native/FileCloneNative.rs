#![allow(non_snake_case, unused_variables, dead_code, unused_imports)]

//! Wire method `file:copy` / `file:cloneFile`. `tokio::fs::copy`
//! preserves content but not xattrs/acls; callers that need metadata
//! should use an OS-specific clone atom (future work).

use serde_json::Value;

use crate::IPC::WindServiceHandlers::Utilities::PathExtraction::extract_path_from_arg;

pub async fn FileCloneNative(Arguments:Vec<Value>) -> Result<Value, String> {
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

	Ok(Value::Null)
}
