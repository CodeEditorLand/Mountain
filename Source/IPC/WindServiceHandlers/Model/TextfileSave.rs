//! Save-intent hint from Wind. Actual disk write happens via `TextfileWrite`.
//! Returns an `IStat`-shaped object (mtime/size) so the workbench's
//! `TextFileEditorModel` can update its etag cache and clear the dirty dot
//! without a spurious "file changed on disk" conflict on the next read.

use std::sync::Arc;

use serde_json::Value;

use crate::{
	IPC::WindServiceHandlers::Utilities::{
		MetadataEncoding::Fn as MetadataToIStat,
		PathExtraction::Fn as ExtractPathFromArg,
	},
	RunTime::ApplicationRunTime::ApplicationRunTime,
	dev_log,
};

pub async fn Fn(_runtime:Arc<ApplicationRunTime>, Arguments:Vec<Value>) -> Result<Value, String> {
	let ResourceArg = Arguments.first().ok_or("textFile:save requires a resource argument")?;

	let Path = ExtractPathFromArg(ResourceArg).map_err(|E| format!("textFile:save bad resource: {}", E))?;

	dev_log!("vfs", "textFile:save path={:?}", Path);

	if Path.is_empty() {
		return Err("textFile:save: empty path after extraction".to_string());
	}

	match tokio::fs::metadata(&Path).await {
		Ok(Meta) => Ok(MetadataToIStat(&Meta)),

		// Propagate stat failure - returning Ok(Null) causes TextFileEditorModel
		// to call .mtime on null → TypeError, flipping the document to conflict
		// state even though the write succeeded.
		Err(E) => Err(format!("textFile:save post-stat failed for {}: {}", Path, E)),
	}
}
