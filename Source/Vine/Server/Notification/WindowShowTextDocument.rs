use serde_json::Value;

use crate::Vine::Server::MountainVinegRPCService::MountainVinegRPCService;

/// Windows show text document.
pub async fn WindowShowTextDocument(Service:&MountainVinegRPCService, Parameter:&Value) {
	::Vine::Server::Notification::WindowShowTextDocument::WindowShowTextDocument(Service, Parameter).await;
}
