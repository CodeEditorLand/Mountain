//! Spawn `git` with the requested args inside `repository_path` (or cwd
//! if unset). stdout lines are returned verbatim; stderr lines are
//! prefixed with `stderr: ` so the extension can differentiate.

use tonic::{Response, Status};

use crate::{
	RPC::CocoonService::CocoonServiceImpl,
	Vine::Generated::{GitExecRequest, GitExecResponse},
	dev_log,
};

pub async fn Fn(_Service:&CocoonServiceImpl, Request:GitExecRequest) -> Result<Response<GitExecResponse>, Status> {
	dev_log!("cocoon", "[CocoonService] git_exec: {}", Request.args.join(" "));

	dev_log!(
		"git",
		"[Git] exec-begin cwd={} args=[{}]",
		if Request.repository_path.is_empty() {
			"<cwd>".to_string()
		} else {
			Request.repository_path.clone()
		},
		Request.args.join(" ")
	);

	let WorkingDirectory = if Request.repository_path.is_empty() {
		std::env::current_dir().unwrap_or_default()
	} else {
		std::path::PathBuf::from(&Request.repository_path)
	};

	let Output = tokio::process::Command::new("git")
		.args(&Request.args)
		.current_dir(&WorkingDirectory)
		.output()
		.await
		.map_err(|Error| {
			dev_log!("cocoon", "error: [CocoonService] git_exec failed to spawn: {}", Error);
			dev_log!(
				"git",
				"[Git] exec-spawn-fail cwd={:?} args=[{}] error={}",
				WorkingDirectory,
				Request.args.join(" "),
				Error
			);
			Status::internal(format!("git_exec: failed to spawn git: {}", Error))
		})?;

	let ExitCode = Output.Status.code().unwrap_or(-1);

	dev_log!(
		"cocoon",
		"[CocoonService] git_exec exit={} stdout={} bytes stderr={} bytes",
		ExitCode,
		Output.stdout.len(),
		Output.stderr.len()
	);

	dev_log!(
		"git",
		"[Git] exec-done args=[{}] exit={} stdout={} stderr={}",
		Request.args.join(" "),
		ExitCode,
		Output.stdout.len(),
		Output.stderr.len()
	);

	let StdoutString = String::from_utf8_lossy(&Output.stdout);

	let StderrString = String::from_utf8_lossy(&Output.stderr);

	let mut OutputLines:Vec<String> = StdoutString.lines().map(|L| L.to_string()).collect();

	for Line in StderrString.lines() {
		OutputLines.push(format!("stderr: {}", Line));
	}

	Ok(Response::new(GitExecResponse { output:OutputLines, exit_code:ExitCode }))
}
