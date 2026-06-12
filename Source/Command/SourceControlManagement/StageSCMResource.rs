//! Tauri command - stage / unstage a single resource. `Staged:true`
//! runs `git add -- <path>`; `Staged:false` runs
//! `git restore --staged -- <path>`. The resource may arrive as a
//! `file://` URI (the SCM viewlet's serialised vscode.Uri) or a raw
//! file-system path; both resolve to the same argv. The `--` separator
//! keeps a path that looks like a flag from being parsed as one.

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
/// Stages s c m resource.
pub async fn StageSCMResource(
	State:State<'_, Arc<ApplicationState>>,

	ResourceURI:String,

	Staged:bool,
) -> Result<Value, String> {
	dev_log!("commands", "staging resource: {}, staged: {}", ResourceURI, Staged);

	if ResourceURI.trim().is_empty() {
		return Err("stage requires a resource path".to_string());
	}

	let Path = if ResourceURI.starts_with("file://") {
		url::Url::parse(&ResourceURI)
			.ok()
			.and_then(|U| U.to_file_path().ok())
			.map(|P| P.to_string_lossy().into_owned())
			.unwrap_or(ResourceURI.clone())
	} else {
		ResourceURI.clone()
	};

	let Cwd = RepositoryCwd::Fn(State.inner())?;

	let Args:Vec<String> = if Staged {
		vec!["add".into(), "--".into(), Path]
	} else {
		vec!["restore".into(), "--staged".into(), "--".into(), Path]
	};

	let (ExitCode, _Stdout, Stderr) = RunGit::Fn("scm:stage", &Args, Some(&Cwd)).await?;

	if ExitCode == 0 {
		Ok(json!({ "success": true }))
	} else {
		Err(if Stderr.trim().is_empty() {
			format!("git {} exited with code {}", Args.join(" "), ExitCode)
		} else {
			Stderr.trim().to_string()
		})
	}
}
