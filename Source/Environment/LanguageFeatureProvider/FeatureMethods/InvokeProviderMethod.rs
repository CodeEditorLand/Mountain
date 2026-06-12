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
	environment:&crate::Environment::MountainEnvironment::MountainEnvironment,

	registration:&ProviderRegistrationDTO,

	method:&str,

	arguments:Vec<Value>,
) -> Result<Value, CommonError> {
	// Extract renderer-supplied requestId from the fourth argument if present.
	let RequestIdentifier = arguments
		.get(3)
		.and_then(|v| v.as_str())
		.filter(|s| !s.is_empty())
		.map(String::from);

	let Cancellations = environment.ApplicationState.Feature.LanguageProviderCancellations.clone();

	super::InvokeProvider::ForwardCancellable(
		registration.SideCarIdentifier.clone(),
		method.to_string(),
		json!(arguments),
		Cancellations,
		RequestIdentifier,
	)
	.await
}
