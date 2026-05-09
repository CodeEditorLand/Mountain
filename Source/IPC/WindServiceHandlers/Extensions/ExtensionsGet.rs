#![allow(non_snake_case)]

//! `extensions:get(id)` - fetch a single extension's manifest
//! by `<publisher>.<name>` identifier. Returns `null` when the
//! id isn't in the scanner's registry; non-error outcome so
//! callers can `?? defaults` cleanly without an `unwrap`.

use std::sync::Arc;

use CommonLibrary::ExtensionManagement::ExtensionManagementService::ExtensionManagementService;

use serde_json::Value;

use crate::RunTime::ApplicationRunTime::ApplicationRunTime;

pub async fn ExtensionsGet(RunTime:Arc<ApplicationRunTime>, Arguments:Vec<Value>) -> Result<Value, String> {

	let Id = Arguments
		.first()
		.and_then(|V| V.as_str())
		.ok_or_else(|| "extensions:get requires string id as first argument".to_string())?
		.to_string();

	let Extension = RunTime
		.Environment
		.GetExtension(Id)
		.await
		.map_err(|Error| format!("extensions:get failed: {}", Error))?;

	Ok(Extension.unwrap_or(Value::Null))
}
