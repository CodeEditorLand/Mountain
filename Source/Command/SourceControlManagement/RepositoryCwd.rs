//! Resolves the working directory for SCM `git` subprocess commands:
//! the first workspace folder's file-system path. SCM viewlet commands
//! operate on "the open repository"; multi-root selection by provider
//! rootUri is future work tracked in the parent module doc.

use std::sync::Arc;

use crate::ApplicationState::State::ApplicationState::ApplicationState;

pub fn Fn(State:&Arc<ApplicationState>) -> Result<String, String> {
	State
		.Workspace
		.GetWorkspaceFolders()
		.first()
		.and_then(|Folder| Folder.URI.to_file_path().ok())
		.map(|Path| Path.to_string_lossy().into_owned())
		.ok_or_else(|| "No workspace folder open - SCM commands require a repository root".to_string())
}
