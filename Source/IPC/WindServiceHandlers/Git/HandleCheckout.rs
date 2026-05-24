//! `localGit:checkout(operationId, repoPath, treeish, detached?)`.
//! `Detached=true` adds `--detach` so the caller can land on a
//! commit hash without creating a tracking branch.

use serde_json::Value;

use crate::IPC::WindServiceHandlers::{
	Git::Shared::RunGit::Fn as RunGit,
	Utilities::JsonValueHelpers::{ArgBool, ArgString},
};

pub async fn Fn(Arguments:Vec<Value>) -> Result<Value, String> {
	let OperationId = ArgString(&Arguments, 0);

	let RepoPath = ArgString(&Arguments, 1);

	let Treeish = ArgString(&Arguments, 2);

	let Detached = ArgBool(&Arguments, 3);

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
