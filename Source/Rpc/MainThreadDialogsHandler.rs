// File: Rpc/MainThreadDialogsHandler.rs
// Defines the RPC handler for requests from the sidecar to show native
// file open or save dialogs.

use std::{path::PathBuf, sync::Arc};

use Common::{
	Errors::CommonError,
	Runtime::AppRuntimeTrait,
	UiEffect::{self, OpenDialogOptions as CommonOpenDialogOptions, SaveDialogOptions as CommonSaveDialogOptions},
};
use log::{debug, info, trace};
use serde_json::{Value, json};
use tauri::{AppHandle, Manager, State, Wry};

use crate::{Handlers::ErrorUtils, Rpc::file_path_to_uri_components_dto, Runtime::AppRuntime}; // Assuming this helper is moved to Rpc module's scope or imported

#[derive(Clone)]
pub struct MainThreadDialogsHandler {
	pub ApplicationHandle:AppHandle<Wry>,
	pub Runtime:Arc<AppRuntime>,
}

impl MainThreadDialogsHandler {
	pub fn New(ApplicationHandle:AppHandle<Wry>, Runtime:Arc<AppRuntime>) -> Self {
		Self { ApplicationHandle, Runtime }
	}

	/// Shows a native file open dialog.
	/// `ArgumentValue` is expected to be an array where `args[0]` (optional)
	/// is the OpenDialogOptions DTO.
	pub async fn ShowOpenDialog(&self, ArgumentValue:Value) -> Result<Value, String> {
		let ParametersArray = ArgumentValue
			.as_array()
			.ok_or_else(|| ErrorUtils::RpcParamErrorString("ShowOpenDialog", "ArgumentValue", "array", None))?;

		let OptionsDtoValueOption = ParametersArray.get(0).cloned();
		info!(
			"[Rpc MainThreadDialogsHandler] ShowOpenDialog (DTO flow). Options: {:?}",
			OptionsDtoValueOption
		);

		let OpenDialogOptionsParsed:Option<CommonOpenDialogOptions> = OptionsDtoValueOption
			.map(|ValueItem| serde_json::from_value(ValueItem.clone()))
			.transpose()
			.map_err(|SerdeError| {
				ErrorUtils::RpcErrorString(
					format!("Invalid OpenDialogOptions DTO for ShowOpenDialog: {}", SerdeError),
					Some("EBADARG_DIALOG_OPTS_OPEN"),
				)
			})?;

		let ShowOpenDialogEffect = UiEffect::ShowOpenDialog(OpenDialogOptionsParsed);

		self.Runtime
			.Run(ShowOpenDialogEffect)
			.await
			.map(|PathsOptionVec:Option<Vec<PathBuf>>| {
				json!(
					PathsOptionVec
						.map(|PathsVec| { PathsVec.iter().map(file_path_to_uri_components_dto).collect::<Vec<_>>() })
				)
			})
			.map_err(|CommonErrorValue| {
				ErrorUtils::MapCommonErrorToRpcString(CommonErrorValue, "ShowOpenDialog DTO effect execution")
			})
	}

	/// Shows a native file save dialog.
	/// `ArgumentValue` is expected to be an array where `args[0]` (optional)
	/// is the SaveDialogOptions DTO.
	pub async fn ShowSaveDialog(&self, ArgumentValue:Value) -> Result<Value, String> {
		let ParametersArray = ArgumentValue
			.as_array()
			.ok_or_else(|| ErrorUtils::RpcParamErrorString("ShowSaveDialog", "ArgumentValue", "array", None))?;

		let OptionsDtoValueOption = ParametersArray.get(0).cloned();
		info!(
			"[Rpc MainThreadDialogsHandler] ShowSaveDialog (DTO flow). Options: {:?}",
			OptionsDtoValueOption
		);

		let SaveDialogOptionsParsed:Option<CommonSaveDialogOptions> = OptionsDtoValueOption
			.map(|ValueItem| serde_json::from_value(ValueItem.clone()))
			.transpose()
			.map_err(|SerdeError| {
				ErrorUtils::RpcErrorString(
					format!("Invalid SaveDialogOptions DTO for ShowSaveDialog: {}", SerdeError),
					Some("EBADARG_DIALOG_OPTS_SAVE"),
				)
			})?;

		let ShowSaveDialogEffect = UiEffect::ShowSaveDialog(SaveDialogOptionsParsed);

		self.Runtime
			.Run(ShowSaveDialogEffect)
			.await
			.map(|PathOption:Option<PathBuf>| {
				json!(PathOption.map(|PathItem| file_path_to_uri_components_dto(&PathItem)))
			})
			.map_err(|CommonErrorValue| {
				ErrorUtils::MapCommonErrorToRpcString(CommonErrorValue, "ShowSaveDialog DTO effect execution")
			})
	}
}
