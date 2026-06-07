//! Tauri command - find all references to a symbol. Delegates to
//! `LanguageFeature::References::provide_references_impl`.

use serde_json::Value;

use tauri::{AppHandle, Wry, command};

use crate::{Command::LanguageFeature::References, dev_log};

#[command]
pub async fn MountainProvideReferences(
	application_handle:AppHandle<Wry>,

	uri:String,

	position:Value,

	context:Value,
) -> Result<Value, String> {

	dev_log!(
		"commands",

		"[Language Feature] Providing references for: {} at {:?}",

		uri,

		position
	);

	References::provide_references_impl(application_handle, uri, position, context).await
}
