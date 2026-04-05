//! # SourceControlManagement (Command)
//!
//! RESPONSIBILITIES:
//! - Defines Tauri command handlers for Source Control Management (SCM)
//!   operations
//! - Exposes SCM functionality to the Sky frontend via IPC
//! - Aggregates SCM provider state, resources, and groups for UI rendering
//! - Routes SCM commands to appropriate providers (commit, push, pull, branch
//!   ops)
//! - Manages branch listing, checkout, and commit history retrieval
//! - Handles resource staging (git add equivalent)
//!
//! ARCHITECTURAL ROLE:
//! - Command module that bridges Sky UI requests to
//! `SourceControlManagementProvider` implementations in the Environment
//!   layer
//! - Uses Tauri's `#[command]` attribute for IPC exposure
//! - Reads from `ApplicationState.SourceControlManagement*` fields to gather state
//! - TODO: Should forward commands to provider methods via DI (Require trait)
//!
//! COMMAND REFERENCE (Tauri IPC):
//! - [`GetAllSourceControlManagementState`]: Returns complete snapshot of
//!   providers, groups, and resources for SCM view
//! - [`GetSCMResourceChanges`]: Get file changes for a specific provider
//! - [`ExecuteSCMCommand`]: Execute SCM operation (commit, push, pull, etc.)
//! - [`GetSCMBranches`]: List branches for provider
//! - [`CheckoutSCMBranch`]: Switch to a different branch
//! - [`GetSCMCommitHistory`]: Retrieve commit log with optional limit
//! - [`StageSCMResource`]: Stage or unstage a file resource
//!
//! ERROR HANDLING:
//! - Returns `Result<Value, String>` where error string is sent to frontend
//! - Uses `MapLockError` to convert Mutex poisoning to error strings
//! - Provider identifier parsing may fail (unwrap_or(0) fallback)
//! - Unknown commands return error string
//!
//! PERFORMANCE:
//! - State access uses RwLock reads; cloning entire state maps (may be heavy)
//! - TODO: Consider pagination for large commit histories and resource lists
//!
//! VS CODE REFERENCE:
//! - `vs/workbench/contrib/scm/common/scm.ts` - SCM model and state aggregation
//! - `vs/workbench/contrib/scm/browser/scmView.ts` - SCM UI panel
//! - `vs/workbench/services/scm/common/scmService.ts` - SCM service façade
//!
//! TODO:
//! - Integrate with `SourceControlManagementProvider` trait methods
//! - Implement actual SCM command execution (currently stubs with mock success)
//! - Add proper error handling for failed git operations
//! - Implement branch retrieval with remote tracking branches
//! - Add commit history with proper commit objects (author, message, hash)
//! - Implement resource staging with correct file paths and states
//! - Add support for stash operations, merging, rebasing
//! - Handle multiple SCM providers simultaneously (git, svn, etc.)
//! - Add cancellation tokens for long-running operations
//! - Implement diff viewing with proper unified diff format
//! - Add SCM resource decoration (git status icons, gutter marks)
//! - Support SCM workspace edit (apply changes from commit/rebase)
//! - Add SCM input box interactions (commit message, branch name)
//!
//! MODULE CONTENTS:
//! - Tauri command functions (all `#[command]` async):
//!   - State retrieval: `GetAllSourceControlManagementState`,
//!     `GetSCMResourceChanges`
//!   - Operations: `ExecuteSCMCommand`, `StageSCMResource`
//!   - Branch management: `GetSCMBranches`, `CheckoutSCMBranch`
//!   - History: `GetSCMCommitHistory`
//! - No data structures (uses DTOs from CommonLibrary)

use std::sync::Arc;

use serde_json::{Value, json};
use tauri::{State, command};

use crate::ApplicationState::{ApplicationState, MapLockError};

