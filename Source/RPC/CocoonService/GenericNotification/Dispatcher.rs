//! Dispatcher for the generic `send_mountain_notification` gRPC endpoint.
//!
//! Legacy fire-and-forget rail used by Cocoon's
//! `MountainGRPCClient.sendNotification(method, params)` for method names
//! that predate the typed proto endpoints.
//!
//! Dispatch shape: `register_*` provider names route through
//! `LanguageProviders::Dispatch` after a single prefix check; the
//! fire-and-forget Sky-emit handlers resolve through a `Lazy<HashMap>`
//! lookup; only the handful of async arms remain in a `match`.
use std::collections::HashMap;

use once_cell::sync::Lazy;
use serde_json::Value;
use tonic::{Request, Response, Status};
use ::Vine::Generated::{Empty, GenericNotification as GenericNotificationMsg};

use super::{Commands, LanguageProviders, SkyEmit};
use crate::{Environment::MountainEnvironment::MountainEnvironment, RPC::CocoonService::CocoonServiceImpl, dev_log};

type SkyEmitHandler = fn(Value, &MountainEnvironment);

static SKY_EMIT_HANDLERS:Lazy<HashMap<&'static str, SkyEmitHandler>> = Lazy::new(|| {
	HashMap::from([
		("onDidReceiveMessage", SkyEmit::OnDidReceiveMessage::Fn as SkyEmitHandler),
		("webview.postMessage", SkyEmit::WebviewPostMessage::Fn as SkyEmitHandler),
		("webview.dispose", SkyEmit::WebviewDispose::Fn as SkyEmitHandler),
		("progress.start", SkyEmit::ProgressStart::Fn as SkyEmitHandler),
		("progress.update", SkyEmit::ProgressUpdate::Fn as SkyEmitHandler),
		("progress.complete", SkyEmit::ProgressComplete::Fn as SkyEmitHandler),
		("openExternal", SkyEmit::OpenExternal::Fn as SkyEmitHandler),
		("setStatusBarText", SkyEmit::SetStatusBarText::Fn as SkyEmitHandler),
		("statusBar.setText", SkyEmit::SetStatusBarText::Fn as SkyEmitHandler),
		("disposeStatusBarItem", SkyEmit::DisposeStatusBarItem::Fn as SkyEmitHandler),
		("statusBar.dispose", SkyEmit::DisposeStatusBarItem::Fn as SkyEmitHandler),
		("output.create", SkyEmit::OutputCreate::Fn as SkyEmitHandler),
		("output.append", SkyEmit::OutputAppend::Fn as SkyEmitHandler),
		("output.appendLine", SkyEmit::OutputAppendLine::Fn as SkyEmitHandler),
		("output.clear", SkyEmit::OutputClear::Fn as SkyEmitHandler),
		("output.show", SkyEmit::OutputShow::Fn as SkyEmitHandler),
		("output.dispose", SkyEmit::OutputDispose::Fn as SkyEmitHandler),
		(
			"set_language_configuration",
			SkyEmit::SetLanguageConfiguration::Fn as SkyEmitHandler,
		),
	])
});

pub async fn Fn(
	Service:&CocoonServiceImpl,

	request:Request<GenericNotificationMsg>,
) -> Result<Response<Empty>, Status> {
	let notification = request.into_inner();

	dev_log!(
		"cocoon",
		"[CocoonService] Notification router: method='{}'",
		notification.method
	);

	// Deserialise notification parameters as JSON
	let Params:Value = if notification.parameter.is_empty() {
		Value::Null
	} else {
		serde_json::from_slice(&notification.parameter).unwrap_or(Value::Null)
	};

	let Method = notification.method.as_str();

	// ---- Language Providers (APIFactoryService.ts register_*_provider strings)
	// ----
	if Method.starts_with("register_") {
		if !LanguageProviders::Dispatch::Fn(Method, Params, Service) {
			dev_log!(
				"cocoon",
				"[CocoonService] Unknown notification method: '{}'",
				notification.method
			);
		}

		return Ok(Response::new(Empty {}));
	}

	// ---- Fire-and-forget Sky emits (webview, status bar, output, progress,
	// openExternal, language configuration) ----
	if let Some(Handler) = SKY_EMIT_HANDLERS.get(Method) {
		Handler(Params, &Service.environment);

		return Ok(Response::new(Empty {}));
	}

	match Method {
		// ---- Commands ----
		"registerCommand" => {
			Commands::RegisterCommand::Fn(Params, &Service.environment).await;
		},

		"unregisterCommand" => {
			Commands::UnregisterCommand::Fn(Params, &Service.environment).await;
		},

		// ---- Secrets (fire-and-forget variants) ----
		"storeSecret" => {
			use CommonLibrary::Secret::SecretProvider::SecretProvider;

			let ExtensionId = Params.get("extensionId").and_then(|V| V.as_str()).unwrap_or("").to_string();

			let Key = Params.get("key").and_then(|V| V.as_str()).unwrap_or("").to_string();

			let Value = Params.get("value").and_then(|V| V.as_str()).unwrap_or("").to_string();

			let _ = Service.environment.StoreSecret(ExtensionId, Key, Value).await;
		},

		"deleteSecret" => {
			use CommonLibrary::Secret::SecretProvider::SecretProvider;

			let ExtensionId = Params.get("extensionId").and_then(|V| V.as_str()).unwrap_or("").to_string();

			let Key = Params.get("key").and_then(|V| V.as_str()).unwrap_or("").to_string();

			let _ = Service.environment.DeleteSecret(ExtensionId, Key).await;
		},

		// ---- File system (fire-and-forget write) ----
		"writeFile" => {
			let Uri = Params
				.get("uri")
				.and_then(|V| V.get("value").or(Some(V)))
				.and_then(|V| V.as_str())
				.unwrap_or("")
				.replace("file://", "");

			let Content:Vec<u8> = Params
				.get("content")
				.and_then(|V| V.as_array())
				.map(|A| A.iter().filter_map(|B| B.as_u64().map(|N| N as u8)).collect())
				.unwrap_or_default();

			let _ = tokio::fs::write(&Uri, &Content).await;
		},

		_ => {
			dev_log!(
				"cocoon",
				"[CocoonService] Unknown notification method: '{}'",
				notification.method
			);
		},
	}

	Ok(Response::new(Empty {}))
}
