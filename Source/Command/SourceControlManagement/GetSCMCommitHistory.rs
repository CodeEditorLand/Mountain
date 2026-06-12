//! Tauri command - paginated commit log for the SCM viewlet's Timeline
//! panel. `git log` with a tab-separated pretty format yields hash,
//! author, ISO date, parent hashes, and subject in one pass; subjects
//! containing tabs are preserved because the subject is the final field
//! and the splitter caps at five pieces.

use std::sync::Arc;

use serde_json::{Value, json};
use tauri::{State, command};

use crate::{
	ApplicationState::State::ApplicationState::ApplicationState,
	Command::SourceControlManagement::RepositoryCwd,
	IPC::WindServiceHandlers::Git::Shared::RunGit,
	dev_log,
};

#[command]
/// Gets s c m commit history.
pub async fn GetSCMCommitHistory(
	State:State<'_, Arc<ApplicationState>>,

	MaxCount:Option<usize>,
) -> Result<Value, String> {
	let MaxCommits = MaxCount.unwrap_or(50);

	dev_log!("commands", "getting commit history, max count: {}", MaxCommits);

	let Cwd = RepositoryCwd::Fn(State.inner())?;

	let Args:Vec<String> = vec![
		"log".into(),
		format!("-n{}", MaxCommits),
		"--pretty=format:%H%x09%an%x09%aI%x09%P%x09%s".into(),
	];

	let (ExitCode, Stdout, Stderr) = RunGit::Fn("scm:log", &Args, Some(&Cwd)).await?;

	if ExitCode != 0 {
		// An empty repository (no HEAD yet) is a normal state for the
		// Timeline panel - return no commits rather than an error toast.
		if Stderr.contains("does not have any commits yet") {
			return Ok(json!({ "commits": Vec::<Value>::new(), "maxCount": MaxCommits }));
		}

		return Err(if Stderr.trim().is_empty() {
			format!("git log exited with code {}", ExitCode)
		} else {
			Stderr.trim().to_string()
		});
	}

	let Commits:Vec<Value> = Stdout
		.lines()
		.filter(|Line| !Line.trim().is_empty())
		.map(|Line| {
			let Fields:Vec<&str> = Line.splitn(5, '\t').collect();

			json!({
				"hash": Fields.first().copied().unwrap_or(""),
				"author": Fields.get(1).copied().unwrap_or(""),
				"date": Fields.get(2).copied().unwrap_or(""),
				"parents": Fields
					.get(3)
					.copied()
					.unwrap_or("")
					.split_whitespace()
					.collect::<Vec<_>>(),
				"message": Fields.get(4).copied().unwrap_or(""),
			})
		})
		.collect();

	Ok(json!({ "commits": Commits, "maxCount": MaxCommits }))
}
