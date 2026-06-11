//! Background task that monitors Cocoon process health, logs crashes,
//! reports health issues, and schedules automatic restarts with
//! exponential backoff.

use std::{sync::Arc, time::Duration};

use tokio::sync::Mutex;

use crate::{
	IPC::Common::HealthStatus::HealthIssue::Enum as HealthIssue,
	ProcessManagement::CocoonManagement::CocoonProcessState::CocoonProcessState,
	dev_log,
};

/// Once the child process has exited (or never existed), the monitor no
/// longer has anything useful to say - it exits quietly instead of
/// flooding the log with "No Cocoon process to monitor" every 5s, which
/// was rendering the dev log unreadable after any Cocoon crash.
pub(crate) async fn Fn(state:Arc<Mutex<CocoonProcessState>>) {
	loop {
		tokio::time::sleep(Duration::from_secs(super::HEALTH_CHECK_INTERVAL_SECONDS)).await;

		let mut state_guard = state.lock().await;

		// Check if we have a child process to monitor
		if state_guard.ChildProcess.is_some() {
			// Get process ID before checking status
			let process_id = state_guard.ChildProcess.as_ref().map(|c| c.id().unwrap_or(0));

			// Check if process is still running
			let exit_status = {
				let child = state_guard.ChildProcess.as_mut().unwrap();

				child.try_wait()
			};

			match exit_status {
				Ok(Some(exit_code)) => {
					// Process has exited (crashed or terminated)
					let uptime = state_guard.StartTime.map(|t| t.elapsed().as_secs()).unwrap_or(0);

					let exit_code_num = exit_code.code().unwrap_or(-1);

					dev_log!(
						"cocoon",
						"warn: [CocoonHealth] Cocoon process crashed [PID: {}] [Exit Code: {}] [Uptime: {}s]",
						process_id.unwrap_or(0),
						exit_code_num,
						uptime
					);

					// Update state
					state_guard.IsRunning = false;

					state_guard.ChildProcess = None;

					super::COCOON_PID.store(0, std::sync::atomic::Ordering::Relaxed);

					// Report health issue
					{
						let mut health = super::COCOON_HEALTH.lock().await;

						health.AddIssue(HealthIssue::Custom(format!("ProcessCrashed (Exit code: {})", exit_code_num)));

						dev_log!("cocoon", "warn: [CocoonHealth] Health score: {}", health.HealthScore);
					}

					// Schedule an automatic restart with exponential backoff.
					let RestartCount = state_guard.RestartCount;

					if RestartCount < 5 {
						state_guard.RestartCount += 1;

						// Backoff: 1, 2, 4, 8, 16 seconds.
						let BackoffSecs = 1u64 << RestartCount.min(4);

						if let Some(ref Tx) = state_guard.RestartTx {
							let _ = Tx.try_send(BackoffSecs);
						}

						dev_log!(
							"cocoon",
							"[CocoonHealth] Scheduling restart attempt {} in {}s",
							RestartCount + 1,
							BackoffSecs
						);
					} else {
						dev_log!(
							"cocoon",
							"error: [CocoonHealth] Max restarts ({}) reached; not restarting",
							state_guard.RestartCount
						);
					}
				},

				Ok(None) => {
					// Process is still running
					dev_log!(
						"cocoon",
						"[CocoonHealth] Cocoon process is healthy [PID: {}]",
						process_id.unwrap_or(0)
					);
				},

				Err(e) => {
					// Error checking process status
					dev_log!("cocoon", "warn: [CocoonHealth] Error checking process status: {}", e);

					// Report health issue
					{
						let mut health = super::COCOON_HEALTH.lock().await;

						health.AddIssue(HealthIssue::Custom(format!("ProcessCheckError: {}", e)));
					}
				},
			}
		} else {
			// No child process exists - log exactly once, then exit the
			// monitor loop. Prior behaviour: flood the log with
			// "No Cocoon process to monitor" every 5s forever after a
			// crash, making the dev log unreadable. A future respawn will
			// spawn a fresh monitor via `StartCocoon`.
			dev_log!("cocoon", "[CocoonHealth] No Cocoon process to monitor - exiting monitor loop");

			drop(state_guard);

			return;
		}
	}
}
