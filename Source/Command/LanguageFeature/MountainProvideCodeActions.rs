//! Tauri command - quick fixes and refactorings for a code range.
//! Delegates to `LanguageFeature::CodeActions::provide_code_actions_impl`.

use serde_json::Value;

use tauri::{AppHandle, Wry, command};

use crate::{Command::LanguageFeature::CodeActions, dev_log};

#[command]
pub async fn MountainProvideCodeActions(
	application_handle:AppHandle<Wry>,

	uri:String,

	position:Value,

	context:Value,
) -> Result<Value, String> {

	dev_log!(
		"commands",

		"[Language Feature] Providing code actions for: {} at {:?}",

		uri,

		position
	);

	CodeActions::provide_code_actions_impl(application_handle, uri, position, context).await
}
