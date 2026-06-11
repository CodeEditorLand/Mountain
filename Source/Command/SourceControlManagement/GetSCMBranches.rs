//! Tauri command - list branches for the open repository. Drives the
//! branch picker UI. Runs `git branch -a` with a tab-separated
//! `--format` so local and remote branches come back with their
//! current-branch marker and upstream tracking ref in one invocation.

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
pub async fn GetSCMBranches(
	State:State<'_, Arc<ApplicationState>>,

	ProviderIdentifier:String,
) -> Result<Value, String> {
	dev_log!("commands", "getting branches for provider: {}", ProviderIdentifier);

	let Cwd = RepositoryCwd::Fn(State.inner())?;

	let Args:Vec<String> = vec![
		"branch".into(),
		"-a".into(),
		"--format=%(refname:short)%09%(HEAD)%09%(upstream:short)".into(),
	];

	let (ExitCode, Stdout, Stderr) = RunGit::Fn("scm:branches", &Args, Some(&Cwd)).await?;

	if ExitCode != 0 {
		return Err(if Stderr.trim().is_empty() {
			format!("git branch exited with code {}", ExitCode)
		} else {
			Stderr.trim().to_string()
		});
	}

	let Branches:Vec<Value> = Stdout
		.lines()
		.filter(|Line| !Line.trim().is_empty())
		.map(|Line| {
			let mut Fields = Line.split('\t');

			let Name = Fields.next().unwrap_or("").trim().to_string();

			let IsCurrent = Fields.next().map(|H| H.trim() == "*").unwrap_or(false);

			let Upstream = Fields.next().map(str::trim).filter(|U| !U.is_empty()).map(str::to_owned);

			// `--format=%(refname:short)` over `branch -a` yields remote
			// refs as `origin/<name>`; locals never contain `/` unless the
			// user named them so, which the upstream column disambiguates.
			let IsRemote = Upstream.is_none() && Name.starts_with("origin/");

			json!({
				"name": Name,
				"isCurrent": IsCurrent,
				"isRemote": IsRemote,
				"upstream": Upstream,
			})
		})
		.collect();

	Ok(json!({ "branches": Branches }))
}
