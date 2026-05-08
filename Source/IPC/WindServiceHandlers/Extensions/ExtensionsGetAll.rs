#![allow(non_snake_case)]

//! `extensions:getAll` - every scanned extension's raw
//! manifest, no `ILocalExtension` reshape. Used by tooling /
//! debug surfaces that want the full payload (extension
//! activity log, marketplace UI, audit reports). Renderer
//! consumers MUST go through `ExtensionsGetInstalled` for the
//! workbench-shaped data.

use std::sync::Arc;

use CommonLibrary::ExtensionManagement::ExtensionManagementService::ExtensionManagementService;
use serde_json::{Value, json};

use crate::{RunTime::ApplicationRunTime::ApplicationRunTime, dev_log};

pub async fn ExtensionsGetAll(RunTime:Arc<ApplicationRunTime>) -> Result<Value, String> {
	let Extensions = RunTime
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
