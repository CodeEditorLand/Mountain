use serde_json::Value;

use crate::Vine::Server::MountainVinegRPCService::MountainVinegRPCService;

/// Sets the language configuration for a file type via Vine IPC.
pub async fn SetLanguageConfiguration(Service:&MountainVinegRPCService, Parameter:&Value) {
	::Vine::Server::Notification::SetLanguageConfiguration::SetLanguageConfiguration(Service, Parameter).await;
}
