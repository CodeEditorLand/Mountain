use serde_json::Value;

use crate::Vine::Server::MountainVinegRPCService::MountainVinegRPCService;

pub async fn OutputChannelReplace(Service:&MountainVinegRPCService, Parameter:&Value) {
	::Vine::Server::Notification::OutputChannelReplace::OutputChannelReplace(Service, Parameter).await;
}
