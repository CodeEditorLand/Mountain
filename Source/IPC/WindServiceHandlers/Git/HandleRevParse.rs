
//! `localGit:revParse(repoPath, ref) -> string`. Defaults
//! `ref=HEAD` so the caller can pass two args or three. Output
//! is trimmed - `git rev-parse` ships a trailing newline that
//! breaks string equality on the JS side.

use serde_json::{Value, json};

use crate::IPC::WindServiceHandlers::Git::Shared::{Generated::Fn as Generated, RunGit::Fn as RunGit};

pub async fn Fn(Arguments:Vec<Value>) -> Result<Value, String> {
	let RepoPath = Arguments.first().and_then(Value::as_str).unwrap_or("").to_string();

	let Reference = Arguments.get(1).and_then(Value::as_str).unwrap_or("HEAD").to_string();

	if RepoPath.is_empty() {
		return Err("git:revParse requires repoPath".to_string());
	}

	let (ExitCode, Stdout, Stderr) =
		RunGit(&Generated(), &["rev-parse".to_string(), Reference], Some(&RepoPath)).await?;

	if ExitCode != 0 {
		return Err(format!("git rev-parse failed: {}", Stderr));
	}

	Ok(json!(Stdout.trim()))
}
