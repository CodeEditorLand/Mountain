//! Tauri command - switch the working tree to a different branch via
//! `git checkout`. Git itself guards uncommitted changes (refuses the
//! switch and reports which files conflict); that stderr is surfaced
//! verbatim so the UI can show it. Stash-and-switch prompts and
//! create-if-missing (`-b`) remain future work in the module doc.

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
/// Checkouts s c m branch.
pub async fn CheckoutSCMBranch(State:State<'_, Arc<ApplicationState>>, BranchName:String) -> Result<Value, String> {
	dev_log!("commands", "checking out branch: {}", BranchName);

	if BranchName.trim().is_empty() {
		return Err("checkout requires a branch name".to_string());
	}

	let Cwd = RepositoryCwd::Fn(State.inner())?;

	let Args:Vec<String> = vec!["checkout".into(), BranchName.clone()];

	let (ExitCode, _Stdout, Stderr) = RunGit::Fn("scm:checkout", &Args, Some(&Cwd)).await?;

	if ExitCode == 0 {
		Ok(json!({ "success": true, "branch": BranchName }))
	} else {
		Err(if Stderr.trim().is_empty() {
			format!("git checkout {} exited with code {}", BranchName, ExitCode)
		} else {
			Stderr.trim().to_string()
		})
	}
}
