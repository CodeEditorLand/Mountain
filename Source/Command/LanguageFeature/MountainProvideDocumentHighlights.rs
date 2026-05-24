//! Tauri command - symbol occurrences in a document. Delegates to
//! `LanguageFeature::Highlights::provide_document_highlights_impl`.

use serde_json::Value;
use tauri::{AppHandle, Wry, command};

use crate::{Command::LanguageFeature::Highlights, dev_log};

#[command]
pub async fn Fn(
	application_handle:AppHandle<Wry>,

	uri:String,

	position:Value,
) -> Result<Value, String> {
	dev_log!(
		"commands",
		"[Language Feature] Providing document highlights for: {} at {:?}",
		uri,
		position
	);

	Highlights::provide_document_highlights_impl(application_handle, uri, position).await
}
