// File: Mountain/Source/Environment/SourceControlManagementProvider.rs

//! # SourceControlManagementProvider Implementation
//!
//! Implements the `SourceControlManagementProvider` trait for the
//! `MountainEnvironment`.

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

use super::MountainEnvironment::MountainEnvironment;

#[async_trait]
impl SourceControlManagementProvider for MountainEnvironment {
	async fn CreateSourceControl(&self, ProviderDataValue:Value) -> Result<u32, CommonError> {
		let ProviderData:SourceControlCreateDTO = serde_json::from_value(ProviderDataValue)?;
		let Handle = self.ApplicationState.GetNextScmProviderHandle();
		info!("[SCMProvider] Creating new SCM provider with handle {}", Handle);

		let ProviderState = SourceControlManagementProviderDTO {
			Handle,
			Label:ProviderData.Label,
			RootURI:Some(json!({ "external": ProviderData.RootUri.to_string() })),
			CommitTemplate:None,
			Count:None,
			InputBox:None,
		};

		self.ApplicationState
			.ScmProviders
			.lock()
			.unwrap()
			.insert(Handle, ProviderState.clone());
		self.ApplicationState
			.ScmGroups
			.lock()
			.unwrap()
			.insert(Handle, Default::default());

		self.ApplicationHandle
			.emit("sky://scm/provider/added", ProviderState)
			.map_err(|e| CommonError::UserInterfaceInteraction { Reason:format!("Failed to emit scm event: {}", e) })?;

		Ok(Handle)
	}

	async fn DisposeSourceControl(&self, ProviderHandle:u32) -> Result<(), CommonError> {
		info!("[SCMProvider] Disposing SCM provider with handle {}", ProviderHandle);

		self.ApplicationState.ScmProviders.lock().unwrap().remove(&ProviderHandle);
		self.ApplicationState.ScmGroups.lock().unwrap().remove(&ProviderHandle);

		self.ApplicationHandle
			.emit("sky://scm/provider/removed", ProviderHandle)
			.map_err(|e| CommonError::UserInterfaceInteraction { Reason:e.to_string() })?;
		Ok(())
	}

	async fn UpdateSourceControl(&self, ProviderHandle:u32, UpdateDataValue:Value) -> Result<(), CommonError> {
		let UpdateData:SourceControlUpdateDTO = serde_json::from_value(UpdateDataValue)?;
		info!("[SCMProvider] Updating provider {}", ProviderHandle);

		if let Some(Provider) = self.ApplicationState.ScmProviders.lock().unwrap().get_mut(&ProviderHandle) {
			if let Some(count) = UpdateData.Count {
				Provider.Count = Some(count);
			}
			if let Some(value) = UpdateData.InputBoxValue {
				if let Some(input_box) = &mut Provider.InputBox {
					input_box.Value = value;
				}
			}

			self.ApplicationHandle
				.emit(
					"sky://scm/provider/changed",
					json!({ "handle": ProviderHandle, "provider": Provider }),
				)
				.map_err(|e| CommonError::UserInterfaceInteraction { Reason:e.to_string() })?;
		}
		Ok(())
	}

	async fn UpdateSourceControlGroup(&self, ProviderHandle:u32, GroupDataValue:Value) -> Result<(), CommonError> {
		let GroupData:SourceControlGroupUpdateDTO = serde_json::from_value(GroupDataValue)?;
		info!(
			"[SCMProvider] Updating group '{}' for provider {}",
			GroupData.GroupID, ProviderHandle
		);

		if let Some(ProviderGroups) = self.ApplicationState.ScmGroups.lock().unwrap().get_mut(&ProviderHandle) {
			let Group = ProviderGroups.entry(GroupData.GroupID.clone()).or_insert_with(|| {
				SourceControlManagementGroupDTO {
					ProviderHandle,
					Identifier:GroupData.GroupID.clone(),
					Label:GroupData.Label.clone(),
				}
			});

			Group.Label = GroupData.Label;

			self.ApplicationHandle
				.emit(
					"sky://scm/group/changed",
					json!({ "providerHandle": ProviderHandle, "group": Group }),
				)
				.map_err(|e| CommonError::UserInterfaceInteraction { Reason:e.to_string() })?;
		} else {
			warn!(
				"[SCMProvider] Received group update for unknown provider handle: {}",
				ProviderHandle
			);
		}

		Ok(())
	}

	async fn RegisterInputBox(&self, ProviderHandle:u32, InputBoxDataValue:Value) -> Result<(), CommonError> {
		let InputBoxData:SourceControlInputBoxDTO = serde_json::from_value(InputBoxDataValue)?;
		info!("[SCMProvider] Registering input box for provider {}", ProviderHandle);

		if let Some(Provider) = self.ApplicationState.ScmProviders.lock().unwrap().get_mut(&ProviderHandle) {
			Provider.InputBox = Some(InputBoxData);
			self.ApplicationHandle
				.emit(
					"sky://scm/provider/changed",
					json!({ "handle": ProviderHandle, "provider": Provider }),
				)
				.map_err(|e| CommonError::UserInterfaceInteraction { Reason:e.to_string() })?;
		}
		Ok(())
	}
}
