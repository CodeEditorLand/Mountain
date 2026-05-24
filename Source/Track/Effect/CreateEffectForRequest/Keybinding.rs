use std::sync::Arc;

use CommonLibrary::{Environment::Requires::Requires, Keybinding::KeybindingProvider::KeybindingProvider};
use serde_json::Value;
use tauri::Runtime;

use crate::Track::Effect::MappedEffectType::MappedEffect;

pub fn Fn<R:Runtime>(MethodName:&str, Parameters:Value) -> Option<Result<MappedEffect, String>> {
	match MethodName {
		"Keybinding.GetResolved" => {
			crate::effect!(RunTime, {
				let Provider:Arc<dyn KeybindingProvider> = RunTime.Environment.Require();
				provider.GetResolvedKeybinding().await.map_err(|E| e.to_string())
			})
		},

		_ => None,
	}
}
