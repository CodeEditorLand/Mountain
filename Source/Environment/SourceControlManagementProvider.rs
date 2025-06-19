//! # SourceControlManagementProvider Implementation
//!
//! Implements the `SourceControlManagementProvider` trait for the
//! `MountainEnvironment`. This is currently a stub implementation.

use Common::{
	Error::CommonError,
	SourceControlManagement::{
		DTO::{
			SourceControlManagementGroupDTO,
			SourceControlManagementProviderDTO,
			SourceControlManagementResourceDTO,
		},
		SourceControlManagementProvider,
	},
};
use async_trait::async_trait;
use log::warn;

use super::MountainEnvironment;

#[async_trait]
impl SourceControlManagementProvider for MountainEnvironment {
	async fn RegisterSourceControlManagementProvider(
		&self,
		_ProviderData:SourceControlManagementProviderDTO,
	) -> Result<u32, CommonError> {
		warn!("[SCMProvider] RegisterSourceControlManagementProvider is not implemented.");
		// A real implementation would store the provider and return a handle.
		Ok(1)
	}

	async fn UpdateSourceControlManagementGroup(
		&self,
		_ProviderHandle:u32,
		_GroupData:SourceControlManagementGroupDTO,
	) -> Result<(), CommonError> {
		warn!("[SCMProvider] UpdateSourceControlManagementGroup is not implemented.");
		// A real implementation would find the provider and send an update event to the
		// UI.
		Ok(())
	}

	async fn UpdateSourceControlManagementGroupResources(
		&self,
		_ProviderHandle:u32,
		_GroupIdentifier:String,
		_Resources:Vec<SourceControlManagementResourceDTO>,
	) -> Result<(), CommonError> {
		warn!("[SCMProvider] UpdateSourceControlManagementGroupResources is not implemented.");
		Ok(())
	}

	async fn GetInputBoxValue(&self, _ProviderHandle:u32) -> Result<String, CommonError> {
		warn!("[SCMProvider] GetInputBoxValue is not implemented.");
		Ok(String::new())
	}
}
