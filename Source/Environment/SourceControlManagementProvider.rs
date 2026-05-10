//! # SourceControlManagementProvider (Environment)
//!
//! Implements the `SourceControlManagementProvider` trait for the
//! `MountainEnvironment`.
//!
//! ## SCM provider architecture
//!
//! Each SCM provider maintains:
//! - **Handle** — unique `u32` identifier; callers may supply their own so
//!   the same handle key used in `ScmNamespace.ts` maps correctly on both
//!   sides of the IPC boundary.
//! - **Label** — user-friendly name (e.g., "Git")
//! - **Root URI** — URI of the repository root
//! - **Groups** — resource groups organizing changed resources
//! - **Input box** — user input widget (e.g., commit messages)
//! - **Count** — badge count for changed items
//!
//! ## Resource groups
//!
//! Groups organize resources by their state:
//! - **Changes** — modified files ready to commit
//! - **Untracked** — new files not yet tracked
//! - **Staged** — files staged for commit
//! - **Merge changes** — files with merge conflicts
//! - **Conflict unresolved** — unresolved conflict markers
//!
//! ## SCM lifecycle
//!
//! 1. **CreateSourceControl** — register provider, emit `SCMProviderAdded`
//! 2. **UpdateSourceControl** — update badge/input-box, emit
//!    `SCMProviderChanged`
//! 3. **UpdateSourceControlGroup** — upsert group entry, emit
//!    `SCMGroupChanged`
//! 4. **RegisterInputBox** — attach input-box DTO to provider
//! 5. **DisposeSourceControl** — remove provider + groups, emit
//!    `SCMProviderRemoved`
//!
//! ## Git integration patterns
//!
//! Typical Git provider workflow:
//! - Detect `.git` directory, run `git status` to populate groups
//! - Run `git diff` for file diffs; use input box for commit messages
//! - Show badge count for changed files
//! - Provide commands: Stage, Unstage, Commit, Push, Pull, Discard
//!
//! ## VS Code reference
//!
//! - `vs/workbench/services/scm/common/scmService.ts`
//! - `vs/platform/scm/common/scm.ts`
//! - `vs/sourcecontrol/git/common/git.ts`

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

// TODO: built-in Git provider (libgit2 or CLI), repository discovery +
// change detection, staging/unstaging/committing, branch management UI,
// remote ops (push/pull/fetch), merge-conflict UI, Git LFS + submodules,
// credential management, SCM extensions API, history/blame views, stash/pop,
// tag management, detached HEAD / bisect, rebase / cherry-pick, telemetry.
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
			.map_err(Utility::ErrorMapping::MapApplicationStateLockErrorToCommonError)?
		;

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
