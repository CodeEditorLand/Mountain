// File: Rpc/MainThreadStorageHandler.rs
// Defines the RPC handler for Memento storage operations (global and workspace)
// requested by the sidecar.

use std::sync::Arc;

use Common::StorageEffect; // Assuming this path
use Common::{Errors::CommonError, Runtime::AppRuntimeTrait};
use log::{debug, info, trace};
use serde_json::{Value, json};
use tauri::{AppHandle, Manager, State, Wry};

use crate::{
	Handlers::ErrorUtils,
	Rpc::Argument::Storage::{GetValueArgument, SetValueArgument},
	Runtime::AppRuntime,
};

#[derive(Clone)]
pub struct MainThreadStorageHandler {
	pub ApplicationHandle:AppHandle<Wry>,
	pub Runtime:Arc<AppRuntime>,
}

impl MainThreadStorageHandler {
	pub fn New(ApplicationHandle:AppHandle<Wry>, Runtime:Arc<AppRuntime>) -> Self {
		Self { ApplicationHandle, Runtime }
	}

	/// Retrieves a value from storage.
	pub async fn GetValue(&self, Argument:GetValueArgument) -> Result<Value, String> {
		info!(
			"[Rpc StorageHandler] GetValue (DTO): Scope={}, Key='{}'",
			Argument.Target.Scope, Argument.Target.Key
		);

		// The StorageEffect::Get expects a single Value argument representing the
		// target. We need to serialize our TargetDto into such a Value.
		let TargetObjectValue = serde_json::to_value(&Argument.Target).map_err(|SerializationError| {
			ErrorUtils::RpcInternalErrorString(format!(
				"Failed to serialize GetValue::TargetDto for effect: {}",
				SerializationError
			))
		})?;

		let Effect = StorageEffect::GetStorageItem(TargetObjectValue);
		self.Runtime.Run(Effect).await
            .map(|OptionalValue| OptionalValue.unwrap_or(Value::Null)) // Return Null if None
            .map_err(|CommonErrorValue| ErrorUtils::MapCommonErrorToRpcString(CommonErrorValue, "GetValue DTO (Storage)"))
	}

	/// Sets or updates a value in storage.
	pub async fn SetValue(&self, Argument:SetValueArgument) -> Result<Value, String> {
		info!(
			"[Rpc StorageHandler] SetValue (DTO): Scope={}, Key='{}', ValueIsPresent={}",
			Argument.Target.Scope,
			Argument.Target.Key,
			!Argument.Value.is_null()
		);

		let TargetObjectValue = serde_json::to_value(&Argument.Target).map_err(|SerializationError| {
			ErrorUtils::RpcInternalErrorString(format!(
				"Failed to serialize SetValue::TargetDto for effect: {}",
				SerializationError
			))
		})?;

		// The effect expects the value to set directly, not wrapped in the DTO.
		let Effect = StorageEffect::SetStorageItem(TargetObjectValue, Argument.Value);
		self.Runtime.Run(Effect).await
            .map(|_| Value::Null) // Success is indicated by Value::Null
            .map_err(|CommonErrorValue| ErrorUtils::MapCommonErrorToRpcString(CommonErrorValue, "SetValue DTO (Storage)"))
	}
}
