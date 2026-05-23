
//! `extensions:isActive(id) -> bool` - predicate over the
//! scanner's registry. Currently a "scanned & present" check;
//! a future revision should consult Cocoon's per-extension
//! activation table so the answer reflects whether the
//! extension's `activate()` function actually ran.

use std::sync::Arc;

use CommonLibrary::ExtensionManagement::ExtensionManagementService::ExtensionManagementService;
use serde_json::{Value, json};

use crate::RunTime::ApplicationRunTime::ApplicationRunTime;

pub async fn Fn(RunTime:Arc<ApplicationRunTime>, Arguments:Vec<Value>) -> Result<Value, String> {
	let Id = Arguments
		.first()
		.and_then(|V| V.as_str())
		.ok_or_else(|| "extensions:isActive requires string id as first argument".to_string())?
		.to_string();

	let Extension = RunTime
		.Environment
		.GetExtension(Id)
		.await
		.map_err(|Error| format!("extensions:isActive failed: {}", Error))?;

	Ok(json!(Extension.is_some()))
}
