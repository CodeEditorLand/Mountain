//! # SourceControlManagementProvider (Environment)
//!
//! Implements the `SourceControlManagementProvider` trait for
//! `MountainEnvironment`, providing Git and other source control management
//! (SCM) capabilities to the application.
//!
//! ## RESPONSIBILITIES
//!
//! ### 1. Repository Detection
//! - Scan workspace folders for Git repositories
//! - Detect repository root directories
//! - Track repository state (clean, dirty, branching)
//! - Monitor repository changes (commit, checkout, merge)
//!
//! ### 2. SCM Providers
//! - Create and manage `SourceControlManagementProvider` instances
//! - Support multiple SCM systems (Git, Mercurial, etc.)
//! - Load extension-provided SCM providers
//! - Route SCM operations to appropriate provider
//!
//! ### 3. Change Management
//! - Track file changes (modified, added, deleted, renamed)
//! - Provide diff information for changed files
//! - Support staging and unstaging changes
//! - Handle merge conflicts
//!
//! ### 4. Authentication
//! - Manage SCM credentials and authentication
//! - Support SSH keys and HTTPS tokens
//! - Store credentials securely via `SecretProvider`
//! - Handle authentication failures and prompts
//!
//! ### 5. Operations
//! - Commit, push, pull, fetch operations
//! - Branch management (create, delete, rename, checkout)
//! - Merge and rebase operations
//! - Remote management (add, remove, rename)
//!
//! ## ARCHITECTURAL ROLE
//!
//! SourceControlManagementProvider is the **SCM integration layer**:
//!
//! ```text
//! UI (SCM View) ──► SourceControlManagementProvider ──► Git CLI / Libgit2
//!                              │
//!                              └─► Extension SCM Providers
//! ```
//!
//! ### Position in Mountain
//! - `Environment` module: SCM capability provider
//! - Implements
//!   `CommonLibrary::SourceControlManagement::SourceControlManagementProvider`
//!   trait
//! - Accessible via `Environment.Require<dyn
//!   SourceControlManagementProvider>()`
//!
//! ### SCM Provider Hierarchy
//! - **Built-in Git Provider**: Native Git implementation (preferred)
//! - **Extension Providers**: Custom SCM support (Mercurial, SVN, etc.)
//! - **Fallback Providers**: Basic functionality for unknown SCM types
//!
//! ### Dependencies
//! - `SecretProvider`: For storing SCM credentials
//! - `FileSystemReader` / `FileSystemWriter`: For .git operations
//! - `Log`: SCM operation logging
//! - External Git binary or libgit2 library
//!
//! ### Dependents
//! - SCM UI view: Display repository state and changes
//! - Source control commands: Commit, push, pull, etc.
//! - `Binary::Main`::`MountainGetWorkbenchConfiguration`: SCM state
//! - Extension SCM providers: Custom SCM implementations
//!
//! ## DATA MODEL
//!
//! Stored in `ApplicationState`:
//! - `SourceControlManagementProviders`: Registered providers by ID
//! - `SourceControlManagementGroups`: Repository groups (by workspace)
//! - `SourceControlManagementResources`: Resource state (changed files)
//!
//! Key structures:
//! - `SourceControlManagementProviderDTO`: Provider metadata
//! - `SourceControlManagementGroupDTO`: Repository group state
//! - `SourceControlManagementResourceDTO`: Changed file information
//!
//! ## REPOSITORY STATES
//!
//! - **Clean**: No uncommitted changes
//! - **Dirty**: Unstaged changes present
//! - **Staged**: Changes staged for commit
//! - **Merging**: Merge in progress
//! - **Rebasing**: Rebase in progress
//! - **Cherry-picking**: Cherry-pick in progress
//!
//! ## ERROR HANDLING
//!
//! - Repository not found: `CommonError::SCMNotFound`
//! - Authentication failure: `CommonError::SCMAuthenticationFailed`
//! - Operation failure: `CommonError::SCMOperationFailed`
//! - Merge conflict: `CommonError::SCMConflict`
//! - Uncommitted changes: `CommonError::SCMUncommittedChanges`
//!
//! ## PERFORMANCE
//!
//! - Repository scanning should be async and cached
//! - Use file system watchers to detect changes
//! - Batch operations when possible (e.g., status of multiple files)
//! - Consider background indexing for large repositories
//!
//! ## VS CODE REFERENCE
//!
//! Patterns from VS Code:
//! - `vs/workbench/services/scm/common/scmService.ts` - SCM service
//! - `vs/platform/scm/common/scm.ts` - SCM provider interface
//! - `vs/sourcecontrol/git/common/git.ts` - Git provider implementation
//!
//! ## TODO
//!
//! - [ ] Implement built-in Git provider using libgit2 or Git CLI
//! - [ ] Add repository discovery and change detection
//! - [ ] Support staging, unstaging, and committing changes
//! - [ ] Implement branch management UI and operations
//! - [ ] Add remote operations (push, pull, fetch)
//! - [ ] Handle merge conflicts with UI resolution
//! - [ ] Support Git LFS and submodules
//! - [ ] Add SCM authentication and credential management
//! - [ ] Implement SCM extensions API for custom providers
//! - [ ] Add SCM history and blame views
//! - [ ] Support stash and pop operations
//! - [ ] Implement tag management
//! - [ ] Add SCM configuration and settings
//! - [ ] Support detached HEAD and bisect operations
//! - [ ] Implement SCM telemetry and diagnostics
//!
//! ## MODULE CONTENTS
//!
//! - [`SourceControlManagementProvider`]: Main struct implementing the trait
//! - Repository detection and tracking
//! - Provider registration and routing
//! - SCM operation implementations
//! - Authentication and credential management

