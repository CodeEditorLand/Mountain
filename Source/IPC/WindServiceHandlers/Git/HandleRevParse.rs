//! `localGit:revParse(repoPath, ref) -> string`. Defaults
//! `ref=HEAD` so the caller can pass two args or three. Output
//! is trimmed - `git rev-parse` ships a trailing newline that
//! breaks string equality on the JS side.

use serde_json::{Value, json};

use crate::IPC::WindServiceHandlers::{
	Git::Shared::RunGit::Fn as RunGit,
	Utilities::JsonValueHelpers::{arg_string, arg_string_or},
};

pub async fn Fn(Arguments:Vec<Value>) -> Result<Value, String> {
	let RepoPath = arg_string(&Arguments, 0);

	let Reference = arg_string_or(&Arguments, 1, "HEAD");

	if RepoPath.is_empty() {
		return Err("git:revParse requires repoPath".to_string());
	}

	let (ExitCode, Stdout, Stderr) = RunGit(
		&uuid::Uuid::new_v4().to_string(),
		&["rev-parse".to_string(), Reference],
		Some(&RepoPath),
	)
	.await?;

	if ExitCode != 0 {
		return Err(format!("git rev-parse failed: {}", Stderr));
	}

	Ok(json!(Stdout.trim()))
}
