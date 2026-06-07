use serde_json::Value;

use crate::Vine::Server::MountainVinegRPCService::MountainVinegRPCService;

pub async fn RegisterCommand(Service:&MountainVinegRPCService, Parameter:&Value) {

	::Vine::Server::Notification::RegisterCommand::RegisterCommand(Service, Parameter).await;
}