// `MountainEnvironment`. Responsibilities:
//   - Manage source control providers (e.g., Git, Mercurial, SVN).
//   - Handle SCM provider registration and disposal.
//   - Manage resource groups (e.g., changes, untracked, merge conflicts).
//   - Handle input boxes for user input (e.g., commit messages).
//   - Emit events to the Sky frontend for UI updates.
//   - Provide Git integration patterns for common operations.
//   - Handle conflict detection and resolution.
//   - Support multiple SCM providers simultaneously.
//
// TODOs:
//   - Implement complete Git integration (status, commit, push, pull, branch)
//   - Add Git diff display with visual comparison
//   - Implement merge conflict resolution UI
//   - Support Git staging/unstaging of resources
//   - Add Git stash operations
//   - Implement Git branch management (create, delete, checkout)
//   - Support Git remote operations
//   - Add Git history/log viewing
//   - Implement Git blame annotations
//   - Support Git submodules
//   - Implement Git LFS (Large File Storage) support
//   - Add Git tag management
//   - Custom implementation for Mercurial, SVN, and other VCS
//   - Implement SCM provider command registration
//   - Support SCM provider decoration (badges, colors)
//   - Add input box validation and validation messaging
//   - Implement resource state caching for performance
//   - Support SCM provider quick picks and menus
//   - Add keyboard shortcuts for common SCM operations
//   - Implement SCM provider extension points
//   - Support Git rebase and cherry-pick operations
//   - Add Git bisect support
//   - Implement Git commit graph visualization
//   - Support Git hooks integration
//   - Add Git signature verification
//   - Implement Git ignore management
//
// Inspired by VSCode's SCM service which:
// - Provides a flexible abstraction over multiple source control systems
// - Manages resource state changes through groups
// - Supports provider-specific operations through commands
// - Handles UI updates through event emission
// - Manages input boxes for user interaction
// - Git integration is the primary implementation with patterns for others
//! # SourceControlManagementProvider Implementation
//!
//! Implements the `SourceControlManagementProvider` trait for the
//! `MountainEnvironment`.
//!
//! ## SCM Provider Architecture
//!
//! Each SCM provider maintains:
//! - **Handle**: Unique identifier for the provider
//! - **Label**: User-friendly name (e.g., "Git")
//! - **Root URI**: URI of the repository root
//! - **Groups**: Resource groups organizing changed resources
//! - **Input Box**: User input widget for operations (e.g., commit messages)
//! - **Count**: Badge count for changed items
//!
//! ## Resource Groups
//!
//! Groups organize resources by their state:
//! - **Changes**: Modified files ready to commit
//! - **Untracked**: New files not yet tracked
//! - **Staged**: Files staged for commit
//! - **Merge Changes**: Files with merge conflicts
//! - **Conflict Unresolved**: Unresolved conflict markers
//
//! ## SCM Lifecycle
//!
//! 1. **Create Provider**: Register a new SCM provider with handle and metadata
//! 2. **Update Provider**: Update provider state (badge count, input box)
//! 3. **Update Group**: Add or remove resources from groups
//! 4. **Register Input Box**: Create input widget for user interaction
//! 5. **Dispose Provider**: Remove provider and all associated state
//
//! ## Git Integration Patterns
//!
//! Typical Git provider workflow:
//! - Detect Git repository via `.git` directory
//! - Run `git status` to populate resource groups
//! - Run `git diff` to provide file diffs
//! - Use input box for commit messages
//! - Show badge count for changed files
//! - Provide commands: Stage, Unstage, Commit, Push, Pull, Discard

