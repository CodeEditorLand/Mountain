use serde_json::Value;

use crate::Vine::Server::MountainVinegRPCService::MountainVinegRPCService;

pub async fn SetTextEditorDecorations(Service:&MountainVinegRPCService, Parameter:&Value) {
	::Vine::Server::Notification::SetTextEditorDecorations::SetTextEditorDecorations(Service, Parameter)
		.await;
}
