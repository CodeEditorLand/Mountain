//! Phase 2 startup activation: collect every `workspaceContains:<pattern>`
//! activation event from the scanned extension registry and fire each event
//! whose pattern matches at least one workspace folder.

use std::sync::Arc;

use crate::{Environment::MountainEnvironment::MountainEnvironment, dev_log};

pub(crate) async fn Fn(SideCarId:&str, Environment:&Arc<MountainEnvironment>) {
	// Phase 2: workspaceContains: events. Iterate the scanned
	// extension registry, collect every pattern contributed via the
	// `workspaceContains:<pattern>` activation event, and fire the
	// event if at least one workspace folder contains a path
	// matching the pattern. Patterns are treated as filename globs
	// relative to any workspace folder root; matching is done with
	// a lightweight walk bounded by depth 3 and 2048 total visited
	// entries per folder to cap worst-case cost on huge repos.
	let WorkspacePatterns = {
		let AppState = &Environment.ApplicationState;

		let Folders:Vec<std::path::PathBuf> = AppState
			.Workspace
			.WorkspaceFolders
			.lock()
			.iter()
			.filter_map(|Folder| Folder.URI.to_file_path().ok())
			.collect::<Vec<_>>();

		let Patterns:Vec<String> = AppState
			.Extension
			.ScannedExtensions
			.ScannedExtensions
			.lock()
			.values()
			.filter_map(|D| D.ActivationEvents.as_ref())
			.flat_map(|Events| Events.iter())
			.filter_map(|E| E.strip_prefix("workspaceContains:").map(str::to_string))
			.collect::<std::collections::BTreeSet<_>>()
			.into_iter()
			.collect();

		(Folders, Patterns)
	};

	let (WorkspaceFolders, Patterns):(Vec<std::path::PathBuf>, Vec<String>) = WorkspacePatterns;

	if !WorkspaceFolders.is_empty() && !Patterns.is_empty() {
		let Matched = super::FindMatchingWorkspaceContainsPatterns::Fn(&WorkspaceFolders, &Patterns);

		dev_log!(
			"exthost",
			"[CocoonManagement] workspaceContains scan: {} pattern(s) matched across {} folder(s)",
			Matched.len(),
			WorkspaceFolders.len()
		);

		for Pattern in Matched {
			let Event = format!("workspaceContains:{}", Pattern);

			if let Err(Error) = crate::Vine::Client::SendRequest::Fn(
				SideCarId,
				"$activateByEvent".to_string(),
				serde_json::json!({ "activationEvent": Event }),
				30_000,
			)
			.await
			{
				dev_log!(
					"cocoon",
					"warn: [CocoonManagement] $activateByEvent({}) failed: {}",
					Event,
					Error
				);
			}
		}
	}
}