use CommonLibrary::{
	Error::CommonError::CommonError,
	IPC::SkyEvent::SkyEvent,
	SourceControlManagement::{
		DTO::{
			SourceControlCreateDTO::SourceControlCreateDTO,
			SourceControlGroupUpdateDTO::SourceControlGroupUpdateDTO,
			SourceControlInputBoxDTO::SourceControlInputBoxDTO,
			SourceControlManagementGroupDTO::SourceControlManagementGroupDTO,
			SourceControlManagementProviderDTO::SourceControlManagementProviderDTO,
			SourceControlUpdateDTO::SourceControlUpdateDTO,
		},
		SourceControlManagementProvider::SourceControlManagementProvider,
	},
};
use async_trait::async_trait;
use serde_json::{Value, json};
use tauri::Emitter;

use super::{MountainEnvironment::MountainEnvironment, Utility};
use crate::dev_log;

#[async_trait]
impl SourceControlManagementProvider for MountainEnvironment {
	async fn CreateSourceControl(&self, ProviderDataValue:Value) -> Result<u32, CommonError> {
		let ProviderData:SourceControlCreateDTO = serde_json::from_value(ProviderDataValue)?;

		// Honor caller-supplied handle when present so the marker maps
		// (`SourceControlManagementProviders` / `SourceControlManagementGroups`)
		// key under the SAME identifier Cocoon's `ScmNamespace.ts` uses
		// for subsequent `register_scm_resource_group` and `update_scm_group`
		// notifications. Without this, `UpdateSourceControlGroup` looks up
		// Cocoon's handle in a map keyed by a Mountain-allocated handle,
		// the entry isn't there, and every group update warns
		// "Received group update for unknown provider handle: <H>" while
		// the SCM viewlet stays empty.
		let Handle = ProviderData
			.Handle
			.unwrap_or_else(|| self.ApplicationState.GetNextSourceControlManagementProviderHandle());

		dev_log!(
			"extensions",
			"[SourceControlManagementProvider] Creating new SCM provider with handle {}",
			Handle
		);

		let ProviderState = SourceControlManagementProviderDTO {
			Handle,

			Identifier:ProviderData.ID.clone(),

			Label:ProviderData.Label,

			RootURI:Some(json!({ "external": ProviderData.RootUri.to_string() })),

			CommitTemplate:None,

			Count:None,

			InputBox:None,
		};

		self.ApplicationState
			.Feature
			.Markers
			.SourceControlManagementProviders
			.lock()
			.map_err(Utility::ErrorMapping::MapApplicationStateLockErrorToCommonError)?
			.insert(Handle, ProviderState.clone());

		self.ApplicationState
			.Feature
			.Markers
			.SourceControlManagementGroups
			.lock()
			.map_err(Utility::ErrorMapping::MapApplicationStateLockErrorToCommonError)?
			.insert(Handle, Default::default());

		self.ApplicationHandle
			.emit(SkyEvent::SCMProviderAdded.AsStr(), ProviderState)
			.map_err(|Error| {
				CommonError::UserInterfaceInteraction { Reason:format!("Failed to emit scm event: {}", Error) }
			})?;

		Ok(Handle)
	}

	async fn DisposeSourceControl(&self, ProviderHandle:u32) -> Result<(), CommonError> {
		dev_log!(
			"extensions",
			"[SourceControlManagementProvider] Disposing SCM provider with handle {}",
			ProviderHandle
		);

		self.ApplicationState
			.Feature
			.Markers
			.SourceControlManagementProviders
			.lock()
			.map_err(Utility::ErrorMapping::MapApplicationStateLockErrorToCommonError)?
			.remove(&ProviderHandle);

		self.ApplicationState
			.Feature
			.Markers
			.SourceControlManagementGroups
			.lock()
			.map_err(Utility::ErrorMapping::MapApplicationStateLockErrorToCommonError)?
			.remove(&ProviderHandle);

		self.ApplicationHandle
			.emit(SkyEvent::SCMProviderRemoved.AsStr(), ProviderHandle)
			.map_err(|Error| CommonError::UserInterfaceInteraction { Reason:Error.to_string() })?;

		Ok(())
	}

