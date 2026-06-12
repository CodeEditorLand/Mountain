use serde_json::Value;

use crate::Vine::Server::MountainVinegRPCService::MountainVinegRPCService;

/// Opens an external URI via Vine IPC.
pub async fn OpenExternal(Service:&MountainVinegRPCService, Parameter:&Value) {
	::Vine::Server::Notification::OpenExternal::OpenExternal(Service, Parameter).await;
}
