use serde_json::Value;

use crate::Vine::Server::MountainVinegRPCService::MountainVinegRPCService;

pub async fn OutputChannelDispose(Service:&MountainVinegRPCService, Parameter:&Value) {

	::Vine::Server::Notification::OutputChannelDispose::OutputChannelDispose(Service, Parameter).await;
}
