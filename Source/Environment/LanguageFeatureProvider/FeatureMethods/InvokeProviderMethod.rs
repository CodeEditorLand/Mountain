//! Invokes a registered language provider over reverse-RPC with an explicit
//! method name instead of the `$provide{ProviderType}` convention. Used for
//! prepare steps (`$prepareCallHierarchyItems`, `$prepareTypeHierarchyItems`)
//! where the method prefix differs from the provider type string.
//!
//! Shares [`super::InvokeProvider::ForwardCancellable`], so a drop of the
//! calling future delivers `CocoonService.CancelOperation` to the side-car.

use CommonLibrary::Error::CommonError::CommonError;
use serde_json::{Value, json};

use crate::ApplicationState::DTO::ProviderRegistrationDTO::ProviderRegistrationDTO;

pub(crate) async fn Fn(
	_environment:&crate::Environment::MountainEnvironment::MountainEnvironment,

	registration:&ProviderRegistrationDTO,

	method:&str,

	arguments:Vec<Value>,
) -> Result<Value, CommonError> {
	super::InvokeProvider::ForwardCancellable(
		registration.SideCarIdentifier.clone(),
		method.to_string(),
		json!(arguments),
	)
	.await
}
