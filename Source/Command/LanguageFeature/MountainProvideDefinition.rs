//! Tauri command - go-to-definition. Delegates to
//! `LanguageFeature::Definition::provide_definition_impl`.

use serde_json::Value;

use tauri::{AppHandle, Wry, command};

use crate::{Command::LanguageFeature::Definition, dev_log};

#[command]
pub async fn MountainProvideDefinition(
	application_handle:AppHandle<Wry>,

	uri:String,

	position:Value,
) -> Result<Value, String> {

	dev_log!(
		"commands",

		"[Language Feature] Providing definition for: {} at {:?}",

		uri,

		position
	);

	Definition::provide_definition_impl(application_handle, uri, position).await
}
