//! Invokes a registered language provider over reverse-RPC with an explicit
//! method name instead of the `$provide{ProviderType}` convention. Used for
//! prepare steps (`$prepareCallHierarchyItems`, `$prepareTypeHierarchyItems`)
//! where the method prefix differs from the provider type string.

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

	method:&str,

	arguments:Vec<Value>,
) -> Result<Value, CommonError> {
	let ipc_provider:Arc<dyn IPCProvider> = environment.Require();

	ipc_provider
		.SendRequestToSideCar(
			registration.SideCarIdentifier.clone(),
			method.to_string(),
			json!(arguments),
			5000,
		)
		.await
}
