use serde_json::Value;

use crate::Vine::Server::MountainVinegRPCService::MountainVinegRPCService;

pub async fn OutputAppend(Service:&MountainVinegRPCService, Parameter:&Value) {

	::Vine::Server::Notification::OutputAppend::OutputAppend(Service, Parameter).await;
}
