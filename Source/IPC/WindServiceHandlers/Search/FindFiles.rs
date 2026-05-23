#![allow(unused_variables, dead_code, unused_imports)]

//! Wire method: `search:findFiles` / `search:fileSearch`.
//! Delegates to `WorkspaceProvider::FindFilesInWorkspace`.

use std::sync::Arc;

use serde_json::{Value, json};

use crate::{RunTime::ApplicationRunTime::ApplicationRunTime, dev_log};

pub async fn Fn(RunTime:Arc<ApplicationRunTime>, Arguments:Vec<Value>) -> Result<Value, String> {
	use CommonLibrary::Workspace::WorkspaceProvider::WorkspaceProvider;

	let IncludePattern = Arguments
		.first()
		.cloned()
		.ok_or_else(|| "search:findFiles requires include pattern in slot 0".to_string())?;

	let ExcludePattern = Arguments.get(1).cloned().filter(|V| !V.is_null());

	let MaxResults = Arguments.get(2).and_then(|V| V.as_u64()).map(|N| N as usize);

	let UseIgnoreFiles = Arguments.get(3).and_then(|V| V.as_bool()).unwrap_or(true);

	let FollowSymlinks = Arguments.get(4).and_then(|V| V.as_bool()).unwrap_or(false);

	dev_log!(
		"search",
		"search:fileSearch delegating to WorkspaceProvider::FindFilesInWorkspace (ignore={}, symlinks={})",
		UseIgnoreFiles,
		FollowSymlinks
	);

	let Urls = RunTime
		.Environment
		.FindFilesInWorkspace(IncludePattern, ExcludePattern, MaxResults, UseIgnoreFiles, FollowSymlinks)
		.await
		.map_err(|Error| Error.to_string())?;

	Ok(json!(Urls.into_iter().map(|U| U.to_string()).collect::<Vec<_>>()))
}
