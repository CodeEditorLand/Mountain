use serde_json::Value;

use crate::Vine::Server::MountainVinegRPCService::MountainVinegRPCService;

pub async fn RegisterLanguageProvider(Service:&MountainVinegRPCService, MethodName:&str, Parameter:&Value) -> bool {
	::Vine::Server::Notification::RegisterLanguageProvider::RegisterLanguageProvider(Service, MethodName, Parameter)
		.await
}
