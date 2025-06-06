// File: Rpc/MainThreadConfigurationHandler.rs
// Defines the RPC handler for configuration-related requests from the sidecar.
// This includes getting, updating, and inspecting configuration values.

use std::sync::Arc;

use Common::Runtime::AppRuntimeTrait; // Assuming this path
use Common::{
	ConfigEffect::{
		self,
		ConfigurationTarget as CommonConfigurationTarget,
		IConfigurationOverrides as CommonConfigurationOverrides,
	},
	Errors::CommonError,
};
use log::{debug, info, warn};
use serde_json::Value;
use tauri::{AppHandle, Manager, State, Wry};

use crate::Handlers::ErrorUtils;
use crate::Rpc::Args::Configuration::{
	GetConfigurationArgument,
	InspectArgument as InspectConfigurationArgument, 
	UpdateArgument as UpdateConfigurationArgument,   
};
use crate::Runtime::AppRuntime; // Mountain's AppRuntime

#[derive(Clone)]
pub struct MainThreadConfigurationHandler {
	pub ApplicationHandle:AppHandle<Wry>,
	pub Runtime:Arc<AppRuntime>,
}

impl MainThreadConfigurationHandler {
	pub fn New(ApplicationHandle:AppHandle<Wry>, Runtime:Arc<AppRuntime>) -> Self {
		Self { ApplicationHandle, Runtime }
	}

	/// Gets a configuration value.
	pub async fn GetConfiguration(&self, Argument:GetConfigurationArgument) -> Result<Value, String> {
		info!(
			"[Rpc ConfigurationHandler] GetConfiguration (DTO): Section='{:?}'",
			Argument.Section
		);

		let CommonOverrides:CommonConfigurationOverrides =
			Argument.Overrides.map_or_else(CommonConfigurationOverrides::default, |Dto| {
				CommonConfigurationOverrides { Resource:Dto.Resource, OverrideIdentifier:Dto.OverrideIdentifier }
			});

		let Effect = ConfigEffect::GetConfiguration(
			Argument.Section,
			// The effect expects a Value for overrides, so we serialize our CommonConfigurationOverrides.
			// It's a bit circular if the effect internally deserializes it back to IConfigurationOverrides,
			// but this matches the pattern from the improvement files.
			serde_json::to_value(CommonOverrides)
				.map_err(|e| ErrorUtils::RpcInternalErrorString(format!("Failed to serialize overrides DTO: {}", e)))?,
			Argument.ScopeToLanguage,
		);

		self.Runtime
			.Run(Effect)
			.await
			.map_err(|Error| ErrorUtils::MapCommonErrorToRpcString(Error, "GetConfiguration DTO"))
	}

	/// Updates a configuration option.
	pub async fn UpdateConfigurationOption(&self, Argument:UpdateConfigurationArgument) -> Result<Value, String> {
		info!(
			"[Rpc ConfigurationHandler] UpdateConfigurationOption (DTO): Key='{}', TargetNumber={}",
			Argument.Key, Argument.Target
		);

		let CommonOverrides:CommonConfigurationOverrides =
			Argument.Overrides.map_or_else(CommonConfigurationOverrides::default, |Dto| {
				CommonConfigurationOverrides { Resource:Dto.Resource, OverrideIdentifier:Dto.OverrideIdentifier }
			});

		// Convert u32 Target to CommonConfigurationTarget enum
		let TargetScope:CommonConfigurationTarget =
			serde_json::from_value(Value::from(Argument.Target)).map_err(|e| {
				ErrorUtils::RpcParamErrorString(
					"UpdateConfigurationOption",
					"Target",
					"valid ConfigurationTarget u32",
					Some(0),
				)
			})?;

		let Effect = ConfigEffect::UpdateConfiguration(
			Argument.Key,
			Argument.Value,
			TargetScope,
			serde_json::to_value(CommonOverrides)
				.map_err(|e| ErrorUtils::RpcInternalErrorString(format!("Failed to serialize overrides DTO: {}", e)))?,
			Argument.ScopeToLanguage,
		);

		self.Runtime
			.Run(Effect)
			.await
			.map(|_| Value::Null)
			.map_err(|Error| ErrorUtils::MapCommonErrorToRpcString(Error, "UpdateConfigurationOption DTO"))
	}

	/// Removes a configuration option (by setting its value to null).
	pub async fn RemoveConfigurationOption(&self, Argument:UpdateConfigurationArgument) -> Result<Value, String> {
		info!(
			"[Rpc ConfigurationHandler] RemoveConfigurationOption (DTO): Key='{}', TargetNumber={}",
			Argument.Key, Argument.Target
		);

		let CommonOverrides:CommonConfigurationOverrides =
			Argument.Overrides.map_or_else(CommonConfigurationOverrides::default, |Dto| {
				CommonConfigurationOverrides { Resource:Dto.Resource, OverrideIdentifier:Dto.OverrideIdentifier }
			});
		let TargetScope:CommonConfigurationTarget =
			serde_json::from_value(Value::from(Argument.Target)).map_err(|e| {
				ErrorUtils::RpcParamErrorString(
					"RemoveConfigurationOption",
					"Target",
					"valid ConfigurationTarget u32",
					Some(0),
				)
			})?;

		let Effect = ConfigEffect::UpdateConfiguration(
			Argument.Key,
			Value::Null, // Removing is achieved by setting to null
			TargetScope,
			serde_json::to_value(CommonOverrides)
				.map_err(|e| ErrorUtils::RpcInternalErrorString(format!("Failed to serialize overrides DTO: {}", e)))?,
			Argument.ScopeToLanguage,
		);

		self.Runtime
			.Run(Effect)
			.await
			.map(|_| Value::Null)
			.map_err(|Error| ErrorUtils::MapCommonErrorToRpcString(Error, "RemoveConfigurationOption DTO"))
	}

	/// Inspects a configuration value, providing details about its source and
	/// effective value.
	pub async fn Inspect(&self, Argument:InspectConfigurationArgument) -> Result<Value, String> {
		info!("[Rpc ConfigurationHandler] Inspect (DTO): Key='{}'", Argument.Key);

		let CommonOverrides:CommonConfigurationOverrides =
			Argument.Overrides.map_or_else(CommonConfigurationOverrides::default, |Dto| {
				CommonConfigurationOverrides { Resource:Dto.Resource, OverrideIdentifier:Dto.OverrideIdentifier }
			});

		let Effect = ConfigEffect::InspectConfigurationValue(
			Argument.Key,
			serde_json::to_value(CommonOverrides)
				.map_err(|e| ErrorUtils::RpcInternalErrorString(format!("Failed to serialize overrides DTO: {}", e)))?,
		);

		self.Runtime
			.Run(Effect)
			.await
			.map(|OptionalInspectData| json!(OptionalInspectData))
			.map_err(|Error| ErrorUtils::MapCommonErrorToRpcString(Error, "Inspect DTO"))
	}
}
