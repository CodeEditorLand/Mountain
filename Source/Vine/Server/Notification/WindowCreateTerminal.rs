use serde_json::Value;

use crate::Vine::Server::MountainVinegRPCService::MountainVinegRPCService;

pub async fn WindowCreateTerminal(Service:&MountainVinegRPCService, Parameter:&Value) {
	::Vine::Server::Notification::WindowCreateTerminal::WindowCreateTerminal(Service, Parameter).await;
}
