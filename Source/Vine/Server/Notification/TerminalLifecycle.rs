use serde_json::Value;

use crate::Vine::Server::MountainVinegRPCService::MountainVinegRPCService;

pub async fn TerminalLifecycle(Service:&MountainVinegRPCService, MethodName:&str, Parameter:&Value) {

	::Vine::Server::Notification::TerminalLifecycle::TerminalLifecycle(Service, MethodName, Parameter).await;
}
