use serde_json::Value;

use crate::Vine::Server::MountainVinegRPCService::MountainVinegRPCService;

/// Registers a command via Vine IPC.
pub async fn RegisterCommand(Service:&MountainVinegRPCService, Parameter:&Value) {
	::Vine::Server::Notification::RegisterCommand::RegisterCommand(Service, Parameter).await;
}
