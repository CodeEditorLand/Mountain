//! Spawns `git`, registers the PID, awaits output, returns
//! `(exit_code, stdout, stderr)`.

use std::time::Duration;

use tokio::process::Command;

use crate::dev_log;

/// Upper-bound wall time for any single `git` invocation. Generous enough that
/// legitimately slow operations (large monorepo clones with credential
/// prompts, file-share-backed working copies, full-repo `log` walks) finish
/// well within budget, but tight enough that a hung subprocess - stalled on
/// a credential prompt with no TTY, a stuck index lock, or a network mount
/// that has gone unresponsive - releases the Mountain effect slot before
/// the extension host's own watchdog fires.
const GIT_EXEC_TIMEOUT:Duration = Duration::from_secs(30);

pub async fn Fn(OperationId:&str, Args:&[String], Cwd:Option<&str>) -> Result<(i32, String, String), String> {
	dev_log!(
		"git",
		"[Git] exec-begin op={} cwd={} Arguments=[{}]",
		OperationId,
		Cwd.unwrap_or("<inherit>"),
		Args.join(" ")
	);

	let WorkingDir = Cwd
		.map(super::ResolveCwd::Fn)
		.unwrap_or_else(|| std::env::current_dir().unwrap_or_default());

	let mut Spawn = Command::new("git");

	Spawn.args(Args).current_dir(&WorkingDir).kill_on_drop(true);

	let Child = Spawn.spawn().map_err(|Error| {
		dev_log!(
			"git",
			"[Git] exec-spawn-fail op={} Arguments=[{}] error={}",
			OperationId,
			Args.join(" "),
			Error
		);
		format!("git spawn failed: {}", Error)
	})?;

	if let Some(Pid) = Child.id() {
		super::RegisterPid::Fn(OperationId, Pid);
	}

	let WaitFuture = Child.wait_with_output();

	let Output = match tokio::time::timeout(GIT_EXEC_TIMEOUT, WaitFuture).await {
		Ok(WaitResult) => {
			WaitResult.map_err(|Error| {
				super::ClearPid::Fn(OperationId);
				format!("git wait failed: {}", Error)
			})?
		},
		Err(_) => {
			// Timeout expired. SIGTERM the PID via the registry-aware helper
			// so the running-process accounting stays consistent; the
			// kill_on_drop(true) on the Command also covers the race where
			// the subprocess hasn't been observed yet.
			let _ = super::TakePid::Fn(OperationId);

			dev_log!(
				"git",
				"warn: [Git] exec-timeout op={} Arguments=[{}] after {}s - subprocess killed",
				OperationId,
				Args.join(" "),
				GIT_EXEC_TIMEOUT.as_secs()
			);

			return Err(format!(
				"git exec timed out after {}s: git {}",
				GIT_EXEC_TIMEOUT.as_secs(),
				Args.join(" ")
			));
		},
	};

	super::ClearPid::Fn(OperationId);

	let ExitCode = Output.status.code().unwrap_or(-1);

	let Stdout = String::from_utf8_lossy(&Output.stdout).into_owned();

	let Stderr = String::from_utf8_lossy(&Output.stderr).into_owned();

	dev_log!(
		"git",
		"[Git] exec-done op={} Arguments=[{}] exit={} stdout={}B stderr={}B",
		OperationId,
		Args.join(" "),
		ExitCode,
		Stdout.len(),
		Stderr.len()
	);

	Ok((ExitCode, Stdout, Stderr))
}
