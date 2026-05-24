//! `CocoonManagement::GetCocoonPid`

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

/// Return the Cocoon child process's OS PID, or `None` if Cocoon has not
/// been spawned (or has exited).
pub fn Fn() -> Option<u32> {
	match COCOON_PID.load(std::sync::atomic::Ordering::Relaxed) {
		0 => None,

		Pid => Some(Pid),
	}
}
