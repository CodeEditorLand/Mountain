//! Mutable lifecycle state for the Cocoon child process. One instance lives
//! behind the module-level `COCOON_STATE` mutex and is shared by the spawn,
//! health-monitor, restart, and shutdown paths.

use tokio::process::Child;

/// Global state for tracking Cocoon process lifecycle
pub(crate) struct CocoonProcessState {
	pub(crate) ChildProcess:Option<Child>,

	pub(crate) IsRunning:bool,

	pub(crate) StartTime:Option<tokio::time::Instant>,

	pub(crate) RestartCount:u32,

	pub(crate) LastRestartTime:Option<tokio::time::Instant>,

	/// Channel used by the health monitor to schedule an automatic restart.
	/// Each send carries the backoff duration in seconds.
	pub(crate) RestartTx:Option<tokio::sync::mpsc::Sender<u64>>,
}

impl Default for CocoonProcessState {
	fn default() -> Self {
		Self {
			ChildProcess:None,

			IsRunning:false,

			StartTime:None,

			RestartCount:0,

			LastRestartTime:None,

			RestartTx:None,
		}
	}
}
