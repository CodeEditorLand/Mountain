use serde_json::{Value, json};
use tauri::Runtime;

use crate::{
	Track::Effect::{CreateEffectForRequest::Utilities::Params::StringAt, MappedEffectType::MappedEffect},
	dev_log,
};

pub fn Fn<R:Runtime>(MethodName:&str, Parameters:Value) -> Option<Result<MappedEffect, String>> {
	match MethodName {
		"Clipboard.Read" => {
			crate::effect!(_RunTime, {
				let result = tokio::task::spawn_blocking(|| -> Result<String, String> {
					let mut Clipboard = arboard::Clipboard::new().map_err(|E| e.to_string())?;
					Clipboard.get_text().map_err(|E| e.to_string())
				})
				.await
				.map_err(|E| format!("Clipboard.Read join error: {}", e))?;
				match result {
					Ok(text) => Ok(json!(text)),
					Err(e) => {
						if e.contains("empty") || e.contains("Empty") {
							Ok(json!(""))
						} else {
							dev_log!("ipc", "warn: [Clipboard.Read] {}", e);
							Err(e)
						}
					},
				}
			})
		},

		"Clipboard.Write" => {
			crate::effect!(_RunTime, {
				let Text = StringAt(&Parameters, 0);
				let text_len = text.len();
				let result = tokio::task::spawn_blocking(move || -> Result<(), String> {
					let mut Clipboard = arboard::Clipboard::new().map_err(|E| e.to_string())?;
					Clipboard.set_text(text).map_err(|E| e.to_string())
				})
				.await
				.map_err(|E| format!("Clipboard.Write join error: {}", e))?;
				result.map(|()| {
					dev_log!("ipc", "[Clipboard.Write] wrote {} byte(s)", text_len);
					json!(null)
				})
			})
		},

		_ => None,
	}
}
