//! `localGit:revListCount(repoPath, fromRef, toRef) -> u64`.
//! Equivalent to `git rev-list --count from..to` - counts
//! commits the GitLens / SCM viewlet "ahead/behind" badges
//! display.

use serde_json::{Value, json};

use crate::IPC::WindServiceHandlers::Git::Shared::{Generated::Fn as Generated, RunGit::Fn as RunGit};

pub async fn Fn(Arguments:Vec<Value>) -> Result<Value, String> {
	let RepoPath = Arguments.first().and_then(Value::as_str).unwrap_or("").to_string();

	let FromRef = Arguments.get(1).and_then(Value::as_str).unwrap_or("").to_string();

	let ToRef = Arguments.get(2).and_then(Value::as_str).unwrap_or("").to_string();

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
