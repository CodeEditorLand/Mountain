//! Save-intent hint from Wind. Actual disk write happens via `TextfileWrite`.
//! Returns an `IStat`-shaped object (mtime/size) so the workbench's
//! `TextFileEditorModel` can update its etag cache and clear the dirty dot
//! without a spurious "file changed on disk" conflict on the next read.

use std::sync::Arc;

use serde_json::{Value, json};

use crate::{
	IPC::WindServiceHandlers::Utilities::{
		MetadataEncoding::Fn as metadata_to_istat,
		PathExtraction::Fn as extract_path_from_arg,
	},
	RunTime::ApplicationRunTime::ApplicationRunTime,
	dev_log,
};

pub async fn Fn(_runtime:Arc<ApplicationRunTime>, Arguments:Vec<Value>) -> Result<Value, String> {
	let ResourceArg = Arguments.first().ok_or("textFile:save requires a resource argument")?;

	let Uri = ResourceArg.to_string();

	let Path = extract_path_from_arg(ResourceArg).map_err(|E| format!("textFile:save bad resource: {}", E))?;

	dev_log!("vfs", "textFile:save path={:?}", Path);

	if Path.is_empty() {
		return Err("textFile:save: empty path after extraction".to_string());
	}

	match tokio::fs::metadata(&Path).await {
		Ok(Meta) => {
			// T1.4 - notify Cocoon that the model on disk matches the editor
			// buffer, firing `onDidSaveTextDocument` for extensions.
			let _ = crate::Vine::Client::SendNotification::Fn(
				"cocoon-main".to_string(),
				"$acceptModelSaved".to_string(),
				json!({ "uri": Uri }),
			);

			Ok(metadata_to_istat(&Meta))
		},

		// Propagate stat failure - returning Ok(Null) causes TextFileEditorModel
		// to call .mtime on null → TypeError, flipping the document to conflict
		// state even though the write succeeded.
		Err(E) => Err(format!("textFile:save post-stat failed for {}: {}", Path, E)),
	}
}
