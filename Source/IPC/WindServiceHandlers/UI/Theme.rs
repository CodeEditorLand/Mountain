#![allow(non_snake_case, unused_variables)]
//! Theme IPC handlers. `themes:getActive` / `themes:list` / `themes:set`
//! drive the workbench's colour-theme picker and the runtime theme swap.
//!
//! Source of truth: `workbench.colorTheme` inside `ConfigurationProvider`.
//! A `themes:set` writes the key and emits `SkyEvent::ThemeChange` so
//! Monaco and the Sky shell re-tint in-place without a window reload.

use std::sync::Arc;

use CommonLibrary::IPC::SkyEvent::SkyEvent;
use serde_json::{Value, json};

use crate::RunTime::ApplicationRunTime::ApplicationRunTime;

pub async fn handle_themes_get_active(runtime:Arc<ApplicationRunTime>) -> Result<Value, String> {
	use CommonLibrary::Configuration::{
		ConfigurationProvider::ConfigurationProvider,
		DTO::ConfigurationOverridesDTO::ConfigurationOverridesDTO,
	};

	let ThemeId = runtime
		.Environment
		.GetConfigurationValue(Some("workbench.colorTheme".to_string()), ConfigurationOverridesDTO::default())
		.await
		.map_err(|Error| format!("themes:getActive failed: {}", Error))?;

	let Id = ThemeId.as_str().unwrap_or("Default Dark Modern").to_string();

	// Infer kind from id string.
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

pub async fn handle_themes_list(_runtime:Arc<ApplicationRunTime>) -> Result<Value, String> {
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

pub async fn handle_themes_set(runtime:Arc<ApplicationRunTime>, args:Vec<Value>) -> Result<Value, String> {
	use CommonLibrary::Configuration::{
		ConfigurationProvider::ConfigurationProvider,
		DTO::{ConfigurationOverridesDTO::ConfigurationOverridesDTO, ConfigurationTarget::ConfigurationTarget},
	};
	use tauri::Emitter;

	let ThemeId = args
		.first()
		.and_then(|V| V.as_str())
		.ok_or("themes:set requires themeId as first argument".to_string())?
		.to_string();

	runtime
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

	let _ = runtime
		.Environment
		.ApplicationHandle
		.emit(SkyEvent::ThemeChange.AsStr(), json!({ "themeId": ThemeId }));

	Ok(Value::Null)
}
