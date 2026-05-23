//! `localGit:fetch(operationId, repoPath)`. Plain `git fetch`
//! against the configured upstream - no remote argument, no
//! `--all`, mirroring stock VS Code's `LocalGitService.fetch`.

use serde_json::Value;

use crate::IPC::WindServiceHandlers::Git::Shared::RunGit::Fn as RunGit;

pub async fn Fn(Arguments:Vec<Value>) -> Result<Value, String> {
	let OperationId = Arguments.first().and_then(Value::as_str).unwrap_or("").to_string();

	let RepoPath = Arguments.get(1).and_then(Value::as_str).unwrap_or("").to_string();

	if RepoPath.is_empty() {
		return Err("git:fetch requires repoPath".to_string());
	}

	let (ExitCode, _, Stderr) = RunGit(&OperationId, &["fetch".to_string()], Some(&RepoPath)).await?;

	if ExitCode != 0 {
		return Err(format!("git fetch failed: {}", Stderr));
	}

	Ok(Value::Null)
}
