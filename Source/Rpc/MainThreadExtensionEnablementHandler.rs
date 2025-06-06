// File: Rpc/MainThreadExtensionEnablementHandler.rs
// Defines the RPC handler for requests related to extension enablement states.

use std::sync::Arc;

use Common::{Errors::CommonError, Runtime::AppRuntimeTrait};
use log::{debug, info, warn};
use serde_json::{Value, json};
use tauri::{AppHandle, Manager, State, Wry};
use vs_platform_extensions_common_extensions::{
	EnablementState as VsEnablementState,
	ExtensionIdentifier as VsExtensionIdentifier,
}; // Assuming this is the correct path after PascalCasing

use crate::Handlers::{self, ErrorUtils}; // Handlers::Enablement will contain the logic
use crate::{
	Rpc::Argument::Enablement::{GetEnablementStateArgument, SetEnablementArgument},
	Runtime::AppRuntime,
};

#[derive(Clone)]
pub struct MainThreadExtensionEnablementHandler {
	pub ApplicationHandle:AppHandle<Wry>,
	// Runtime might not be strictly needed if all logic is in Handlers::Enablement
	// and Handlers::Enablement uses effects or direct AppState access.
	// pub Runtime: Arc<AppRuntime>,
}

impl MainThreadExtensionEnablementHandler {
	pub fn New(ApplicationHandle:AppHandle<Wry> /* , Runtime: Arc<AppRuntime> */) -> Self {
		Self { ApplicationHandle /* , Runtime */ }
	}

	/// Retrieves the enablement state of a specific extension.
	pub async fn GetEnablementState(&self, Argument:GetEnablementStateArgument) -> Result<Value, String> {
		// Deserialize the extensionIdDto into VsExtensionIdentifier
		let ExtensionIdToQuery:VsExtensionIdentifier = serde_json::from_value(Argument.ExtensionIdentifierDto.clone())
			.map_err(|Error| {
				ErrorUtils::RpcParamErrorString(
					"GetEnablementState",
					"ExtensionIdentifierDto",
					"valid IExtensionIdentifier DTO",
					Some(0),
				)
			})?;

		info!(
			"[Rpc EnablementHandler] GetEnablementState (DTO): ExtensionIdentifier='{}'",
			ExtensionIdToQuery.Value
		);

		// Delegate to the logic handler
		Handlers::Enablement::HandleGetEnablementStateLogic(self.ApplicationHandle.clone(), ExtensionIdToQuery).await
	}

	/// Sets the enablement state for one or more extensions.
	pub async fn SetEnablement(&self, Argument:SetEnablementArgument) -> Result<Value, String> {
		let ExtensionIdentifierDtosVec = Argument.ExtensionIdentifierDtos.as_array().ok_or_else(|| {
			ErrorUtils::RpcParamErrorString(
				"SetEnablement",
				"ExtensionIdentifierDtos",
				"array of IExtensionIdentifier DTOs",
				Some(0),
			)
		})?;

		let mut ExtensionsToUpdate:Vec<VsExtensionIdentifier> = Vec::new();
		for DtoValue in ExtensionIdentifierDtosVec {
			let ExtensionId:VsExtensionIdentifier = serde_json::from_value(DtoValue.clone()).map_err(|Error| {
				ErrorUtils::RpcErrorString(
					format!(
						"Invalid IExtensionIdentifier DTO in list for SetEnablement: {}. Value: {:?}",
						Error, DtoValue
					),
					Some("EBADARG_EXTID_IN_LIST_DTO"),
				)
			})?;
			ExtensionsToUpdate.push(ExtensionId);
		}

		let NewStateEnum = VsEnablementState::from_u32(Argument.NewState).ok_or_else(|| {
			ErrorUtils::RpcParamErrorString("SetEnablement", "NewState", "valid EnablementState u32", Some(1))
		})?;

		info!(
			"[Rpc EnablementHandler] SetEnablement (DTO): ExtensionCount={}, NewState={:?}",
			ExtensionsToUpdate.len(),
			NewStateEnum
		);

		// Delegate to the logic handler
		Handlers::Enablement::HandleSetEnablementLogic(self.ApplicationHandle.clone(), ExtensionsToUpdate, NewStateEnum)
			.await
	}
}
