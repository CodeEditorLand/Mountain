#![allow(non_snake_case)]

//! Theme domain handlers for Wind IPC.

use std::sync::Arc;

use serde_json::{Value, json};

use crate::RunTime::ApplicationRunTime::ApplicationRunTime;

/// Return the active color theme metadata from ConfigurationProvider.
pub async fn handle_themes_get_active(Runtime:Arc<ApplicationRunTime>) -> Result<Value, String> {
	use CommonLibrary::Configuration::{
		ConfigurationProvider::ConfigurationProvider,
		DTO::ConfigurationOverridesDTO::ConfigurationOverridesDTO,
	};

	let ThemeId = Runtime
		.Environment
		.GetConfigurationValue(Some("workbench.colorTheme".to_string()), ConfigurationOverridesDTO::default())
		.await
		.map_err(|Error| format!("themes:getActive failed: {}", Error))?;

	let Id = ThemeId.as_str().unwrap_or("Default Dark Modern").to_string();

	let Kind = if Id.to_lowercase().contains("light") {
		"light"
	} else if Id.to_lowercase().contains("high contrast light") {
		"highContrastLight"
	} else if Id.to_lowercase().contains("high contrast") {
		"highContrast"
	} else {
		"dark"
	};

	Ok(json!({ "id": Id, "label": Id, "kind": Kind }))
}

/// Return installed theme extensions.
pub async fn handle_themes_list(Runtime:Arc<ApplicationRunTime>) -> Result<Value, String> {
	let Themes = vec![
		json!({ "id": "Default Dark Modern", "label": "Default Dark Modern", "kind": "dark" }),
		json!({ "id": "Default Light Modern", "label": "Default Light Modern", "kind": "light" }),
		json!({ "id": "Default Dark+", "label": "Default Dark+", "kind": "dark" }),
		json!({ "id": "Default Light+", "label": "Default Light+", "kind": "light" }),
		json!({ "id": "High Contrast", "label": "High Contrast", "kind": "highContrast" }),
		json!({ "id": "High Contrast Light", "label": "High Contrast Light", "kind": "highContrastLight" }),
	];

	Ok(json!(Themes))
}

/// Set the active color theme by updating ConfigurationProvider.
pub async fn handle_themes_set(Runtime:Arc<ApplicationRunTime>, Args:Vec<Value>) -> Result<Value, String> {
	use CommonLibrary::Configuration::{
		ConfigurationProvider::ConfigurationProvider,
		DTO::{ConfigurationOverridesDTO::ConfigurationOverridesDTO, ConfigurationTarget::ConfigurationTarget},
	};
	use tauri::Emitter;

	let ThemeId = Args
		.first()
		.and_then(|V| V.as_str())
		.ok_or("themes:set requires themeId as first argument".to_string())?
		.to_string();

	Runtime
		.Environment
		.UpdateConfigurationValue(
			"workbench.colorTheme".to_string(),
			json!(ThemeId),
			ConfigurationTarget::User,
			ConfigurationOverridesDTO::default(),
			None,
		)
		.await
		.map_err(|Error| format!("themes:set failed: {}", Error))?;

	let _ = Runtime
		.Environment
		.ApplicationHandle
		.emit("sky://theme/change", json!({ "themeId": ThemeId }));

	Ok(Value::Null)
}
