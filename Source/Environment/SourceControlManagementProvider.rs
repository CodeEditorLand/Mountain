// File: Mountain/Source/Environment/SourceControlManagementProvider.rs

//! # SourceControlManagementProvider Implementation
//!
//! Implements the `SourceControlManagementProvider` trait for the
//! `MountainEnvironment`.

#![allow(non_snake_case, non_camel_case_types)]

use Common::{
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
