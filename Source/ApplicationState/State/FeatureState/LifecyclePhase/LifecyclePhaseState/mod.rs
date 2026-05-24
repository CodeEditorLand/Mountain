pub mod GetPhase;
pub mod SetPhase;
pub mod AdvanceAndBroadcast;

use std::sync::{Arc, Mutex as StandardMutex};
use tokio::sync::Notify;
use CommonLibrary::IPC::SkyEvent::SkyEvent;
use crate::{IPC::SkyEmit::Fn, dev_log};

/// Application lifecycle phases (mirrors VS Code LifecyclePhase).
/// 1 = Starting, 2 = Ready, 3 = Restored, 4 = Eventually
pub type Phase = u8;

/// Tracks the current application lifecycle phase.
/// `PhaseNotify` fires every time the phase advances so `LifecycleWhenPhase`
/// can await it instead of polling at 100 ms intervals.
#[derive(Clone)]
pub struct Struct {
	CurrentPhase:Arc<StandardMutex<Phase>>,

	/// Fired (notify_waiters) on every forward phase transition.
	pub PhaseNotify:Arc<Notify>,
}
