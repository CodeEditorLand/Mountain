#![allow(non_snake_case)]

//! Extension host domain handlers for Wind IPC.

use std::sync::Arc;

use serde_json::{Value, json};

use CommonLibrary::ExtensionManagement::ExtensionManagementService::ExtensionManagementService;

use crate::{
	RunTime::ApplicationRunTime::ApplicationRunTime,
	dev_log,
};

/// Return metadata for all scanned extensions.
pub async fn handle_extensions_get_all(Runtime:Arc<ApplicationRunTime>) -> Result<Value, String> {
	let Extensions = Runtime
		.Environment
		.GetExtensions()
		.await
		.map_err(|Error| format!("extensions:getAll failed: {}", Error))?;

	dev_log!("extensions", "extensions:getAll returning {} extensions", Extensions.len());
	if let Some(First) = Extensions.first() {
		dev_log!(
			"extensions",
			"extensions:getAll sample: {}",
			serde_json::to_string(First)
				.unwrap_or_default()
				.chars()
				.take(300)
				.collect::<String>()
		);
	}
	Ok(json!(Extensions))
}

/// Return metadata for a single extension by ID.
pub async fn handle_extensions_get(Runtime:Arc<ApplicationRunTime>, Args:Vec<Value>) -> Result<Value, String> {
	let Id = Args
		.first()
		.and_then(|V| V.as_str())
		.ok_or_else(|| "extensions:get requires string id as first argument".to_string())?
		.to_string();

	let Extension = Runtime
		.Environment
		.GetExtension(Id)
		.await
		.map_err(|Error| format!("extensions:get failed: {}", Error))?;

	Ok(Extension.unwrap_or(Value::Null))
}

/// Check whether an extension is currently active (scanned and present).
pub async fn handle_extensions_is_active(Runtime:Arc<ApplicationRunTime>, Args:Vec<Value>) -> Result<Value, String> {
	let Id = Args
		.first()
		.and_then(|V| V.as_str())
		.ok_or_else(|| "extensions:isActive requires string id as first argument".to_string())?
		.to_string();

	let Extension = Runtime
		.Environment
		.GetExtension(Id)
		.await
		.map_err(|Error| format!("extensions:isActive failed: {}", Error))?;

	Ok(json!(Extension.is_some()))
}
