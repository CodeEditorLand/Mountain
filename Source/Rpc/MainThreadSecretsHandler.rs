// File: Rpc/MainThreadSecretsHandler.rs
// Defines the RPC handler for secret storage operations (get, set, delete)
// requested by the sidecar, using the system's keyring.

use std::sync::Arc;

use Common::SecretsEffects; // Assuming this path
use Common::{Errors::CommonError, Runtime::AppRuntimeTrait};
use log::{debug, info, trace};
use serde_json::{Value, json};
use tauri::{AppHandle, Manager, State, Wry};

use crate::Handlers::{self, ErrorUtils}; // Assuming Handlers::Secrets contains the logic
use crate::{
	Rpc::Args::Secrets::{GetSecretArgument, SetSecretArgument},
	Runtime::AppRuntime,
};

#[derive(Clone)]
pub struct MainThreadSecretsHandler {
	pub ApplicationHandle:AppHandle<Wry>,
	pub Runtime:Arc<AppRuntime>,
}

impl MainThreadSecretsHandler {
	pub fn New(ApplicationHandle:AppHandle<Wry>, Runtime:Arc<AppRuntime>) -> Self {
		Self { ApplicationHandle, Runtime }
	}

	/// Retrieves a secret from the keyring.
	pub async fn GetPassword(&self, Argument:GetSecretArgument) -> Result<Value, String> {
		info!(
			"[Rpc SecretsHandler] GetPassword (DTO): ExtensionIdentifier='{}', Key='{}'",
			Argument.ExtensionIdentifier, Argument.Key
		);

		let Effect = SecretsEffects::GetSecret(Argument.ExtensionIdentifier.clone(), Argument.Key.clone());
		self.Runtime.Run(Effect).await
            .map(|OptionalValue| OptionalValue.map_or(Value::Null, Value::String)) // Return string or null
            .map_err(|CommonErrorValue| ErrorUtils::MapCommonErrorToRpcString(CommonErrorValue, "GetPassword DTO (Secrets)"))
	}

	/// Stores a secret in the keyring.
	pub async fn SetPassword(&self, Argument:SetSecretArgument) -> Result<Value, String> {
		info!(
			"[Rpc SecretsHandler] SetPassword (DTO): ExtensionIdentifier='{}', Key='{}'",
			Argument.ExtensionIdentifier, Argument.Key
		);
		let Effect = SecretsEffects::StoreSecret(
			Argument.ExtensionIdentifier.clone(),
			Argument.Key.clone(),
			Argument.Value.clone(),
		);
		self.Runtime.Run(Effect).await
            .map(|_| Value::Null) // Success is indicated by Value::Null
            .map_err(|CommonErrorValue| ErrorUtils::MapCommonErrorToRpcString(CommonErrorValue, "SetPassword DTO (Secrets)"))
	}

	/// Deletes a secret from the keyring.
	/// Note: The DTO for delete is the same as for get.
	pub async fn DeletePassword(&self, Argument:GetSecretArgument) -> Result<Value, String> {
		info!(
			"[Rpc SecretsHandler] DeletePassword (DTO): ExtensionIdentifier='{}', Key='{}'",
			Argument.ExtensionIdentifier, Argument.Key
		);
		let Effect = SecretsEffects::DeleteSecret(Argument.ExtensionIdentifier.clone(), Argument.Key.clone());
		self.Runtime.Run(Effect).await
            .map(|_| Value::Null) // Success is indicated by Value::Null
            .map_err(|CommonErrorValue| ErrorUtils::MapCommonErrorToRpcString(CommonErrorValue, "DeletePassword DTO (Secrets)"))
	}
}
