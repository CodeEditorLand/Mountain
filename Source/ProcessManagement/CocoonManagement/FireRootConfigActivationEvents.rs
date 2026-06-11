//! Supplementary startup activation: probe the first workspace folder for
//! well-known root-level config files (Cargo.toml, package.json, `*.sln`, …)
//! and fire the matching `workspaceContains:` events.

use std::sync::Arc;

use crate::Environment::MountainEnvironment::MountainEnvironment;

pub(crate) async fn Fn(Environment:&Arc<MountainEnvironment>) {
	// Supplementary workspaceContains check for common root-level config
	// files. Phase 2 fires pattern-based events from extension manifests;
	// this block fires well-known events that extensions may declare via
	// exact filenames rather than glob patterns, ensuring activations aren't
	// missed when the manifest scan completes before the workspace is open.
	let WorkspaceFolders:Vec<std::path::PathBuf> = Environment
		.ApplicationState
		.Workspace
		.WorkspaceFolders
		.lock()
		.iter()
		.filter_map(|Folder| Folder.URI.to_file_path().ok())
		.collect();

	if !WorkspaceFolders.is_empty() {
		if let Some(First) = WorkspaceFolders.first() {
			for (FileName, Event) in [
				("Cargo.toml", "workspaceContains:Cargo.toml"),
				("package.json", "workspaceContains:package.json"),
				("tsconfig.json", "workspaceContains:tsconfig.json"),
				("go.mod", "workspaceContains:go.mod"),
				("pyproject.toml", "workspaceContains:pyproject.toml"),
				("CMakeLists.txt", "workspaceContains:CMakeLists.txt"),
				("build.gradle", "workspaceContains:build.gradle"),
				("pom.xml", "workspaceContains:pom.xml"),
				("Dockerfile", "workspaceContains:Dockerfile"),
				("composer.json", "workspaceContains:composer.json"),
				("Gemfile", "workspaceContains:Gemfile"),
			] {
				let FilePath = First.join(FileName);

				if FilePath.exists() {
					let _ = crate::Vine::Client::SendNotification::Fn(
						"cocoon-main".to_string(),
						"$activateByEvent".to_string(),
						serde_json::json!({ "activationEvent": Event }),
					)
					.await;
				}
			}

			// C#/.NET solution files match `**/*.sln` - a glob that
			// cannot be resolved with a simple path join. Probe the
			// workspace root for any `.sln` entry directly instead.
			let HasSln = First
				.read_dir()
				.ok()
				.map(|Entries| {
					Entries
						.flatten()
						.any(|E| E.path().extension().and_then(|X| X.to_str()) == Some("sln"))
				})
				.unwrap_or(false);

			if HasSln {
				let _ = crate::Vine::Client::SendNotification::Fn(
					"cocoon-main".to_string(),
					"$activateByEvent".to_string(),
					serde_json::json!({ "activationEvent": "workspaceContains:**/*.sln" }),
				)
				.await;
			}
		}
	}
}
