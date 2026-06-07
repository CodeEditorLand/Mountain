use serde_json::Value;

use crate::Vine::Server::MountainVinegRPCService::MountainVinegRPCService;

pub async fn ProgressReport(Service:&MountainVinegRPCService, Parameter:&Value) {

	::Vine::Server::Notification::ProgressReport::ProgressReport(Service, Parameter).await;
}
