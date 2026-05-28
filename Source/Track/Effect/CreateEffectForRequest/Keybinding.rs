use std::sync::Arc;

use CommonLibrary::{Environment::Requires::Requires, Keybinding::KeybindingProvider::KeybindingProvider};
use serde_json::Value;
use tauri::Runtime;

use crate::Track::Effect::MappedEffectType::MappedEffect;

pub fn CreateEffect<R:Runtime>(MethodName:&str, Parameters:Value) -> Option<Result<MappedEffect, String>> {
	match MethodName {
		"Keybinding.GetResolved" => {
			crate::effect!(run_time, {
				let provider:Arc<dyn KeybindingProvider> = run_time.Environment.Require();

				provider.GetResolvedKeybinding().await.map_err(|e| e.to_string())
			})
		},

		_ => None,
	}
}
