//! Lifecycle command dispatcher.

use serde_json::{Value, json};
use tauri::Manager;

use crate::IPC::WindServiceHandlers::{
	UI::{
		LifecycleGetPhase::Fn as LifecycleGetPhase,
		LifecycleRequestShutdown::Fn as LifecycleRequestShutdown,
		LifecycleWhenPhase::Fn as LifecycleWhenPhase,
	},
	Utilities::JsonValueHelpers::arg_u64_or,
};

use crate::UI::{
	LifecycleGetPhase::Fn as LifecycleGetPhase,
	LifecycleRequestShutdown::Fn as LifecycleRequestShutdown,
	LifecycleWhenPhase::Fn as LifecycleWhenPhase,
};

/// Dispatches lifecycle commands.
///
/// Handled commands:
/// - `lifecycle:getPhase`
/// - `lifecycle:whenPhase`
/// - `lifecycle:requestShutdown`
/// - `lifecycle:advancePhase`
/// - `lifecycle:setPhase`
pub async fn dispatch_lifecycle(
	app_handle:&tauri::AppHandle,

<<<<<<< HEAD
	runtime:&crate::RunTime::ApplicationRunTime::ApplicationRunTime,
=======
	runtime:std::sync::Arc<crate::RunTime::ApplicationRunTime::ApplicationRunTime>,
>>>>>>> 8e05e904fef6242d1b7fe4804dd9ac660dc91867

	command:&str,

	arguments:Vec<Value>,
) -> Result<Value, String> {
	match command {
		"lifecycle:getPhase" => LifecycleGetPhase(runtime.clone()).await,

		"lifecycle:whenPhase" => LifecycleWhenPhase(runtime.clone(), arguments).await,

		"lifecycle:requestShutdown" => LifecycleRequestShutdown(app_handle.clone()).await,

		"lifecycle:advancePhase" | "lifecycle:setPhase" => {
			crate::dev_log!("lifecycle", "{}", command);

<<<<<<< HEAD
			let new_phase = crate::Utilities::JsonValueHelpers::arg_u64_or(&arguments, 0, 1) as u8;
=======
			let new_phase = arg_u64_or(&arguments, 0, 1) as u8;
>>>>>>> 8e05e904fef6242d1b7fe4804dd9ac660dc91867

			runtime
				.Environment
				.ApplicationState
				.Feature
				.Lifecycle
				.AdvanceAndBroadcast(new_phase, app_handle);

			if new_phase >= 3 {
				if let Some(main_window) = app_handle.get_webview_window("main") {
					if let Ok(false) = main_window.is_visible() {
						if let Err(e) = main_window.show() {
							crate::dev_log!(
								"lifecycle",
								"warn: [Lifecycle] main window show() failed on phase {}: {}",
								new_phase,
								e
							);
						} else {
							crate::dev_log!(
								"lifecycle",
								"[Lifecycle] main window revealed on phase {} (hidden-until-ready)",
								new_phase
							);

							let _ = main_window.set_focus();
						}
					}
				}
			}

			Ok(json!(runtime.Environment.ApplicationState.Feature.Lifecycle.GetPhase()))
		},

		_ => Err(format!("Unknown lifecycle command: {}", command)),
	}
}
