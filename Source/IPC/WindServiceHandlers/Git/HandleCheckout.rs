//! `localGit:checkout(operationId, repoPath, treeish, detached?)`.
//! `Detached=true` adds `--detach` so the caller can land on a
//! commit hash without creating a tracking branch.

use serde_json::Value;

use crate::IPC::WindServiceHandlers::{
	Git::Shared::RunGit::Fn as RunGit,
	Utilities::JsonValueHelpers::{arg_bool, arg_string},
};

pub async fn Fn(Arguments:Vec<Value>) -> Result<Value, String> {
	let OperationId = arg_string(&Arguments, 0);

	let RepoPath = arg_string(&Arguments, 1);

	let Treeish = arg_string(&Arguments, 2);

	let Detached = arg_bool(&Arguments, 3);

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
