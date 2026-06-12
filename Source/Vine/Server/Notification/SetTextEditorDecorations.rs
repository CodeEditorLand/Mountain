use serde_json::Value;

use crate::Vine::Server::MountainVinegRPCService::MountainVinegRPCService;

/// Sets text editor decorations.
pub async fn SetTextEditorDecorations(Service:&MountainVinegRPCService, Parameter:&Value) {
	::Vine::Server::Notification::SetTextEditorDecorations::SetTextEditorDecorations(Service, Parameter).await;

	// Persist the decoration range batch in ApplicationState so it survives
	// window reloads and can be replayed via sky:replay-events.
	if let Some(Uri) = Parameter.get("uri").and_then(|U| U.as_str()) {
		Service
			.RunTime()
			.Environment
			.ApplicationState
			.Feature
			.Decorations
			.SetDecoration(Uri, Parameter.clone());
	}
}
