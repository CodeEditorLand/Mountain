//! Wire the automatic-restart channel: the health monitor sends a backoff
//! duration (seconds) on crash; a dedicated current-thread runtime sleeps
//! then respawns Cocoon via `LaunchAndManageCocoonSideCar`.

use std::sync::Arc;

use crate::{Environment::MountainEnvironment::MountainEnvironment, dev_log};

pub(crate) async fn Fn(ApplicationHandle:&tauri::AppHandle, Environment:&Arc<MountainEnvironment>) {
	let (RestartTx, mut RestartRx) = tokio::sync::mpsc::channel::<u64>(1);

	super::COCOON_STATE.lock().await.RestartTx = Some(RestartTx);

	let RestartAppHandle = ApplicationHandle.clone();

	let RestartEnv = Environment.clone();

	let RestartState = Arc::clone(&super::COCOON_STATE);

	// `LaunchAndManageCocoonSideCar` is `!Send` (parking_lot guards held
	// across .await). We give it a dedicated current-thread runtime
	// wrapped in a `LocalSet` so `spawn_local` works and no cross-thread
	// move is required. `RestartRx`, `AppHandle`, and `Arc<MountainEnvironment>`
	// are all `Send`, so they cross the thread boundary without issue.
	let _ = std::thread::Builder::new()
		.name("cocoon-restart".into())
		.spawn(move || {
			let rt = tokio::runtime::Builder::new_current_thread()
				.enable_all()
				.build()
				.expect("cocoon-restart runtime");

			let local = tokio::task::LocalSet::new();

			local.block_on(&rt, async move {
				while let Some(BackoffSecs) = RestartRx.recv().await {
					tokio::time::sleep(tokio::time::Duration::from_secs(BackoffSecs)).await;

					dev_log!("cocoon", "[CocoonRestart] Restarting Cocoon after {}s backoff...", BackoffSecs);

					{
						let mut Guard = RestartState.lock().await;

						Guard.IsRunning = false;

						Guard.ChildProcess = None;
					}

					match super::LaunchAndManageCocoonSideCar::Fn(RestartAppHandle.clone(), RestartEnv.clone()).await
					{
						Ok(()) => {
							dev_log!("cocoon", "[CocoonRestart] Cocoon restarted successfully");

							RestartState.lock().await.RestartCount = 0;
						},

						Err(Error) => {
							dev_log!("cocoon", "error: [CocoonRestart] Restart failed: {}", Error);
						},
					}
				}
			});
		})
		.expect("cocoon-restart thread");
}
