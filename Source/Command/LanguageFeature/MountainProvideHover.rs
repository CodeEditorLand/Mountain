#![allow(non_snake_case)]

//! Tauri command - show hover information at the cursor position.
//! Delegates to `LanguageFeature::Hover::provide_hover_impl`.

use serde_json::Value;
use tauri::{AppHandle, Wry, command};

use crate::{Command::LanguageFeature::Hover, dev_log};

#[command]
pub async fn MountainProvideHover(
	application_handle:AppHandle<Wry>,
	uri:String,
	position:Value,
) -> Result<Value, String> {
	dev_log!("commands", "[Language Feature] Providing hover for: {} at {:?}", uri, position);
	Hover::provide_hover_impl(application_handle, uri, position).await
}
