#![allow(non_snake_case, unused_variables)]
//! Theme IPC handlers. `themes:getActive` / `themes:list` / `themes:set`
//! drive the workbench's colour-theme picker and the RunTime theme swap.
//!
//! Source of truth: `workbench.colorTheme` inside `ConfigurationProvider`.
//! A `themes:set` writes the key and emits `SkyEvent::ThemeChange` so
//! Monaco and the Sky shell re-tint in-place without a window reload.

use std::sync::Arc;

use CommonLibrary::IPC::SkyEvent::SkyEvent;
use serde_json::{Value, json};

use crate::RunTime::ApplicationRunTime::ApplicationRunTime;

pub async fn ThemesGetActive(RunTime:Arc<ApplicationRunTime>) -> Result<Value, String> {
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

	// Infer kind from id string.
	// `ColorThemeKind` numeric values from VS Code:
	//   Light = 1, Dark = 2, HighContrast = 3, HighContrastLight = 4
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
		// `kind` is the string variant used by Land's own UI layer.
		"kind": Kind,
		// `type` is the numeric `ColorThemeKind` enum that VS Code's
		// workbench (`ThemeService`, `TokenizationRegistry`) reads to decide
		// syntax highlighting colour sets.
		"type": TypeNum,
		// Minimal tokenization / color fields; workbench falls back to
		// built-in defaults for missing entries.
		"semanticHighlighting": false,
	}))
}

pub async fn ThemesList(_runtime:Arc<ApplicationRunTime>) -> Result<Value, String> {
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

pub async fn ThemesSet(RunTime:Arc<ApplicationRunTime>, Arguments:Vec<Value>) -> Result<Value, String> {
	use CommonLibrary::Configuration::{
		ConfigurationProvider::ConfigurationProvider,
		DTO::{ConfigurationOverridesDTO::ConfigurationOverridesDTO, ConfigurationTarget::ConfigurationTarget},
	};
	use tauri::Emitter;

	let ThemeId = Arguments
		.first()
		.and_then(|V| V.as_str())
		.ok_or("themes:set requires themeId as first argument".to_string())?
		.to_string();

	RunTime
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

	let _ = RunTime
		.Environment
		.ApplicationHandle
		.emit(SkyEvent::ThemeChange.AsStr(), json!({ "themeId": ThemeId }));

	Ok(Value::Null)
}
