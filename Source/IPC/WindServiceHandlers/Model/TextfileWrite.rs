//! Write text to a file on disk. Counterpart to `TextfileRead`;
//! does not touch the document registry. After a successful disk
//! write, fires `$acceptModelSaved` to Cocoon so extensions
//! receive `onDidSaveTextDocument` (T1.4).

use std::sync::Arc;

use serde_json::{Value, json};

use crate::{RunTime::ApplicationRunTime::ApplicationRunTime, dev_log};

pub async fn Fn(_runtime:Arc<ApplicationRunTime>, Arguments:Vec<Value>) -> Result<Value, String> {
	let Path = Arguments
		.first()
		.and_then(|V| V.as_str())
		.ok_or_else(|| "textFile:write requires path as first argument".to_string())?
		.to_string();

	let Content = Arguments.get(1).and_then(|V| V.as_str()).unwrap_or("").to_string();

	tokio::fs::write(&Path, Content.as_bytes())
		.await
		.map_err(|Error| format!("textFile:write failed: {}", Error))?;

	dev_log!("vfs", "textFile:write ok path={} bytes={}", Path, Content.len());

	// T1.4 - notify Cocoon that the model on disk now matches the editor
	// buffer, firing `onDidSaveTextDocument` for all subscribed extensions
	// (format-on-save, organize-imports, save listeners, etc.).
	// Fire-and-forget: the write is already complete; a Vine failure here
	// must not cause the save IPC call to fail from the workbench's
	// perspective.
	let FileUri = format!("file://{}", Path);

	tokio::spawn(async move {
		if let Err(Error) = crate::Vine::Client::SendNotification::Fn(
			"cocoon-main".to_string(),
			"$acceptModelSaved".to_string(),
			json!({ "uri": FileUri }),
		)
		.await
		{
			dev_log!("vfs", "warn: [TextfileWrite] $acceptModelSaved notify failed: {:?}", Error);
		}
	});

	Ok(Value::Null)
}
