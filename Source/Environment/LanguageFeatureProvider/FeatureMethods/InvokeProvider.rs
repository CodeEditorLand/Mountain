//! Invokes a registered language provider over reverse-RPC using the
//! `$provide{ProviderType}` method-name convention.

use std::sync::Arc;

use CommonLibrary::{
	Environment::Requires::Requires,
	Error::CommonError::CommonError,
	IPC::IPCProvider::IPCProvider,
};
use serde_json::{Value, json};

use crate::ApplicationState::DTO::ProviderRegistrationDTO::ProviderRegistrationDTO;

pub(crate) async fn Fn(
	environment:&crate::Environment::MountainEnvironment::MountainEnvironment,

	registration:&ProviderRegistrationDTO,

	arguments:Vec<Value>,
) -> Result<Value, CommonError> {
	let rpc_method = format!("$provide{}", registration.ProviderType.to_string());

	let ipc_provider:Arc<dyn IPCProvider> = environment.Require();

	ipc_provider
		.SendRequestToSideCar(registration.SideCarIdentifier.clone(), rpc_method, json!(arguments), 5000)
		.await
}
