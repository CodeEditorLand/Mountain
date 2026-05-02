#![allow(non_snake_case)]

//! Tauri command - code completion suggestions. Delegates to
//! `LanguageFeature::Completions::provide_completions_impl`.

use serde_json::Value;
use tauri::{AppHandle, Wry, command};

use crate::{Command::LanguageFeature::Completions, dev_log};

#[command]
pub async fn MountainProvideCompletions(
	application_handle:AppHandle<Wry>,
	uri:String,
	position:Value,
	context:Value,
) -> Result<Value, String> {
	dev_log!(
		"commands",
		"[Language Feature] Providing completions for: {} at {:?}",
		uri,
		position
	);
	Completions::provide_completions_impl(application_handle, uri, position, context).await
}
