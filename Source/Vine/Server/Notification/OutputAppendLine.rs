use serde_json::Value;

use crate::Vine::Server::MountainVinegRPCService::MountainVinegRPCService;

/// Appends a line of text to an output channel via Vine IPC.
pub async fn OutputAppendLine(Service:&MountainVinegRPCService, Parameter:&Value) {
	::Vine::Server::Notification::OutputAppendLine::OutputAppendLine(Service, Parameter).await;
}
