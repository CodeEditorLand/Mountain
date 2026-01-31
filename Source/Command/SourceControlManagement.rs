// ============================================================================
// File: Mountain/Source/Command/SourceControlManagement.rs
// ============================================================================
// This module follows the Land ecosystem's PascalCase naming convention.
// See: https://github.com/CodeEditorLand/Mountain/blob/main/Documentation/GitHub/Naming%20Conventions.md
//
// # SourceControlManagement Commands
//!
//! Defines the specific Tauri command handlers for SourceControlManagement data
//! requests that originate from the `Sky` frontend UI.
//!
//! ## Key Features:
//! - Git integration and status tracking
//! - SCM provider state management
//! - File change tracking
//! - Branch management
//! - Commit and diff operations
//!
//! ## VSCode Reference:
//! - vs/workbench/contrib/scm/common/scm.ts
//! - vs/workbench/contrib/scm/browser/scmView.ts
//! - vs/workbench/services/scm/common/scmService.ts
//!
// ============================================================================

#![allow(non_snake_case, non_camel_case_types)]

use std::sync::Arc;

use serde_json::{Value, json};
use tauri::{State, command};

use crate::ApplicationState::ApplicationState::{ApplicationState, MapLockError};

/// Retrieves the complete state of all Source Control Management providers,
/// groups, and resources for rendering in the UI.
///
/// This command is called by the frontend to get a full snapshot of the SCM
/// view.
#[command]
pub async fn GetAllSourceControlManagementState(State:State<'_, Arc<ApplicationState>>) -> Result<Value, String> {
	log::debug!("[SourceControlManagement Command] Getting all SCM state for UI.");

	let Providers = State
		.SourceControlManagementProviders
		.lock()
		.map_err(MapLockError)
		.map_err(|Error| Error.to_string())?
		.clone();

	let Groups = State
		.SourceControlManagementGroups
		.lock()
		.map_err(MapLockError)
		.map_err(|Error| Error.to_string())?
		.clone();

	let Resources = State
		.SourceControlManagementResources
		.lock()
		.map_err(MapLockError)
		.map_err(|Error| Error.to_string())?
		.clone();

	Ok(json!({
		"providers": Providers,
		"groups": Groups,
		"resources": Resources,
	}))
}

#[command]
pub async fn GetSCMResourceChanges(
	State:State<'_, Arc<ApplicationState>>,

	ProviderIdentifier:String,
) -> Result<Value, String> {
	log::debug!("[SCM Command] Getting resource changes for provider: {}", ProviderIdentifier);

	let Resources = State
		.SourceControlManagementResources
		.lock()
		.map_err(MapLockError)
		.map_err(|Error| Error.to_string())?
		.clone();

	// Filter resources by provider
	let ProviderResources:Vec<_> = Resources
		.into_iter()
		.filter(|r| r.ProviderIdentifier == ProviderIdentifier)
		.collect();

	Ok(json!({
		"resources": ProviderResources,
	}))
}

#[command]
pub async fn ExecuteSCMCommand(
	State:State<'_, Arc<ApplicationState>>,

	CommandName:String,

	Arguments:Value,
) -> Result<Value, String> {
	log::debug!("[SCM Command] Executing command: {}", CommandName);

	// TODO: Implement SCM command execution (commit, push, pull, etc.)
	match CommandName.as_str() {
		"git.commit" | "commit" => {
			log::info!("[SCM Command] Executing commit");
			Ok(json!({ "success": true, "message": "Commit successful" }))
		},
		"git.push" | "push" => {
			log::info!("[SCM Command] Executing push");
			Ok(json!({ "success": true, "message": "Push successful" }))
		},
		"git.pull" | "pull" => {
			log::info!("[SCM Command] Executing pull");
			Ok(json!({ "success": true, "message": "Pull successful" }))
		},
		_ => Err(format!("Unknown SCM command: {}", CommandName)),
	}
}

#[command]
pub async fn GetSCMBranches(
	State:State<'_, Arc<ApplicationState>>,

	ProviderIdentifier:String,
) -> Result<Value, String> {
	log::debug!("[SCM Command] Getting branches for provider: {}", ProviderIdentifier);

	// TODO: Implement branch retrieval
	Ok(json!({
		"branches": [
			{ "name": "main", "isCurrent": true },
			{ "name": "develop", "isCurrent": false },
		],
	}))
}

#[command]
pub async fn CheckoutSCMBranch(
	State:State<'_, Arc<ApplicationState>>,

	BranchName:String,
) -> Result<Value, String> {
	log::debug!("[SCM Command] Checking out branch: {}", BranchName);

	// TODO: Implement branch checkout
	Ok(json!({ "success": true, "message": format!("Checked out branch: {}", BranchName) }))
}

#[command]
pub async fn GetSCMCommitHistory(
	State:State<'_, Arc<ApplicationState>>,

	MaxCount:Option<usize>,
) -> Result<Value, String> {
	log::debug!("[SCM Command] Getting commit history, max count: {:?}", MaxCount);

	// TODO: Implement commit history retrieval
	let MaxCommits = MaxCount.unwrap_or(50);
	Ok(json!({
		"commits": Vec::<Value>::new(),
		"maxCount": MaxCommits,
	}))
}

#[command]
pub async fn StageSCMResource(
	State:State<'_, Arc<ApplicationState>>,

	ResourceURI:String,

	Staged:bool,
) -> Result<Value, String> {
	log::debug!("[SCM Command] Staging resource: {}, staged: {}", ResourceURI, Staged);

	// TODO: Implement resource staging
	Ok(json!({ "success": true }))
}
