use serde_json::Value;

use crate::Vine::Server::MountainVinegRPCService::MountainVinegRPCService;

/// Opens a document in the workbench via Vine IPC.
pub async fn WindowShowTextDocument(Service:&MountainVinegRPCService, Parameter:&Value) {
	::Vine::Server::Notification::WindowShowTextDocument::WindowShowTextDocument(Service, Parameter).await;
}
