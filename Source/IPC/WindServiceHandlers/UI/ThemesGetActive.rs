#![allow(unused_variables)]

//! Wire method: `themes:getColorTheme`.

use std::sync::Arc;

use serde_json::{Value, json};

use crate::RunTime::ApplicationRunTime::ApplicationRunTime;

pub async fn Fn(RunTime:Arc<ApplicationRunTime>) -> Result<Value, String> {
	use CommonLibrary::Configuration::{
		ConfigurationProvider::ConfigurationProvider,
		DTO::ConfigurationOverridesDTO::ConfigurationOverridesDTO,
	};

	let ThemeId = RunTime
		.Environment
		.GetConfigurationValue(Some("workbench.colorTheme".to_string()), ConfigurationOverridesDTO::default())
		.await
		.map_err(|Error| format!("themes:getActive failed: {}", Error))?;

	let Id = ThemeId.as_str().unwrap_or("Default Dark Modern").to_string();

	let (Kind, TypeNum) = if Id.to_lowercase().contains("high contrast light") {
		("highContrastLight", 4u8)
	} else if Id.to_lowercase().contains("high contrast") {
		("highContrast", 3u8)
	} else if Id.to_lowercase().contains("light") {
		("light", 1u8)
	} else {
		("dark", 2u8)
	};

	Ok(json!({
		"id": Id,
		"label": Id,
		"kind": Kind,
		"type": TypeNum,
		"semanticHighlighting": false,
	}))
}
