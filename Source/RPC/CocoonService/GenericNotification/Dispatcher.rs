//! Dispatcher for the generic `send_mountain_notification` gRPC endpoint.
//!
//! Legacy fire-and-forget rail used by Cocoon's
//! `MountainGRPCClient.sendNotification(method, params)` for method names
//! that predate the typed proto endpoints.

use serde_json::json;
use tonic::{Request, Response, Status};
use CommonLibrary::{
	Command::CommandExecutor::CommandExecutor,
	LanguageFeature::{
		DTO::ProviderType::ProviderType,
		LanguageFeatureProviderRegistry::LanguageFeatureProviderRegistry,
	},
};

use crate::{
	RPC::CocoonService::CocoonServiceImpl,
	Vine::Generated::{Empty, GenericNotification as GenericNotificationMsg},
	dev_log,
};

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
	let Params:serde_json::Value = if notification.parameter.is_empty() {
		serde_json::Value::Null
	} else {
		serde_json::from_slice(&notification.parameter).unwrap_or(serde_json::Value::Null)
	};

	match notification.method.as_str() {
		// ---- Commands ----
		"registerCommand" => {
			let CommandId = Params.get("commandId").and_then(|V| V.as_str()).unwrap_or("").to_string();

			let ExtensionId = Params.get("extensionId").and_then(|V| V.as_str()).unwrap_or("").to_string();

			if let Err(Error) = Service.environment.RegisterCommand(ExtensionId, CommandId.clone()).await {
				dev_log!(
					"cocoon",
					"warn: [CocoonService] notification: registerCommand '{}' failed: {:?}",
					CommandId,
					Error
				);
			}
		},

		"unregisterCommand" => {
			let ExtensionId = Params.get("extensionId").and_then(|V| V.as_str()).unwrap_or("").to_string();

			let CommandId = Params.get("commandId").and_then(|V| V.as_str()).unwrap_or("").to_string();

			let _ = Service.environment.UnregisterCommand(ExtensionId, CommandId).await;
		},

		// ---- Language Providers (APIFactoryService.ts register_*_provider strings) ----
		"register_hover_provider" => {
			let Handle = Params.get("handle").and_then(|V| V.as_u64()).unwrap_or(0) as u32;

			let Selector = Params.get("language_selector").and_then(|V| V.as_str()).unwrap_or("*");

			let ExtId = Params.get("ExtensionId").and_then(|V| V.as_str()).unwrap_or("");

			Service.RegisterProvider(Handle, ProviderType::Fn, Selector, ExtId);
		},

		"register_completion_item_provider" => {
			let Handle = Params.get("handle").and_then(|V| V.as_u64()).unwrap_or(0) as u32;

			let Selector = Params.get("language_selector").and_then(|V| V.as_str()).unwrap_or("*");

			let ExtId = Params.get("ExtensionId").and_then(|V| V.as_str()).unwrap_or("");

			Service.RegisterProvider(Handle, ProviderType::Completion, Selector, ExtId);
		},

		"register_definition_provider" => {
			let Handle = Params.get("handle").and_then(|V| V.as_u64()).unwrap_or(0) as u32;

			let Selector = Params.get("language_selector").and_then(|V| V.as_str()).unwrap_or("*");

			let ExtId = Params.get("ExtensionId").and_then(|V| V.as_str()).unwrap_or("");

			Service.RegisterProvider(Handle, ProviderType::Definition, Selector, ExtId);
		},

		"register_reference_provider" => {
			let Handle = Params.get("handle").and_then(|V| V.as_u64()).unwrap_or(0) as u32;

			let Selector = Params.get("language_selector").and_then(|V| V.as_str()).unwrap_or("*");

			let ExtId = Params.get("ExtensionId").and_then(|V| V.as_str()).unwrap_or("");

			Service.RegisterProvider(Handle, ProviderType::References, Selector, ExtId);
		},

		"register_code_actions_provider" => {
			let Handle = Params.get("handle").and_then(|V| V.as_u64()).unwrap_or(0) as u32;

			let Selector = Params.get("language_selector").and_then(|V| V.as_str()).unwrap_or("*");

			let ExtId = Params.get("ExtensionId").and_then(|V| V.as_str()).unwrap_or("");

			Service.RegisterProvider(Handle, ProviderType::CodeAction, Selector, ExtId);
		},

		"register_document_highlight_provider" => {
			let Handle = Params.get("handle").and_then(|V| V.as_u64()).unwrap_or(0) as u32;

			let Selector = Params.get("language_selector").and_then(|V| V.as_str()).unwrap_or("*");

			let ExtId = Params.get("ExtensionId").and_then(|V| V.as_str()).unwrap_or("");

			Service.RegisterProvider(Handle, ProviderType::DocumentHighlight, Selector, ExtId);
		},

		"register_document_symbol_provider" => {
			let Handle = Params.get("handle").and_then(|V| V.as_u64()).unwrap_or(0) as u32;

			let Selector = Params.get("language_selector").and_then(|V| V.as_str()).unwrap_or("*");

			let ExtId = Params.get("ExtensionId").and_then(|V| V.as_str()).unwrap_or("");

			Service.RegisterProvider(Handle, ProviderType::DocumentSymbol, Selector, ExtId);
		},

		"register_workspace_symbol_provider" => {
			let Handle = Params.get("handle").and_then(|V| V.as_u64()).unwrap_or(0) as u32;

			let Selector = Params.get("language_selector").and_then(|V| V.as_str()).unwrap_or("*");

			let ExtId = Params.get("ExtensionId").and_then(|V| V.as_str()).unwrap_or("");

			Service.RegisterProvider(Handle, ProviderType::WorkspaceSymbol, Selector, ExtId);
		},

		"register_rename_provider" => {
			let Handle = Params.get("handle").and_then(|V| V.as_u64()).unwrap_or(0) as u32;

			let Selector = Params.get("language_selector").and_then(|V| V.as_str()).unwrap_or("*");

			let ExtId = Params.get("ExtensionId").and_then(|V| V.as_str()).unwrap_or("");

			Service.RegisterProvider(Handle, ProviderType::Rename, Selector, ExtId);
		},

		"register_document_formatting_provider" => {
			let Handle = Params.get("handle").and_then(|V| V.as_u64()).unwrap_or(0) as u32;

			let Selector = Params.get("language_selector").and_then(|V| V.as_str()).unwrap_or("*");

			let ExtId = Params.get("ExtensionId").and_then(|V| V.as_str()).unwrap_or("");

			Service.RegisterProvider(Handle, ProviderType::DocumentFormatting, Selector, ExtId);
		},

		"register_document_range_formatting_provider" => {
			let Handle = Params.get("handle").and_then(|V| V.as_u64()).unwrap_or(0) as u32;

			let Selector = Params.get("language_selector").and_then(|V| V.as_str()).unwrap_or("*");

			let ExtId = Params.get("ExtensionId").and_then(|V| V.as_str()).unwrap_or("");

			Service.RegisterProvider(Handle, ProviderType::DocumentRangeFormatting, Selector, ExtId);
		},

		"register_on_type_formatting_provider" => {
			let Handle = Params.get("handle").and_then(|V| V.as_u64()).unwrap_or(0) as u32;

			let Selector = Params.get("language_selector").and_then(|V| V.as_str()).unwrap_or("*");

			let ExtId = Params.get("ExtensionId").and_then(|V| V.as_str()).unwrap_or("");

			Service.RegisterProvider(Handle, ProviderType::OnTypeFormatting, Selector, ExtId);
		},

		"register_signature_help_provider" => {
			let Handle = Params.get("handle").and_then(|V| V.as_u64()).unwrap_or(0) as u32;

			let Selector = Params.get("language_selector").and_then(|V| V.as_str()).unwrap_or("*");

			let ExtId = Params.get("ExtensionId").and_then(|V| V.as_str()).unwrap_or("");

			Service.RegisterProvider(Handle, ProviderType::SignatureHelp, Selector, ExtId);
		},

		"register_code_lens_provider" => {
			let Handle = Params.get("handle").and_then(|V| V.as_u64()).unwrap_or(0) as u32;

			let Selector = Params.get("language_selector").and_then(|V| V.as_str()).unwrap_or("*");

			let ExtId = Params.get("ExtensionId").and_then(|V| V.as_str()).unwrap_or("");

			Service.RegisterProvider(Handle, ProviderType::CodeLens, Selector, ExtId);
		},

		"register_folding_range_provider" => {
			let Handle = Params.get("handle").and_then(|V| V.as_u64()).unwrap_or(0) as u32;

			let Selector = Params.get("language_selector").and_then(|V| V.as_str()).unwrap_or("*");

			let ExtId = Params.get("ExtensionId").and_then(|V| V.as_str()).unwrap_or("");

			Service.RegisterProvider(Handle, ProviderType::FoldingRange, Selector, ExtId);
		},

		"register_selection_range_provider" => {
			let Handle = Params.get("handle").and_then(|V| V.as_u64()).unwrap_or(0) as u32;

			let Selector = Params.get("language_selector").and_then(|V| V.as_str()).unwrap_or("*");

			let ExtId = Params.get("ExtensionId").and_then(|V| V.as_str()).unwrap_or("");

			Service.RegisterProvider(Handle, ProviderType::SelectionRange, Selector, ExtId);
		},

		"register_semantic_tokens_provider" => {
			let Handle = Params.get("handle").and_then(|V| V.as_u64()).unwrap_or(0) as u32;

			let Selector = Params.get("language_selector").and_then(|V| V.as_str()).unwrap_or("*");

			let ExtId = Params.get("ExtensionId").and_then(|V| V.as_str()).unwrap_or("");

			Service.RegisterProvider(Handle, ProviderType::SemanticTokens, Selector, ExtId);
		},

		"register_inlay_hints_provider" => {
			let Handle = Params.get("handle").and_then(|V| V.as_u64()).unwrap_or(0) as u32;

			let Selector = Params.get("language_selector").and_then(|V| V.as_str()).unwrap_or("*");

			let ExtId = Params.get("ExtensionId").and_then(|V| V.as_str()).unwrap_or("");

			Service.RegisterProvider(Handle, ProviderType::InlayHint, Selector, ExtId);
		},

		"register_type_hierarchy_provider" => {
			let Handle = Params.get("handle").and_then(|V| V.as_u64()).unwrap_or(0) as u32;

			let Selector = Params.get("language_selector").and_then(|V| V.as_str()).unwrap_or("*");

			let ExtId = Params.get("ExtensionId").and_then(|V| V.as_str()).unwrap_or("");

			Service.RegisterProvider(Handle, ProviderType::TypeHierarchy, Selector, ExtId);
		},

		"register_call_hierarchy_provider" => {
			let Handle = Params.get("handle").and_then(|V| V.as_u64()).unwrap_or(0) as u32;

			let Selector = Params.get("language_selector").and_then(|V| V.as_str()).unwrap_or("*");

			let ExtId = Params.get("ExtensionId").and_then(|V| V.as_str()).unwrap_or("");

			Service.RegisterProvider(Handle, ProviderType::CallHierarchy, Selector, ExtId);
		},

		"register_linked_editing_range_provider" => {
			let Handle = Params.get("handle").and_then(|V| V.as_u64()).unwrap_or(0) as u32;

			let Selector = Params.get("language_selector").and_then(|V| V.as_str()).unwrap_or("*");

			let ExtId = Params.get("ExtensionId").and_then(|V| V.as_str()).unwrap_or("");

			Service.RegisterProvider(Handle, ProviderType::LinkedEditingRange, Selector, ExtId);
		},

		"register_document_link_provider" => {
			let Handle = Params.get("handle").and_then(|V| V.as_u64()).unwrap_or(0) as u32;

			let Selector = Params.get("language_selector").and_then(|V| V.as_str()).unwrap_or("*");

			let ExtId = Params.get("ExtensionId").and_then(|V| V.as_str()).unwrap_or("");

			Service.RegisterProvider(Handle, ProviderType::DocumentLink, Selector, ExtId);
		},

		"register_color_provider" => {
			let Handle = Params.get("handle").and_then(|V| V.as_u64()).unwrap_or(0) as u32;

			let Selector = Params.get("language_selector").and_then(|V| V.as_str()).unwrap_or("*");

			let ExtId = Params.get("ExtensionId").and_then(|V| V.as_str()).unwrap_or("");

			Service.RegisterProvider(Handle, ProviderType::Color, Selector, ExtId);
		},

		"register_implementation_provider" => {
			let Handle = Params.get("handle").and_then(|V| V.as_u64()).unwrap_or(0) as u32;

			let Selector = Params.get("language_selector").and_then(|V| V.as_str()).unwrap_or("*");

			let ExtId = Params.get("ExtensionId").and_then(|V| V.as_str()).unwrap_or("");

			Service.RegisterProvider(Handle, ProviderType::Implementation, Selector, ExtId);
		},

		"register_type_definition_provider" => {
			let Handle = Params.get("handle").and_then(|V| V.as_u64()).unwrap_or(0) as u32;

			let Selector = Params.get("language_selector").and_then(|V| V.as_str()).unwrap_or("*");

			let ExtId = Params.get("ExtensionId").and_then(|V| V.as_str()).unwrap_or("");

			Service.RegisterProvider(Handle, ProviderType::TypeDefinition, Selector, ExtId);
		},

		"register_declaration_provider" => {
			let Handle = Params.get("handle").and_then(|V| V.as_u64()).unwrap_or(0) as u32;

			let Selector = Params.get("language_selector").and_then(|V| V.as_str()).unwrap_or("*");

			let ExtId = Params.get("ExtensionId").and_then(|V| V.as_str()).unwrap_or("");

			Service.RegisterProvider(Handle, ProviderType::Declaration, Selector, ExtId);
		},

		"register_evaluatable_expression_provider" => {
			let Handle = Params.get("handle").and_then(|V| V.as_u64()).unwrap_or(0) as u32;

			let Selector = Params.get("language_selector").and_then(|V| V.as_str()).unwrap_or("*");

			let ExtId = Params.get("ExtensionId").and_then(|V| V.as_str()).unwrap_or("");

			Service.RegisterProvider(Handle, ProviderType::EvaluatableExpression, Selector, ExtId);
		},

		"register_inline_values_provider" => {
			let Handle = Params.get("handle").and_then(|V| V.as_u64()).unwrap_or(0) as u32;

			let Selector = Params.get("language_selector").and_then(|V| V.as_str()).unwrap_or("*");

			let ExtId = Params.get("ExtensionId").and_then(|V| V.as_str()).unwrap_or("");

			Service.RegisterProvider(Handle, ProviderType::InlineValues, Selector, ExtId);
		},

		// ---- Webview ----
		"onDidReceiveMessage" => {
			use tauri::Emitter;

			let Handle = Params.get("handle").and_then(|V| V.as_u64()).unwrap_or(0);

			let Message = Params
				.Get("stringMessage")
				.and_then(|V| V.as_str())
				.map(|S| S.to_string())
				.or_else(|| Params.get("bytesMessage").map(|_| "[binary]".to_string()))
				.unwrap_or_default();

			let _ = Service
				.environment
				.ApplicationHandle
				.emit("sky://webview/postMessage", json!({ "handle": Handle, "message": Message }));
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
				.Get("uri")
				.and_then(|V| V.get("value").or(Some(V)))
				.and_then(|V| V.as_str())
				.unwrap_or("")
				.replace("file://", "");

			let Content:Vec<u8> = Params
				.Get("content")
				.and_then(|V| V.as_array())
				.map(|A| A.iter().filter_map(|B| B.as_u64().map(|N| N as u8)).collect())
				.unwrap_or_default();

			let _ = tokio::fs::write(&Uri, &Content).await;
		},

		// ---- Webview panel ----
		"webview.postMessage" => {
			use tauri::Emitter;

			let PanelId = Params.get("panelId").and_then(|V| V.as_str()).unwrap_or("").to_string();

			let Method = Params.get("method").and_then(|V| V.as_str()).unwrap_or("").to_string();

			let MsgParams = Params.get("params").cloned().unwrap_or(serde_json::Value::Null);

			let _ = Service.environment.ApplicationHandle.emit(
				"sky://webview/message",
				json!({ "panelId": PanelId, "method": Method, "params": MsgParams }),
			);
		},

		"webview.Dispose" => {
			use tauri::Emitter;

			let PanelId = Params.get("panelId").and_then(|V| V.as_str()).unwrap_or("").to_string();

			let _ = Service
				.environment
				.ApplicationHandle
				.emit("sky://webview/dispose", json!({ "panelId": PanelId }));
		},

		// ---- Progress indicator ----
		"progress.Start" => {
			use tauri::Emitter;

			let Id = Params.get("id").and_then(|V| V.as_str()).unwrap_or("").to_string();

			let Title = Params.get("title").and_then(|V| V.as_str()).map(|S| S.to_string());

			let Location = Params.get("location").cloned();

			let Cancellable = Params.get("cancellable").and_then(|V| V.as_bool()).unwrap_or(false);

			let _ = Service.environment.ApplicationHandle.emit(
				"sky://progress/start",
				json!({ "id": Id, "title": Title, "location": Location, "cancellable": Cancellable }),
			);
		},

		"progress.update" => {
			use tauri::Emitter;

			let Id = Params.get("id").and_then(|V| V.as_str()).unwrap_or("").to_string();

			let Message = Params.get("message").and_then(|V| V.as_str()).map(|S| S.to_string());

			let Increment = Params.get("increment").and_then(|V| V.as_f64());

			let _ = Service.environment.ApplicationHandle.emit(
				"sky://progress/update",
				json!({ "id": Id, "message": Message, "increment": Increment }),
			);
		},

		"progress.complete" => {
			use tauri::Emitter;

			let Id = Params.get("id").and_then(|V| V.as_str()).unwrap_or("").to_string();

			let _ = Service
				.environment
				.ApplicationHandle
				.emit("sky://progress/complete", json!({ "id": Id }));
		},

		// ---- Native shell ----
		"openExternal" => {
			use tauri::Emitter;

			let Url = Params.get("url").and_then(|V| V.as_str()).unwrap_or("").to_string();

			let _ = Service
				.environment
				.ApplicationHandle
				.emit("sky://native/openExternal", json!({ "url": Url }));
		},

		// ---- StatusBar updates (fire-and-forget from Window.ts setters) ----
		"setStatusBarText" | "statusBar.setText" => {
			use tauri::Emitter;

			let ItemId = Params.get("itemId").and_then(|V| V.as_str()).unwrap_or("").to_string();

			let Text = Params.get("text").and_then(|V| V.as_str()).unwrap_or("").to_string();

			let _ = Service
				.environment
				.ApplicationHandle
				.emit("sky://statusbar/update", json!({ "id": ItemId, "text": Text }));
		},

		"disposeStatusBarItem" | "statusBar.Dispose" => {
			use tauri::Emitter;

			let ItemId = Params.get("itemId").and_then(|V| V.as_str()).unwrap_or("").to_string();

			let _ = Service
				.environment
				.ApplicationHandle
				.emit("sky://statusbar/dispose", json!({ "id": ItemId }));
		},

		// ---- Output channel (fire-and-forget from Window.ts OutputChannel proxy) ----
		"output.create" => {
			use tauri::Emitter;

			let Id = Params.get("id").and_then(|V| V.as_str()).unwrap_or("").to_string();

			let Name = Params.get("name").and_then(|V| V.as_str()).unwrap_or("").to_string();

			let _ = Service
				.environment
				.ApplicationHandle
				.emit("sky://output/create", json!({ "id": Id, "name": Name }));
		},

		"output.append" => {
			use tauri::Emitter;

			let Channel = Params.get("channel").and_then(|V| V.as_str()).unwrap_or("").to_string();

			let Text = Params.get("value").and_then(|V| V.as_str()).unwrap_or("").to_string();

			let _ = Service
				.environment
				.ApplicationHandle
				.emit("sky://output/append", json!({ "channel": Channel, "text": Text }));
		},

		"output.appendLine" => {
			use tauri::Emitter;

			let Channel = Params.get("channel").and_then(|V| V.as_str()).unwrap_or("").to_string();

			let Line = Params.get("value").and_then(|V| V.as_str()).unwrap_or("").to_string();

			let _ = Service.environment.ApplicationHandle.emit(
				"sky://output/append",
				json!({ "channel": Channel, "text": format!("{}\n", Line) }),
			);
		},

		"output.clear" => {
			use tauri::Emitter;

			let Channel = Params.get("channel").and_then(|V| V.as_str()).unwrap_or("").to_string();

			let _ = Service
				.environment
				.ApplicationHandle
				.emit("sky://output/clear", json!({ "channel": Channel }));
		},

		"output.show" => {
			use tauri::Emitter;

			let Channel = Params.get("channel").and_then(|V| V.as_str()).unwrap_or("").to_string();

			let _ = Service
				.environment
				.ApplicationHandle
				.emit("sky://output/show", json!({ "channel": Channel }));
		},

		"output.Dispose" => {
			use tauri::Emitter;

			let Channel = Params.get("channel").and_then(|V| V.as_str()).unwrap_or("").to_string();

			let _ = Service
				.environment
				.ApplicationHandle
				.emit("sky://output/dispose", json!({ "channel": Channel }));
		},

		// ---- Language configuration ----
		"set_language_configuration" => {
			// Language configuration is consumed by Sky - emit for workbench to pick up
			use tauri::Emitter;

			let Language = Params.get("language").and_then(|V| V.as_str()).unwrap_or("").to_string();

			let _ = Service
				.environment
				.ApplicationHandle
				.emit("sky://language/configure", json!({ "language": Language }));
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
