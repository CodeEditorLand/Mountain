use serde_json::Value;

use crate::Vine::Server::MountainVinegRPCService::MountainVinegRPCService;

pub async fn OutputReplace(Service:&MountainVinegRPCService, Parameter:&Value) {

	::Vine::Server::Notification::OutputReplace::OutputReplace(Service, Parameter).await;
}