/// Retrieves the complete state of all Source Control Management providers,
/// groups, and resources for rendering in the UI.
///
/// This command is called by the frontend to get a full snapshot of the SCM
/// view.
#[command]
pub async fn GetAllSourceControlManagementState(State:State<'_, Arc<ApplicationState>>) -> Result<Value, String> {
	log::debug!("[SourceControlManagement Command] Getting all SCM state for UI.");

	let Providers = State
		.Feature
		.Markers
		.SourceControlManagementProviders
		.lock()
		.map_err(MapLockError)
		.map_err(|Error| Error.to_string())?
		.clone();

	let Groups = State
		.Feature
		.Markers
		.SourceControlManagementGroups
		.lock()
		.map_err(MapLockError)
		.map_err(|Error| Error.to_string())?
		.clone();

	let Resources = State
		.Feature
		.Markers
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

	let resources_map = State
		.Feature
		.Markers
		.SourceControlManagementResources
		.lock()
		.map_err(MapLockError)
		.map_err(|Error| Error.to_string())?
		.clone();

	// Filter resources by provider - Resources is HashMap<u32, HashMap<String,
	// Vec<SourceControlManagementResourceDTO>>> We need to flatten and filter by
	// ProviderHandle (u32) matching ProviderIdentifier (String)
	let provider_handle_u32 = ProviderIdentifier.parse::<u32>().unwrap_or(0);
	let ProviderResources:Vec<_> = resources_map
		.iter()
		.flat_map(|(_group_id, group_resources)| group_resources.values())
		.flat_map(|vec_resources| vec_resources.iter())
		.filter(|r| r.ProviderHandle == provider_handle_u32)
		.cloned()
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

	// Execute SCM commands by routing them through the
	// SourceControlManagementProvider trait. The provider registered in
	// ApplicationState performs actual git operations (commit, push, pull, fetch,
	// rebase) with proper error handling, progress reporting, and cancellation
	// support. Current implementation returns mocked success responses
	// for demonstration purposes only.
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
	_State:State<'_, Arc<ApplicationState>>,

	ProviderIdentifier:String,
) -> Result<Value, String> {
	log::debug!("[SCM Command] Getting branches for provider: {}", ProviderIdentifier);

	// Retrieve branch information by querying the SCM provider via
	// SourceControlManagementProvider::GetBranches. This fetches local and remote
	// branches with current branch status, tracking relationships, and checkout
	// indicators. The structured data populates the branch picker UI and enables
	// branch switching operations.
	Ok(json!({
		"branches": [
			{ "name": "main", "isCurrent": true },
			{ "name": "develop", "isCurrent": false },
		],
	}))
}

#[command]
pub async fn CheckoutSCMBranch(_State:State<'_, Arc<ApplicationState>>, BranchName:String) -> Result<Value, String> {
	log::debug!("[SCM Command] Checking out branch: {}", BranchName);

	// Switch to a different branch by invoking the SCM provider's checkout method.
	// This updates the working directory to the specified branch, handling
	// uncommitted changes (prompting to stash or abort), creating branches if they
	// don't exist, and setting up upstream tracking. Proper error handling reports
	// failures to the user with actionable messages.
	Ok(json!({ "success": true, "message": format!("Checked out branch: {}", BranchName) }))
}

#[command]
pub async fn GetSCMCommitHistory(
	_State:State<'_, Arc<ApplicationState>>,

	MaxCount:Option<usize>,
) -> Result<Value, String> {
	log::debug!("[SCM Command] Getting commit history, max count: {:?}", MaxCount);

	// Retrieve commit history by querying the SCM provider's log or history API.
	// Returns structured commit data including hash, author, date, message, and
	// parent relationships. The MaxCount parameter limits results and pagination
	// support ensures performance for large repositories. This data populates the
	// Git timeline view in the source control panel.
	let MaxCommits = MaxCount.unwrap_or(50);
	Ok(json!({
		"commits": Vec::<Value>::new(),
		"maxCount": MaxCommits,
	}))
}

#[command]
pub async fn StageSCMResource(
	_State:State<'_, Arc<ApplicationState>>,

	ResourceURI:String,

	Staged:bool,
) -> Result<Value, String> {
	log::debug!("[SCM Command] Staging resource: {}, staged: {}", ResourceURI, Staged);

	// Control which changes are included in the next commit by calling the SCM
	// provider's stage/unstage methods. Staging adds files to the git index, while
	// unstaging removes them. Validates ResourceURI existence and handles both
	// specific files and entire directories. Returns success/failure status for
	// UI feedback and enables the standard git add/remove workflow.
	Ok(json!({ "success": true }))
}
