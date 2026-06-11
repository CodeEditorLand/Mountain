//! Force-terminate the stored Cocoon child after the graceful `$shutdown`
//! attempt, then reset the process state. No-op when no child is stored.

use crate::dev_log;

pub(crate) async fn Fn() {
	let mut State = super::COCOON_STATE.lock().await;

	if let Some(mut Child) = State.ChildProcess.take() {
		let Pid = Child.id().unwrap_or(0);

		match Child.try_wait() {
			Ok(Some(_Status)) => {
				dev_log!("cocoon", "[CocoonShutdown] Child PID {} already exited; clearing handle.", Pid);
			},

			Ok(None) => {
				dev_log!(
					"cocoon",
					"[CocoonShutdown] Child PID {} still alive after $shutdown; sending SIGKILL.",
					Pid
				);

				if let Err(Error) = Child.start_kill() {
					dev_log!("cocoon", "warn: [CocoonShutdown] start_kill failed on PID {}: {}", Pid, Error);
				}

				// Best-effort wait so the OS reaps and frees the port.
				let _ = tokio::time::timeout(std::time::Duration::from_secs(2), Child.wait()).await;
			},

			Err(Error) => {
				dev_log!("cocoon", "warn: [CocoonShutdown] try_wait failed on PID {}: {}", Pid, Error);
			},
		}
	}

	State.IsRunning = false;
}
