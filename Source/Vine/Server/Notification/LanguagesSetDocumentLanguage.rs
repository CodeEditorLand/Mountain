use serde_json::Value;

use crate::Vine::Server::MountainVinegRPCService::MountainVinegRPCService;

/// Sets the document language mode via Vine IPC.
pub async fn LanguagesSetDocumentLanguage(Service:&MountainVinegRPCService, Parameter:&Value) {
	::Vine::Server::Notification::LanguagesSetDocumentLanguage::LanguagesSetDocumentLanguage(Service, Parameter).await;
}
