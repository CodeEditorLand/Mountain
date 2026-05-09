#![allow(non_snake_case)]

//! `localGit:pull(operationId, repoPath) -> bool`. Three-call
//! sequence: read HEAD, `pull --ff-only`, read HEAD again.
//! Returns `true` when the second HEAD differs from the first
//! (i.e. the pull actually moved the branch). `--ff-only`
//! avoids surprise merge commits - callers handle non-FF cases
//! explicitly.

use serde_json::{Value, json};

use crate::IPC::WindServiceHandlers::Git::Shared::RunGit;

pub async fn HandlePull(Arguments:Vec<Value>) -> Result<Value, String> {

	let OperationId = Arguments.first().and_then(Value::as_str).unwrap_or("").to_string();

	let RepoPath = Arguments.get(1).and_then(Value::as_str).unwrap_or("").to_string();

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
