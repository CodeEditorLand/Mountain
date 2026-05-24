//! Bootstrap helper - construct `Features::Struct`, stash a
//! clone in Tauri state, spawn the monitor tasks. Called from
//! `Binary/Register/AdvancedFeaturesRegister.rs`.

use std::sync::Arc;

use tauri::Manager;

use crate::{
	IPC::AdvancedFeatures::Features::Struct as Features,
	RunTime::ApplicationRunTime::ApplicationRunTime,
	dev_log,
};

pub fn Fn(app_handle:&tauri::AppHandle, runtime:Arc<ApplicationRunTime>) -> Result<(), String> {
	dev_log!("lifecycle", "Initializing advanced IPC features");

	let features = Features::new(runtime);

	app_handle.manage(features.clone());

	let features_clone = features.clone();

	tokio::spawn(async move {
		if let Err(e) = features_clone.StartMonitoring().await {
			dev_log!("ipc", "error: [AdvancedFeatures] Failed to start monitoring: {}", e);
		}
	});

	Ok(())
}
