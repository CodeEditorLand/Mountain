
//! Spawns `git`, registers the PID, awaits output, returns
//! `(exit_code, stdout, stderr)`.

use tokio::process::Command;

use crate::dev_log;

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

	let Output = Child.wait_with_output().await.map_err(|Error| {
		super::ClearPid::Fn(OperationId);
		format!("git wait failed: {}", Error)
	})?;

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
