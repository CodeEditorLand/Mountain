// File: Mountain/Source/Environment/SourceControlManagementProvider.rs
// Role: Implements the `SourceControlManagementProvider` trait for the
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
use log::{info, warn};
use serde_json::{Value, json};
use tauri::Emitter;

use super::{MountainEnvironment::MountainEnvironment, Utility};

#[async_trait]
impl SourceControlManagementProvider for MountainEnvironment {
	async fn CreateSourceControl(&self, ProviderDataValue:Value) -> Result<u32, CommonError> {
		let ProviderData:SourceControlCreateDTO = serde_json::from_value(ProviderDataValue)?;

		let Handle = self.ApplicationState.GetNextSourceControlManagementProviderHandle();

		info!(
			"[SourceControlManagementProvider] Creating new SCM provider with handle {}",
			Handle
		);

		let ProviderState = SourceControlManagementProviderDTO {
			Handle,
			Label:ProviderData.Label,
			RootURI:Some(json!({ "external": ProviderData.RootUri.to_string() })),
			CommitTemplate:None,
			Count:None,
			InputBox:None,
		};

		self.ApplicationState
			.SourceControlManagementProviders
			.lock()
			.map_err(Utility::MapApplicationStateLockErrorToCommonError)?
			.insert(Handle, ProviderState.clone());

		self.ApplicationState
			.SourceControlManagementGroups
			.lock()
			.map_err(Utility::MapApplicationStateLockErrorToCommonError)?
			.insert(Handle, Default::default());

		self.ApplicationHandle
			.emit("sky://scm/provider/added", ProviderState)
			.map_err(|Error| {
				CommonError::UserInterfaceInteraction { Reason:format!("Failed to emit scm event: {}", Error) }
			})?;

		Ok(Handle)
	}

	async fn DisposeSourceControl(&self, ProviderHandle:u32) -> Result<(), CommonError> {
		info!(
			"[SourceControlManagementProvider] Disposing SCM provider with handle {}",
			ProviderHandle
		);

		self.ApplicationState
			.SourceControlManagementProviders
			.lock()
			.map_err(Utility::MapApplicationStateLockErrorToCommonError)?
			.remove(&ProviderHandle);

		self.ApplicationState
			.SourceControlManagementGroups
			.lock()
			.map_err(Utility::MapApplicationStateLockErrorToCommonError)?
			.remove(&ProviderHandle);

		self.ApplicationHandle
			.emit("sky://scm/provider/removed", ProviderHandle)
			.map_err(|Error| CommonError::UserInterfaceInteraction { Reason:Error.to_string() })?;

		Ok(())
	}

	async fn UpdateSourceControl(&self, ProviderHandle:u32, UpdateDataValue:Value) -> Result<(), CommonError> {
		let UpdateData:SourceControlUpdateDTO = serde_json::from_value(UpdateDataValue)?;

		info!("[SourceControlManagementProvider] Updating provider {}", ProviderHandle);

		let mut ProvidersGuard = self
			.ApplicationState
			.SourceControlManagementProviders
			.lock()
			.map_err(Utility::MapApplicationStateLockErrorToCommonError)?;

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
					"sky://scm/provider/changed",
					json!({ "handle": ProviderHandle, "provider": ProviderClone }),
				)
				.map_err(|Error| CommonError::UserInterfaceInteraction { Reason:Error.to_string() })?;
		}

		Ok(())
	}

	async fn UpdateSourceControlGroup(&self, ProviderHandle:u32, GroupDataValue:Value) -> Result<(), CommonError> {
		let GroupData:SourceControlGroupUpdateDTO = serde_json::from_value(GroupDataValue)?;

		info!(
			"[SourceControlManagementProvider] Updating group '{}' for provider {}",
			GroupData.GroupID, ProviderHandle
		);

		let mut GroupsGuard = self
			.ApplicationState
			.SourceControlManagementGroups
			.lock()
			.map_err(Utility::MapApplicationStateLockErrorToCommonError)?;

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
					"sky://scm/group/changed",
					json!({ "providerHandle": ProviderHandle, "group": GroupClone }),
				)
				.map_err(|Error| CommonError::UserInterfaceInteraction { Reason:Error.to_string() })?;
		} else {
			warn!(
				"[SourceControlManagementProvider] Received group update for unknown provider handle: {}",
				ProviderHandle
			);
		}

		Ok(())
	}

	async fn RegisterInputBox(&self, ProviderHandle:u32, InputBoxDataValue:Value) -> Result<(), CommonError> {
		let InputBoxData:SourceControlInputBoxDTO = serde_json::from_value(InputBoxDataValue)?;

		info!(
			"[SourceControlManagementProvider] Registering input box for provider {}",
			ProviderHandle
		);

		let mut ProvidersGuard = self
			.ApplicationState
			.SourceControlManagementProviders
			.lock()
			.map_err(Utility::MapApplicationStateLockErrorToCommonError)?;

		if let Some(Provider) = ProvidersGuard.get_mut(&ProviderHandle) {
			Provider.InputBox = Some(InputBoxData);

			let ProviderClone = Provider.clone();

			// Release lock before emitting
			drop(ProvidersGuard);

			self.ApplicationHandle
				.emit(
					"sky://scm/provider/changed",
					json!({ "handle": ProviderHandle, "provider": ProviderClone }),
				)
				.map_err(|Error| CommonError::UserInterfaceInteraction { Reason:Error.to_string() })?;
		}

		Ok(())
	}
}
