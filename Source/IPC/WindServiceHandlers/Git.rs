#![allow(non_snake_case, unused_variables, dead_code)]
//! Git subprocess handlers exposed to the renderer via the `localGit`
//! channel. Mirrors stock VS Code's `ILocalGitService` API
//! (`src/vs/platform/git/common/localGitService.ts`) - `clone`, `pull`,
//! `checkout`, `revParse`, `fetch`, `revListCount`, `cancel`. Adds two
//! Land-specific auxiliaries: `exec` (arbitrary argv, used by the Git
//! extension) and `isAvailable` (synchronous feature detection).
//!
//! Cancellation is keyed on `OperationId`; the shared `RunningProcesses`
//! map survives across invokes so the renderer can fire `cancel` from a
//! different Tauri invoke. `tokio::process::Child` doesn't expose a
//! stable Send handle across threads, so we store the PID instead and
//! send `SIGTERM` on cancel (Unix) / TerminateProcess via taskkill on
//! Windows.

use std::{
	collections::HashMap,
	path::PathBuf,
	sync::{Mutex, OnceLock},
};

use serde_json::{Value, json};
use tokio::process::Command;

use crate::dev_log;

fn RunningProcesses() -> &'static Mutex<HashMap<String, u32>> {
	static SLOT:OnceLock<Mutex<HashMap<String, u32>>> = OnceLock::new();
	SLOT.get_or_init(|| Mutex::new(HashMap::new()))
}

fn RegisterPid(OperationId:&str, Pid:u32) {
	if OperationId.is_empty() {
		return;
	}
	if let Ok(mut Map) = RunningProcesses().lock() {
		Map.insert(OperationId.to_string(), Pid);
	}
}

fn ClearPid(OperationId:&str) {
	if OperationId.is_empty() {
		return;
	}
	if let Ok(mut Map) = RunningProcesses().lock() {
		Map.remove(OperationId);
	}
}

fn TakePid(OperationId:&str) -> Option<u32> {
	if OperationId.is_empty() {
		return None;
	}
	RunningProcesses().lock().ok().and_then(|mut M| M.remove(OperationId))
}

fn ResolveCwd(Raw:&str) -> PathBuf {
	if Raw.is_empty() {
		std::env::current_dir().unwrap_or_default()
	} else {
		PathBuf::from(Raw)
	}
}

