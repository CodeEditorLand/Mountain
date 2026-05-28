use serde_json::Value;

use crate::Vine::Server::MountainVinegRPCService::MountainVinegRPCService;

pub async fn OutputAppendLine(Service:&MountainVinegRPCService, Parameter:&Value) {
	::Vine::Server::Notification::OutputAppendLine::OutputAppendLine(Service, Parameter).await;
}
