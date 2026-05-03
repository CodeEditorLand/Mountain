#![allow(non_snake_case)]

//! `localGit:checkout(operationId, repoPath, treeish, detached?)`.
//! `Detached=true` adds `--detach` so the caller can land on a
//! commit hash without creating a tracking branch.

use serde_json::Value;

use crate::IPC::WindServiceHandlers::Git::Shared::RunGit;

pub async fn HandleCheckout(Arguments:Vec<Value>) -> Result<Value, String> {
	let OperationId = Arguments.first().and_then(Value::as_str).unwrap_or("").to_string();
	let RepoPath = Arguments.get(1).and_then(Value::as_str).unwrap_or("").to_string();
	let Treeish = Arguments.get(2).and_then(Value::as_str).unwrap_or("").to_string();
	let Detached = Arguments.get(3).and_then(Value::as_bool).unwrap_or(false);

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
