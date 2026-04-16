use std::sync::{Arc, Mutex as StandardMutex};
use crate::dev_log;


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
		dev_log!("lifecycle", "[LifecyclePhaseState] Initializing default lifecycle state (phase 1: Starting)...");
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
}
