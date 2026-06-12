//! Tauri command - dispatch SCM operations (commit / push / pull /
//! fetch) as real `git` subprocess invocations against the first
//! workspace folder, via the shared `Git::Shared::RunGit` runner (PID
//! registry + 30s timeout). A non-zero exit surfaces stderr as the
//! error so the SCM viewlet shows git's own message instead of a
//! mocked success.

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
/// Executes s c m command.
pub async fn ExecuteSCMCommand(
	State:State<'_, Arc<ApplicationState>>,

	CommandName:String,

	Arguments:Value,
) -> Result<Value, String> {
	dev_log!("commands", "executing SCM command: {}", CommandName);

	let Cwd = RepositoryCwd::Fn(State.inner())?;

	let GitArgs:Vec<String> = match CommandName.as_str() {
		"git.commit" | "commit" => {
			let Message = Arguments
				.as_str()
				.map(str::to_owned)
				.or_else(|| Arguments.get("message").and_then(Value::as_str).map(str::to_owned))
				.filter(|M| !M.is_empty())
				.ok_or("commit requires a non-empty message".to_string())?;

			vec!["commit".into(), "-m".into(), Message]
		},

		"git.push" | "push" => vec!["push".into()],

		"git.pull" | "pull" => vec!["pull".into()],

		"git.fetch" | "fetch" => vec!["fetch".into(), "--all".into(), "--prune".into()],

		_ => return Err(format!("Unknown SCM command: {}", CommandName)),
	};

	let OperationId = format!("scm:{}", CommandName);

	let (ExitCode, Stdout, Stderr) = RunGit::Fn(&OperationId, &GitArgs, Some(&Cwd)).await?;

	if ExitCode == 0 {
		Ok(json!({
			"success": true,
			"message": if Stdout.trim().is_empty() { Stderr.trim() } else { Stdout.trim() },
		}))
	} else {
		Err(if Stderr.trim().is_empty() {
			format!("git {} exited with code {}", GitArgs.join(" "), ExitCode)
		} else {
			Stderr.trim().to_string()
		})
	}
}
