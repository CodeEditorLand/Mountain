//! `CocoonManagement::HardKillCocoon`

use std::{collections::HashMap, process::Stdio, sync::Arc, time::Duration};
use CommonLibrary::Error::CommonError::CommonError;
use tauri::{
	AppHandle,
	Manager,
	Wry,
	path::{BaseDirectory, PathResolver},
};
use tokio::{
	io::{AsyncBufReadExt, BufReader},
	process::{Child, Command},
	sync::Mutex,
	time::sleep,
};
use super::{InitializationData, NodeResolver};
use crate::{
	Environment::MountainEnvironment::MountainEnvironment,
	IPC::Common::HealthStatus::{HealthIssue::Enum as HealthIssue, HealthMonitor::Struct as HealthMonitor},
	ProcessManagement::ExtractDevTag::Fn as ExtractDevTag,
	Vine,
	dev_log,
};

const COCOON_SIDE_CAR_IDENTIFIER:&str = "cocoon-main";
const COCOON_GRPC_PORT:u16 = 50052;
const MOUNTAIN_GRPC_PORT:u16 = 50051;
const BOOTSTRAP_SCRIPT_PATH:&str = "scripts/cocoon/bootstrap-fork.js";
const GRPC_CONNECT_INITIAL_MS:u64 = 50;
const GRPC_CONNECT_MAX_DELAY_MS:u64 = 2_000;
const GRPC_CONNECT_BUDGET_MS:u64 = 30_000;
const COCOON_BUNDLE_PROBE:&str = "../Cocoon/Target/Bootstrap/Implementation/Cocoon/Main.js";
const HANDSHAKE_TIMEOUT_MS:u64 = 60000;
const HEALTH_CHECK_INTERVAL_SECONDS:u64 = 5;
const MAX_RESTART_ATTEMPTS:u32 = 3;
const RESTART_WINDOW_SECONDS:u64 = 300;
static COCOON_PID:std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

/// Atom I6: post-shutdown hard-kill. Called by RuntimeShutdown after the
/// `$shutdown` gRPC notification has been sent (and either succeeded or
/// timed out). Grabs the stored `Child` handle and force-terminates it if
/// still alive, then resets COCOON_STATE. This plugs the "Mountain exits
/// cleanly but child stays running" leak that leads to zombie-Cocoon
/// zombies holding the gRPC port.
///
/// Call AFTER the graceful $shutdown attempt - we don't want to race the
/// child's own cleanup. Safe to call with no stored child (no-op).
pub async fn Fn() {
	let mut State = COCOON_STATE.lock().await;

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
