use serde_json::Value;

use crate::Vine::Server::MountainVinegRPCService::MountainVinegRPCService;

/// Appends plain text to an output channel via Vine IPC.
pub async fn OutputAppend(Service:&MountainVinegRPCService, Parameter:&Value) {
	::Vine::Server::Notification::OutputAppend::OutputAppend(Service, Parameter).await;
}
