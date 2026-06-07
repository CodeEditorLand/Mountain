use serde_json::Value;

use crate::Vine::Server::MountainVinegRPCService::MountainVinegRPCService;

pub async fn SetLanguageConfiguration(Service:&MountainVinegRPCService, Parameter:&Value) {

	::Vine::Server::Notification::SetLanguageConfiguration::SetLanguageConfiguration(Service, Parameter).await;
}
