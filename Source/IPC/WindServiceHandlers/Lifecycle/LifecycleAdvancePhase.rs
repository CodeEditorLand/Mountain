//! Wire method: `lifecycle:advancePhase` / `lifecycle:setPhase`.
//!
//! Wind calls this at the end of every workbench init pass so the phase
//! advances Starting → Ready → Restored → Eventually. Mountain emits
//! `sky://lifecycle/phaseChanged` so any extension host or service waiting on
//! a later phase wakes up.
//!
//! Hidden-until-ready: the main window is built with `.visible(false)` to
//! suppress the four-repaint flash (native chrome → inline bg → theme CSS →
//! workbench DOM). Phase 3 = Restored means `.monaco-workbench` is attached
//! and the first frame is painted; show the window now so the user's first
//! glimpse is the finished editor rather than the paint cascade.
//!
//! `set_focus()` follows `show()` so keyboard input routes to the editor
//! immediately on reveal. Failures are logged but swallowed — if the window
//! is already visible (phase 3 re-fired from another consumer) Tauri returns
//! a benign error.

use std::sync::Arc;

use serde_json::{Value, json};
use tauri::Manager;

use crate::{
	IPC::WindServiceHandlers::Utilities::JsonValueHelpers::arg_u64_or,
	RunTime::ApplicationRunTime::ApplicationRunTime,
	dev_log,
};

pub async fn Fn(
	RunTime:Arc<ApplicationRunTime>,

	ApplicationHandle:tauri::AppHandle,

	Arguments:Vec<Value>,
) -> Result<Value, String> {
	let NewPhase = arg_u64_or(&Arguments, 0, 1) as u8;

	RunTime
		.Environment
		.ApplicationState
		.Feature
		.Lifecycle
		.AdvanceAndBroadcast(NewPhase, &ApplicationHandle);

	if NewPhase >= 3 {
		if let Some(MainWindow) = ApplicationHandle.get_webview_window("main") {
			if let Ok(false) = MainWindow.is_visible() {
				if let Err(Error) = MainWindow.show() {
					dev_log!(
						"lifecycle",
						"warn: [Lifecycle] main window show() failed on phase {}: {}",
						NewPhase,
						Error
					);
				} else {
					dev_log!(
						"lifecycle",
						"[Lifecycle] main window revealed on phase {} (hidden-until-ready)",
						NewPhase
					);

					let _ = MainWindow.set_focus();
				}
			}
		}
	}

	Ok(json!(RunTime.Environment.ApplicationState.Feature.Lifecycle.GetPhase()))
}
