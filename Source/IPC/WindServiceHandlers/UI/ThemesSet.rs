
//! Wire method: `themes:set`.

use std::sync::Arc;

use serde_json::{Value, json};

use crate::RunTime::ApplicationRunTime::ApplicationRunTime;

pub async fn Fn(RunTime:Arc<ApplicationRunTime>, Arguments:Vec<Value>) -> Result<Value, String> {
	use CommonLibrary::{
		Configuration::{
			ConfigurationProvider::ConfigurationProvider,
			DTO::{ConfigurationOverridesDTO::ConfigurationOverridesDTO, ConfigurationTarget::ConfigurationTarget},
		},
		IPC::SkyEvent::SkyEvent,
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
