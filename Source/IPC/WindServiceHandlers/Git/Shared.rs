#![allow(non_snake_case, dead_code)]

//! Shared helpers for the `Git/*` atomic handlers. Holds:
//!
//! - `RunningProcesses` static - the cancel-by-`OperationId` pid map. Survives
//!   across `tauri::invoke` calls so a different invoke can cancel an in-flight
//!   git op.
//! - `RunGit` - spawn `git`, register the pid, await the output, clear the pid,
//!   return `(exit, stdout, stderr)`.
//! - `AsStringArray`, `Generated`, `ResolveCwd` - small parsers used by every
//!   entry point.

use std::{
	collections::HashMap,
	path::PathBuf,
	sync::{Mutex, OnceLock},
};

use serde_json::Value;
use tokio::process::Command;

use crate::dev_log;

pub fn RunningProcesses() -> &'static Mutex<HashMap<String, u32>> {
	static SLOT:OnceLock<Mutex<HashMap<String, u32>>> = OnceLock::new();

	SLOT.get_or_init(|| Mutex::new(HashMap::new()))
}

pub fn RegisterPid(OperationId:&str, Pid:u32) {
	if OperationId.is_empty() {
		return;
	}

	if let Ok(mut Map) = RunningProcesses().lock() {
		Map.insert(OperationId.to_string(), Pid);
	}
}

pub fn ClearPid(OperationId:&str) {
	if OperationId.is_empty() {
		return;
	}

	if let Ok(mut Map) = RunningProcesses().lock() {
		Map.remove(OperationId);
	}
}

pub fn TakePid(OperationId:&str) -> Option<u32> {
	if OperationId.is_empty() {
		return None;
	}

	RunningProcesses().lock().ok().and_then(|mut M| M.remove(OperationId))
}

pub fn ResolveCwd(Raw:&str) -> PathBuf {
	if Raw.is_empty() {
		std::env::current_dir().unwrap_or_default()
	} else {
		PathBuf::from(Raw)
	}
}

pub async fn RunGit(OperationId:&str, Args:&[String], Cwd:Option<&str>) -> Result<(i32, String, String), String> {
	dev_log!(
		"git",
		"[Git] exec-begin op={} cwd={} Arguments=[{}]",
		OperationId,
		Cwd.unwrap_or("<inherit>"),
		Args.join(" ")
	);

	let WorkingDir = Cwd
		.map(ResolveCwd)
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
		RegisterPid(OperationId, Pid);
	}

	let Output = Child.wait_with_output().await.map_err(|Error| {
		ClearPid(OperationId);
		format!("git wait failed: {}", Error)
	})?;

	ClearPid(OperationId);

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

pub fn AsStringArray(Value:&Value) -> Vec<String> {
	Value
		.as_array()
		.map(|Arr| Arr.iter().filter_map(|V| V.as_str().map(str::to_string)).collect())
		.unwrap_or_default()
}

pub fn Generated() -> String { uuid::Uuid::new_v4().to_string() }
