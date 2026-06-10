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

	// Dual-emit to Cocoon so `vscode.window.onDidChangeActiveColorTheme`
	// subscribers fire inside the Node extension host. Without this, the
	// `sky://theme/change` Tauri emit above only reaches the renderer;
	// Node-resident extensions (GitLens, Roo, rust-analyzer) that adapt
	// their UI based on theme.kind never see the change. Cocoon's
	// `Services/Handler/Notification/Handler.ts` maps
	// `$acceptActiveColorTheme` → `Emitter.emit("window.didChangeActiveColorTheme",
	// ...)` which `Window/Namespace.ts:1041` subscribers attach to via
	// `MakeEventSubscriber`. Payload includes the workbench's theme kind
	// (1=Light, 2=Dark, 3=HighContrast, 4=HighContrastLight) when
	// discoverable from the theme id, otherwise just the id.
	let ThemeKind = if ThemeId.to_ascii_lowercase().contains("light") {
		if ThemeId.to_ascii_lowercase().contains("high") { 4i32 } else { 1i32 }
	} else if ThemeId.to_ascii_lowercase().contains("high") {
		3i32
	} else {
		2i32
	};

	let _ = crate::Vine::Client::SendNotification::Fn(
		"cocoon-main".to_string(),
		"$acceptActiveColorTheme".to_string(),
		json!({ "id": ThemeId, "kind": ThemeKind }),
	)
	.await;

	Ok(Value::Null)
}
