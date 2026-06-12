use serde_json::Value;

use crate::Vine::Server::MountainVinegRPCService::MountainVinegRPCService;

/// Creates a new terminal window via Vine IPC.
pub async fn WindowCreateTerminal(Service:&MountainVinegRPCService, Parameter:&Value) {
	::Vine::Server::Notification::WindowCreateTerminal::WindowCreateTerminal(Service, Parameter).await;
}
