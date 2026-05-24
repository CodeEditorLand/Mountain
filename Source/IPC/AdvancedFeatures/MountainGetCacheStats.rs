//! `MountainGetCacheStats` Tauri command - returns the
//! `MessageCache::Struct` snapshot (entries + hit / miss
//! counters).

use tauri::Manager;

use crate::{
	IPC::AdvancedFeatures::{Features::Struct as Features, MessageCache::Struct as MessageCache},
	dev_log,
};

#[tauri::command]
pub async fn Fn(app_handle:tauri::AppHandle) -> Result<MessageCache, String> {
	dev_log!("lifecycle", "Tauri command: get_cache_stats");

	if let Some(features) = app_handle.try_state::<Features>() {
		features.GetCacheStats().await
	} else {
		Err("AdvancedFeatures not found in application state".to_string())
	}
}
