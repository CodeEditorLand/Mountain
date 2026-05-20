#![allow(non_snake_case)]

//! Save-intent hint from Wind. Actual disk write happens via `TextfileWrite`.
//! Returns an `IStat`-shaped object (mtime/size) so the workbench's
//! `TextFileEditorModel` can update its etag cache and clear the dirty dot
//! without a spurious "file changed on disk" conflict on the next read.

use std::sync::Arc;

use serde_json::Value;

use crate::{
	IPC::WindServiceHandlers::Utilities::{MetadataEncoding::metadata_to_istat, PathExtraction::extract_path_from_arg},
	RunTime::ApplicationRunTime::ApplicationRunTime,
	dev_log,
};

pub async fn TextfileSave(_runtime:Arc<ApplicationRunTime>, Arguments:Vec<Value>) -> Result<Value, String> {
	let ResourceArg = Arguments.first().ok_or("textFile:save requires a resource argument")?;

	let Path = extract_path_from_arg(ResourceArg).unwrap_or_default();

	dev_log!("vfs", "textFile:save path={:?}", Path);

	if Path.is_empty() {
		return Ok(Value::Null);
	}

	match tokio::fs::metadata(&Path).await {
		Ok(Meta) => Ok(metadata_to_istat(&Meta)),

		Err(_) => Ok(Value::Null),
	}
}
