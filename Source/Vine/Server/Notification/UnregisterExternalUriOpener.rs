use serde_json::Value;

use crate::Vine::Server::MountainVinegRPCService::MountainVinegRPCService;

pub async fn UnregisterExternalUriOpener(Service:&MountainVinegRPCService, Parameter:&Value) {
	// Remove any scheme entries registered under this opener_id from
	// the FeatureState map so `nativeHost:openExternal` stops routing
	// to a provider whose extension has deactivated.
	if let Some(OpenerId) = Parameter.get("opener_id").and_then(Value::as_str) {
		let mut Guard = Service
			.RunTime()
			.Environment
			.ApplicationState
			.Feature
			.ExternalUriOpeners
			.lock();

		Guard.retain(|_, Registration| Registration.OpenerId != OpenerId);
	}

	// Also unregister from the Vine provider registry by handle (the
	// standard path for all provider-unregistration notifications).
	::Vine::Server::Notification::UnregisterExternalUriOpener::UnregisterExternalUriOpener(Service, Parameter).await;
}
