use std::sync::Arc;

use parking_lot::Mutex;
use tokio::sync::Notify;
use CommonLibrary::IPC::SkyEvent::SkyEvent;

use crate::{IPC::SkyEmit::LogSkyEmit, dev_log};

/// Application lifecycle phases (mirrors VS Code LifecyclePhase).
/// 1 = Starting, 2 = Ready, 3 = Restored, 4 = Eventually
pub type Phase = u8;

/// Tracks the current application lifecycle phase.
/// `PhaseNotify` fires every time the phase advances so `LifecycleWhenPhase`
/// can await it instead of polling at 100 ms intervals.
#[derive(Clone)]
pub struct LifecyclePhaseState {
	CurrentPhase:Arc<Mutex<Phase>>,

	/// Fired (notify_waiters) on every forward phase transition.
	pub PhaseNotify:Arc<Notify>,
}

impl Default for LifecyclePhaseState {
	fn default() -> Self {
		dev_log!(
			"lifecycle",
			"[LifecyclePhaseState] Initializing default lifecycle state (phase 1: Starting)..."
		);

		Self { CurrentPhase:Arc::new(Mutex::new(1)), PhaseNotify:Arc::new(Notify::new()) }
	}
}

impl LifecyclePhaseState {
	/// Return the current lifecycle phase.
	pub fn GetPhase(&self) -> Phase { *self.CurrentPhase.lock() }

	/// Advance the lifecycle phase. Only advances forward - never backwards.
	pub fn SetPhase(&self, NewPhase:Phase) {
		let mut Guard = self.CurrentPhase.lock();

		if NewPhase > *Guard {
			dev_log!("lifecycle", "[LifecyclePhaseState] Phase advanced: {} → {}", *Guard, NewPhase);

			*Guard = NewPhase;

			// Wake all `LifecycleWhenPhase` waiters immediately.
			self.PhaseNotify.notify_waiters();
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

		match LogSkyEmit(
			ApplicationHandle,
			SkyEvent::LifecyclePhaseChanged.AsStr(),
			serde_json::json!({
				"phase": NewPhase,
				"previous": Previous,
				"label": Label,
			}),
		) {
			Ok(()) => {},

			Err(Error) => {
				dev_log!(
					"lifecycle",
					"warn: [LifecyclePhaseState] sky://lifecycle/phaseChanged emit failed: {}",
					Error
				)
			},
		}
	}
}