	async fn UpdateSourceControl(&self, ProviderHandle:u32, UpdateDataValue:Value) -> Result<(), CommonError> {
		let UpdateData:SourceControlUpdateDTO = serde_json::from_value(UpdateDataValue)?;

		dev_log!(
			"extensions",
			"[SourceControlManagementProvider] Updating provider {}",
			ProviderHandle
		);

		let mut ProvidersGuard = self
			.ApplicationState
			.Feature
			.Markers
			.SourceControlManagementProviders
			.lock()
			.map_err(Utility::ErrorMapping::MapApplicationStateLockErrorToCommonError)?;

		if let Some(Provider) = ProvidersGuard.get_mut(&ProviderHandle) {
			if let Some(count) = UpdateData.Count {
				Provider.Count = Some(count);
			}

			if let Some(value) = UpdateData.InputBoxValue {
				if let Some(input_box) = &mut Provider.InputBox {
					input_box.Value = value;
				}
			}

			let ProviderClone = Provider.clone();

			// Release lock before emitting
			drop(ProvidersGuard);

			self.ApplicationHandle
				.emit(
					SkyEvent::SCMProviderChanged.AsStr(),
					json!({ "handle": ProviderHandle, "provider": ProviderClone }),
				)
				.map_err(|Error| CommonError::UserInterfaceInteraction { Reason:Error.to_string() })?;
		}

		Ok(())
	}

	async fn UpdateSourceControlGroup(&self, ProviderHandle:u32, GroupDataValue:Value) -> Result<(), CommonError> {
		let GroupData:SourceControlGroupUpdateDTO = serde_json::from_value(GroupDataValue)?;

		dev_log!(
			"extensions",
			"[SourceControlManagementProvider] Updating group '{}' for provider {}",
			GroupData.GroupID,
			ProviderHandle
		);

		let mut GroupsGuard = self
			.ApplicationState
			.Feature
			.Markers
			.SourceControlManagementGroups
			.lock()
			.map_err(Utility::ErrorMapping::MapApplicationStateLockErrorToCommonError)?;

		if let Some(ProviderGroups) = GroupsGuard.get_mut(&ProviderHandle) {
			let Group = ProviderGroups.entry(GroupData.GroupID.clone()).or_insert_with(|| {
				SourceControlManagementGroupDTO {
					ProviderHandle,
					Identifier:GroupData.GroupID.clone(),
					Label:GroupData.Label.clone(),
				}
			});

			Group.Label = GroupData.Label;

			let GroupClone = Group.clone();

			// Release lock before emitting
			drop(GroupsGuard);

			self.ApplicationHandle
				.emit(
					SkyEvent::SCMGroupChanged.AsStr(),
					json!({ "providerHandle": ProviderHandle, "group": GroupClone }),
				)
				.map_err(|Error| CommonError::UserInterfaceInteraction { Reason:Error.to_string() })?;
		} else {
			dev_log!(
				"extensions",
				"warn: [SourceControlManagementProvider] Received group update for unknown provider handle: {}",
				ProviderHandle
			);
		}

		Ok(())
	}

	async fn RegisterInputBox(&self, ProviderHandle:u32, InputBoxDataValue:Value) -> Result<(), CommonError> {
		let InputBoxData:SourceControlInputBoxDTO = serde_json::from_value(InputBoxDataValue)?;

		dev_log!(
			"extensions",
			"[SourceControlManagementProvider] Registering input box for provider {}",
			ProviderHandle
		);

		let mut ProvidersGuard = self
			.ApplicationState
			.Feature
			.Markers
			.SourceControlManagementProviders
			.lock()
			.map_err(Utility::ErrorMapping::MapApplicationStateLockErrorToCommonError)?;

		if let Some(Provider) = ProvidersGuard.get_mut(&ProviderHandle) {
			Provider.InputBox = Some(InputBoxData);

			let ProviderClone = Provider.clone();

			// Release lock before emitting
			drop(ProvidersGuard);

			self.ApplicationHandle
				.emit(
					SkyEvent::SCMProviderChanged.AsStr(),
					json!({ "handle": ProviderHandle, "provider": ProviderClone }),
				)
				.map_err(|Error| CommonError::UserInterfaceInteraction { Reason:Error.to_string() })?;
		}

		Ok(())
	}
}
