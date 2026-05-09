use std::sync::{Arc, Mutex as StandardMutex};

use CommonLibrary::IPC::SkyEvent::SkyEvent;

use crate::{IPC::SkyEmit::LogSkyEmit, dev_log};

/// Application lifecycle phases (mirrors VS Code LifecyclePhase).
/// 1 = Starting, 2 = Ready, 3 = Restored, 4 = Eventually
pub type Phase = u8;

/// Tracks the current application lifecycle phase.
/// Components poll this to defer work until the editor is fully initialised.
#[derive(Clone)]
pub struct LifecyclePhaseState {

	CurrentPhase:Arc<StandardMutex<Phase>>,
}

impl Default for LifecyclePhaseState {

	fn default() -> Self {

		dev_log!(
			"lifecycle",

			"[LifecyclePhaseState] Initializing default lifecycle state (phase 1: Starting)..."
		);

		Self { CurrentPhase:Arc::new(StandardMutex::new(1)) }
	}
}

impl LifecyclePhaseState {

	/// Return the current lifecycle phase.
	pub fn GetPhase(&self) -> Phase { self.CurrentPhase.lock().ok().map(|Guard| *Guard).unwrap_or(1) }

	/// Advance the lifecycle phase. Only advances forward - never backwards.
	pub fn SetPhase(&self, NewPhase:Phase) {

		if let Ok(mut Guard) = self.CurrentPhase.lock() {

			if NewPhase > *Guard {

				dev_log!("lifecycle", "[LifecyclePhaseState] Phase advanced: {} → {}", *Guard, NewPhase);

				*Guard = NewPhase;
			}
		}
	}

	/// Advance the phase and emit a `sky://lifecycle/phaseChanged` Tauri
	/// event so the workbench (subscribed via
	/// `TauriChannel("lifecycle").listen("onDidChangePhase")`) can gate
	/// long-running services (extension discovery, telemetry, heavy
	/// providers) until the editor is fully restored. Mirrors VS Code's
	/// `ILifecycleService.onDidChangePhase` signal.
	pub fn AdvanceAndBroadcast<R:tauri::Runtime>(&self, NewPhase:Phase, ApplicationHandle:&tauri::AppHandle<R>) {

		// Local `use tauri::Emitter` removed - now routed through
		// `LogSkyEmit` which carries the trait import internally.
		let Previous = self.GetPhase();

		if NewPhase <= Previous {

			return;
		}

		self.SetPhase(NewPhase);

		let Label = match NewPhase {

			1 => "Starting",

			2 => "Ready",

			3 => "Restored",

			4 => "Eventually",

			_ => "Unknown",
		};

		if let Err(Error) = LogSkyEmit(
			ApplicationHandle,

			SkyEvent::LifecyclePhaseChanged.AsStr(),

			serde_json::json!({
				"phase": NewPhase,
				"previous": Previous,
				"label": Label,
			}),
		) {

			dev_log!(
				"lifecycle",

				"warn: [LifecyclePhaseState] sky://lifecycle/phaseChanged emit failed: {}",

				Error
			);
		}
	}
}
