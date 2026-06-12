use serde_json::Value;

use crate::Vine::Server::MountainVinegRPCService::MountainVinegRPCService;

/// Languagess set document language.
pub async fn LanguagesSetDocumentLanguage(Service:&MountainVinegRPCService, Parameter:&Value) {
	::Vine::Server::Notification::LanguagesSetDocumentLanguage::LanguagesSetDocumentLanguage(Service, Parameter).await;
}
