use serde_json::Value;

use crate::Vine::Server::MountainVinegRPCService::MountainVinegRPCService;

pub async fn ApplyTextEdits(Service:&MountainVinegRPCService, Parameter:&Value) {

	::Vine::Server::Notification::ApplyTextEdits::ApplyTextEdits(Service, Parameter).await;
}
