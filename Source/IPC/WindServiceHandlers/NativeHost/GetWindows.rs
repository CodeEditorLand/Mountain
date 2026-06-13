//! Wire method: `nativeHost:getWindows`.
//!
//! Returns the single-window list with active-document metadata so the
//! workbench/extension host sees the expected `IWindowsMainService` shape.

use std::sync::Arc;

use serde_json::{Value, json};

use crate::RunTime::ApplicationRunTime::ApplicationRunTime;

pub async fn Fn(RunTime:Arc<ApplicationRunTime>) -> Result<Value, String> {
	let Title = std::env::var("ProductNameShort").unwrap_or_else(|_| "Land".into());

	let ActiveDoc = RunTime
		.Environment
		.ApplicationState
		.Workspace
		.GetActiveDocumentURI()
		.unwrap_or_default();

	Ok(json!([{ "id": 1, "title": Title, "filename": ActiveDoc }]))
}
