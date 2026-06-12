use serde_json::Value;

use crate::Vine::Server::MountainVinegRPCService::MountainVinegRPCService;

/// Handles security incident notifications via Vine IPC.
pub async fn SecurityIncident(Service:&MountainVinegRPCService, Parameter:&Value) {
	::Vine::Server::Notification::SecurityIncident::SecurityIncident(Service, Parameter).await;
}
