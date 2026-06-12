use serde_json::Value;

use crate::Vine::Server::MountainVinegRPCService::MountainVinegRPCService;

/// Applies text editor decorations via Vine IPC.
pub async fn SetTextEditorDecorations(Service:&MountainVinegRPCService, Parameter:&Value) {
	::Vine::Server::Notification::SetTextEditorDecorations::SetTextEditorDecorations(Service, Parameter).await;

	// P4.6: Lazy-register unknown decoration type keys. Extensions may call
	// `editor.setDecorations(type, ranges)` before `createTextEditorDecorationType`
	// resolves, or the registration notification may arrive out-of-order relative
	// to the decoration batch. When a `decorationTypeKey` in the payload isn't
	// yet known, auto-forward it as a create so Monaco has styling registered by
	// the time `setDecorationsByType` executes.
	if let Some(Key) = Parameter.get("key").and_then(|K| K.as_str()) {
		let Decorations = &Service.RunTime().Environment.ApplicationState.Feature.Decorations;

		if Decorations.RegisterTypeKey(Key) {
			// This is a newly-seen decoration type key. Forward a synthetic
			// createTextEditorDecorationType notification so Sky can call
			// `ICodeEditorService.registerDecorationType("ext", Key, options)`.
			let CreatePayload = serde_json::json!({
				"key": Key,
				"options": Parameter.get("options").unwrap_or(&Value::Null),
			});

			::Vine::Server::Notification::DecorationTypeLifecycle::DecorationTypeLifecycle(
				Service,
				"window.createTextEditorDecorationType",
				&CreatePayload,
			)
			.await;
		}
	}

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
