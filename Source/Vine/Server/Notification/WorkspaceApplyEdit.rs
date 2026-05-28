use serde_json::Value;

use crate::Vine::Server::MountainVinegRPCService::MountainVinegRPCService;

pub async fn WorkspaceApplyEdit(Service:&MountainVinegRPCService, Parameter:&Value) {
	::Vine::Server::Notification::WorkspaceApplyEdit::WorkspaceApplyEdit(Service, Parameter).await;
}
