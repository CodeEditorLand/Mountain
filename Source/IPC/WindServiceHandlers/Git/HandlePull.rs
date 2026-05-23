//! `localGit:pull(operationId, repoPath) -> bool`. Three-call
//! sequence: read HEAD, `pull --ff-only`, read HEAD again.
//! Returns `true` when the second HEAD differs from the first
//! (i.e. the pull actually moved the branch). `--ff-only`
//! avoids surprise merge commits - callers handle non-FF cases
//! explicitly.

use serde_json::{Value, json};

use crate::IPC::WindServiceHandlers::{Git::Shared::RunGit::Fn as RunGit, Utilities::JsonValueHelpers::arg_string};

pub async fn Fn(Arguments:Vec<Value>) -> Result<Value, String> {
	let OperationId = arg_string(&Arguments, 0);

	let RepoPath = arg_string(&Arguments, 1);

	if RepoPath.is_empty() {
		return Err("git:pull requires repoPath".to_string());
	}

	let (BeforeExit, Before, _) =
		RunGit(&OperationId, &["rev-parse".to_string(), "HEAD".to_string()], Some(&RepoPath)).await?;

	if BeforeExit != 0 {
		return Err("git:pull: failed to read HEAD before pull".to_string());
	}

	let (PullExit, _, PullStderr) =
		RunGit(&OperationId, &["pull".to_string(), "--ff-only".to_string()], Some(&RepoPath)).await?;

	if PullExit != 0 {
		return Err(format!("git pull failed: {}", PullStderr));
	}

	let (AfterExit, After, _) =
		RunGit(&OperationId, &["rev-parse".to_string(), "HEAD".to_string()], Some(&RepoPath)).await?;

	if AfterExit != 0 {
		return Err("git:pull: failed to read HEAD after pull".to_string());
	}

	Ok(json!(Before.trim() != After.trim()))
}