async fn RunGit(OperationId:&str, Args:&[String], Cwd:Option<&str>) -> Result<(i32, String, String), String> {
	dev_log!(
		"git",
		"[Git] exec-begin op={} cwd={} args=[{}]",
		OperationId,
		Cwd.unwrap_or("<inherit>"),
		Args.join(" ")
	);

	let WorkingDir = Cwd.map(ResolveCwd).unwrap_or_else(|| std::env::current_dir().unwrap_or_default());

	let mut Spawn = Command::new("git");
	Spawn.args(Args).current_dir(&WorkingDir);
	#[cfg(unix)]
	{
		// Keep the child in its own process group so a single SIGTERM
		// targeted at the PID cleans up any pager the subprocess may have
		// spawned.
		use std::os::unix::process::CommandExt;
		unsafe {
			Spawn.pre_exec(|| {
				libc::setsid();
				Ok(())
			});
		}
	}

	let Child = Spawn.spawn().map_err(|Error| {
		dev_log!(
			"git",
			"[Git] exec-spawn-fail op={} args=[{}] error={}",
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
		"[Git] exec-done op={} args=[{}] exit={} stdout={}B stderr={}B",
		OperationId,
		Args.join(" "),
		ExitCode,
		Stdout.len(),
		Stderr.len()
	);

	Ok((ExitCode, Stdout, Stderr))
}

fn AsStringArray(Value:&Value) -> Vec<String> {
	Value.as_array()
		.map(|Arr| Arr.iter().filter_map(|V| V.as_str().map(str::to_string)).collect())
		.unwrap_or_default()
}

fn Generated() -> String { uuid::Uuid::new_v4().to_string() }

/// `localGit:exec` - accept either `{ args, cwd?, operationId? }` or a
/// legacy positional `(args: string[], cwd?: string)`. Returns
/// `{ stdout, stderr, exitCode }`.
pub async fn HandleExec(args:Vec<Value>) -> Result<Value, String> {
	let (Argv, Cwd, OperationId) = match args.first() {
		Some(First) if First.is_object() => {
			let Obj = First.as_object().unwrap();
			let Argv = Obj.get("args").map(AsStringArray).unwrap_or_default();
			let Cwd = Obj.get("cwd").and_then(Value::as_str).unwrap_or("").to_string();
			let OperationId = Obj
				.get("operationId")
				.and_then(Value::as_str)
				.unwrap_or("")
				.to_string();
			(Argv, Cwd, OperationId)
		},
		Some(First) if First.is_array() => {
			let Argv = AsStringArray(First);
			let Cwd = args.get(1).and_then(Value::as_str).unwrap_or("").to_string();
			(Argv, Cwd, String::new())
		},
		_ => (Vec::new(), String::new(), String::new()),
	};

	if Argv.is_empty() {
		return Err("git:exec requires non-empty args".to_string());
	}

	let OperationIdRef = if OperationId.is_empty() { Generated() } else { OperationId };
	let CwdOpt = if Cwd.is_empty() { None } else { Some(Cwd.as_str()) };
	let (ExitCode, Stdout, Stderr) = RunGit(&OperationIdRef, &Argv, CwdOpt).await?;

	Ok(json!({
		"stdout": Stdout,
		"stderr": Stderr,
		"exitCode": ExitCode,
	}))
}

/// `localGit:clone(operationId, cloneUrl, targetPath, ref?)`
pub async fn HandleClone(args:Vec<Value>) -> Result<Value, String> {
	let OperationId = args.first().and_then(Value::as_str).unwrap_or("").to_string();
	let CloneURL = args.get(1).and_then(Value::as_str).unwrap_or("").to_string();
	let TargetPath = args.get(2).and_then(Value::as_str).unwrap_or("").to_string();
	let Reference = args.get(3).and_then(Value::as_str).map(str::to_string);

	if CloneURL.is_empty() || TargetPath.is_empty() {
		return Err("git:clone requires cloneUrl and targetPath".to_string());
	}

	let mut Argv:Vec<String> = vec!["clone".to_string()];
	if let Some(Ref) = Reference {
		Argv.push("--branch".to_string());
		Argv.push(Ref);
	}
	Argv.push("--".to_string());
	Argv.push(CloneURL);
	Argv.push(TargetPath);

	let (ExitCode, _, Stderr) = RunGit(&OperationId, &Argv, None).await?;
	if ExitCode != 0 {
		return Err(format!("git clone failed: {}", Stderr));
	}
	Ok(Value::Null)
}

/// `localGit:pull(operationId, repoPath) -> boolean` (true = HEAD moved).
pub async fn HandlePull(args:Vec<Value>) -> Result<Value, String> {
	let OperationId = args.first().and_then(Value::as_str).unwrap_or("").to_string();
	let RepoPath = args.get(1).and_then(Value::as_str).unwrap_or("").to_string();
	if RepoPath.is_empty() {
		return Err("git:pull requires repoPath".to_string());
	}

	let (BeforeExit, Before, _) = RunGit(&OperationId, &["rev-parse".to_string(), "HEAD".to_string()], Some(&RepoPath)).await?;
	if BeforeExit != 0 {
		return Err("git:pull: failed to read HEAD before pull".to_string());
	}

	let (PullExit, _, PullStderr) =
		RunGit(&OperationId, &["pull".to_string(), "--ff-only".to_string()], Some(&RepoPath)).await?;
	if PullExit != 0 {
		return Err(format!("git pull failed: {}", PullStderr));
	}

	let (AfterExit, After, _) = RunGit(&OperationId, &["rev-parse".to_string(), "HEAD".to_string()], Some(&RepoPath)).await?;
	if AfterExit != 0 {
		return Err("git:pull: failed to read HEAD after pull".to_string());
	}

	Ok(json!(Before.trim() != After.trim()))
}

/// `localGit:checkout(operationId, repoPath, treeish, detached?)`
pub async fn HandleCheckout(args:Vec<Value>) -> Result<Value, String> {
	let OperationId = args.first().and_then(Value::as_str).unwrap_or("").to_string();
	let RepoPath = args.get(1).and_then(Value::as_str).unwrap_or("").to_string();
	let Treeish = args.get(2).and_then(Value::as_str).unwrap_or("").to_string();
	let Detached = args.get(3).and_then(Value::as_bool).unwrap_or(false);

	if RepoPath.is_empty() || Treeish.is_empty() {
		return Err("git:checkout requires repoPath and treeish".to_string());
	}

	let Argv:Vec<String> = if Detached {
		vec!["checkout".to_string(), "--detach".to_string(), Treeish]
	} else {
		vec!["checkout".to_string(), Treeish]
	};

	let (ExitCode, _, Stderr) = RunGit(&OperationId, &Argv, Some(&RepoPath)).await?;
	if ExitCode != 0 {
		return Err(format!("git checkout failed: {}", Stderr));
	}
	Ok(Value::Null)
}

/// `localGit:revParse(repoPath, ref) -> string`
pub async fn HandleRevParse(args:Vec<Value>) -> Result<Value, String> {
	let RepoPath = args.first().and_then(Value::as_str).unwrap_or("").to_string();
	let Reference = args.get(1).and_then(Value::as_str).unwrap_or("HEAD").to_string();
	if RepoPath.is_empty() {
		return Err("git:revParse requires repoPath".to_string());
	}
	let (ExitCode, Stdout, Stderr) = RunGit(&Generated(), &["rev-parse".to_string(), Reference], Some(&RepoPath)).await?;
	if ExitCode != 0 {
		return Err(format!("git rev-parse failed: {}", Stderr));
	}
	Ok(json!(Stdout.trim()))
}

/// `localGit:fetch(operationId, repoPath)`
pub async fn HandleFetch(args:Vec<Value>) -> Result<Value, String> {
	let OperationId = args.first().and_then(Value::as_str).unwrap_or("").to_string();
	let RepoPath = args.get(1).and_then(Value::as_str).unwrap_or("").to_string();
	if RepoPath.is_empty() {
		return Err("git:fetch requires repoPath".to_string());
	}
	let (ExitCode, _, Stderr) = RunGit(&OperationId, &["fetch".to_string()], Some(&RepoPath)).await?;
	if ExitCode != 0 {
		return Err(format!("git fetch failed: {}", Stderr));
	}
	Ok(Value::Null)
}

/// `localGit:revListCount(repoPath, fromRef, toRef) -> number`
pub async fn HandleRevListCount(args:Vec<Value>) -> Result<Value, String> {
	let RepoPath = args.first().and_then(Value::as_str).unwrap_or("").to_string();
	let FromRef = args.get(1).and_then(Value::as_str).unwrap_or("").to_string();
	let ToRef = args.get(2).and_then(Value::as_str).unwrap_or("").to_string();
	if RepoPath.is_empty() || FromRef.is_empty() || ToRef.is_empty() {
		return Err("git:revListCount requires repoPath, fromRef, toRef".to_string());
	}
	let Range = format!("{}..{}", FromRef, ToRef);
	let (ExitCode, Stdout, Stderr) = RunGit(
		&Generated(),
		&["rev-list".to_string(), "--count".to_string(), Range],
		Some(&RepoPath),
	)
	.await?;
	if ExitCode != 0 {
		return Err(format!("git rev-list failed: {}", Stderr));
	}
	Ok(json!(Stdout.trim().parse::<u64>().unwrap_or(0)))
}

/// `localGit:cancel(operationId)` - SIGTERM the pid we stashed for
/// `operationId`. Silent no-op if unknown.
pub async fn HandleCancel(args:Vec<Value>) -> Result<Value, String> {
	let OperationId = args.first().and_then(Value::as_str).unwrap_or("").to_string();
	if let Some(Pid) = TakePid(&OperationId) {
		dev_log!("git", "[Git] cancel op={} pid={}", OperationId, Pid);
		#[cfg(unix)]
		{
			unsafe {
				libc::kill(Pid as libc::pid_t, libc::SIGTERM);
			}
		}
		#[cfg(windows)]
		{
			let _ = std::process::Command::new("taskkill")
				.args(["/PID", &Pid.to_string(), "/T", "/F"])
				.output();
		}
	} else {
		dev_log!("git", "[Git] cancel op={} pid=<unknown>", OperationId);
	}
	Ok(Value::Null)
}

/// `localGit:isAvailable` - cheap `git --version` probe, cached for the
/// process lifetime so UI polling doesn't re-exec git.
pub async fn HandleIsAvailable(_args:Vec<Value>) -> Result<Value, String> {
	static CACHE:OnceLock<bool> = OnceLock::new();
	if let Some(Cached) = CACHE.get() {
		return Ok(json!(*Cached));
	}
	let Available = Command::new("git")
		.arg("--version")
		.output()
		.await
		.map(|O| O.status.success())
		.unwrap_or(false);
	let _ = CACHE.set(Available);
	dev_log!("git", "[Git] isAvailable={}", Available);
	Ok(json!(Available))
}
