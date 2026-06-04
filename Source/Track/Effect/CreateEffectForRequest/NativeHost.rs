pub fn Matches(MethodName:&str) -> bool {
	match MethodName {
		NativeHost.OpenExternal => true,
		_ => false,
	}
}

use serde_json::{Value, json};
use tauri::Runtime;

use crate::{
	Track::Effect::{CreateEffectForRequest::Utilities::Params::string_at, MappedEffectType::MappedEffect},
	dev_log,
};
pub fn CreateEffect<R:Runtime>(MethodName:&str, Parameters:&Value) -> Option<Result<MappedEffect, String>> {
	match MethodName {
		"NativeHost.OpenExternal" => {
			crate::effect!(_run_time, {
				let uri = string_at(&Parameters, 0);

				let lower = uri.to_ascii_lowercase();

				const BlockedSchemes:&[&str] = &["javascript:", "data:", "vbscript:", "file:"];

				for scheme in BlockedSchemes {
					if lower.starts_with(scheme) {
						dev_log!("ipc", "warn: [NativeHost.OpenExternal] rejected scheme '{}': {}", scheme, uri);

						return Err(format!("NativeHost.OpenExternal: scheme '{}' is not allowed", scheme));
					}
				}

				if uri.is_empty() {
					return Err("NativeHost.OpenExternal: empty URI".to_string());
				}

				let uri_owned = uri.clone();

				let result = tokio::task::spawn_blocking(move || open::that_detached(uri_owned))
					.await
					.map_err(|e| format!("NativeHost.OpenExternal join error: {}", e))?;

				match result {
					Ok(()) => {
						dev_log!("ipc", "[NativeHost.OpenExternal] opened {}", uri);

						Ok(json!(true))
					},
					Err(e) => {
						dev_log!("ipc", "warn: [NativeHost.OpenExternal] failed uri={} error={}", uri, e);

						Err(e.to_string())
					},
				}
			})
		},

		_ => None,
	}
}
