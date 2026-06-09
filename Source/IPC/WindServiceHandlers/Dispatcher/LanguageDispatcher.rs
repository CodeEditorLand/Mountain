//! Language feature command dispatcher - forwards to Cocoon Node.js runtime.

use CommonLibrary::LanguageFeature::{
	DTO::PositionDTO::PositionDTO,
	LanguageFeatureProviderRegistry::LanguageFeatureProviderRegistry,
};
use serde_json::{Value, json};

use crate::IPC::WindServiceHandlers::Utilities::JsonValueHelpers::arg_val;

fn cocoon_payload(args:Vec<serde_json::Value>) -> serde_json::Value {
	match args.len() {
		0 => serde_json::Value::Null,
		1 => args.into_iter().next().unwrap(),
		_ => serde_json::Value::Array(args),
	}
}

/// Dispatches language feature commands.
///
/// Handled commands:
/// - `language:provideInlineCompletions`
/// - `languages:getAll`
/// - `languages:getEncodedLanguageId`
/// - `language:prepareCallHierarchy`
/// - `language:provideCallHierarchyIncomingCalls`
/// - `language:provideCallHierarchyOutgoingCalls`
/// - `language:prepareTypeHierarchy`
/// - `language:provideTypeHierarchySupertypes`
/// - `language:provideTypeHierarchySubtypes`
/// - `language:provideLinkedEditingRanges`
pub async fn dispatch_language(
	runtime:std::sync::Arc<crate::RunTime::ApplicationRunTime::ApplicationRunTime>,

	command:&str,

	arguments:Vec<Value>,
) -> Result<Value, String> {
	match command {
		"language:provideInlineCompletions" => {
			let payload = arg_val(&arguments, 0);

			let uri_str = payload.get("uri").and_then(Value::as_str).unwrap_or("").to_string();

			if uri_str.is_empty() {
				return Ok(json!({ "items": [] }));
			}

			let line = payload
				.get("position")
				.and_then(|p| p.get("line"))
				.and_then(Value::as_u64)
				.unwrap_or(0) as i64
				+ 1;

			let character = payload
				.get("position")
				.and_then(|p| p.get("character"))
				.and_then(Value::as_u64)
				.unwrap_or(0) as i64
				+ 1;

			let context = payload.get("context").cloned().unwrap_or_else(|| json!({ "triggerKind": 0 }));

			match url::Url::parse(&uri_str) {
				Ok(uri) => {
					let position = PositionDTO { LineNumber:line as u32, Column:character as u32 };

					match runtime.Environment.ProvideInlineCompletionItems(uri, position, context).await {
						Ok(Some(result)) => {
							let items = result
								.get("items")
								.cloned()
								.unwrap_or_else(|| if result.is_array() { result } else { json!([]) });

							Ok(json!({ "items": items }))
						},

						Ok(None) => Ok(json!({ "items": [] })),

						Err(e) => {
							crate::dev_log!("ipc", "warn: language:provideInlineCompletions error: {}", e);

							Ok(json!({ "items": [] }))
						},
					}
				},

				Err(_) => Ok(json!({ "items": [] })),
			}
		},

		"languages:getAll" | "languages:getEncodedLanguageId" => {
			crate::dev_log!("extensions", "languages: {} (→ Cocoon)", command);

			let payload = arguments.into_iter().next().unwrap_or(Value::Null);

			if !crate::Vine::Client::IsClientConnected::Fn("cocoon-main") {
				Ok(Value::Array(Vec::new()))
			} else {
				Ok(
					crate::Vine::Client::SendRequest::Fn("cocoon-main", command.to_string(), payload, 5_000)
						.await
						.unwrap_or(Value::Array(Vec::new())),
				)
			}
		},

		"language:prepareCallHierarchy"
		| "language:provideCallHierarchyIncomingCalls"
		| "language:provideCallHierarchyOutgoingCalls" => {
			let payload = cocoon_payload(arguments);

			let _ = crate::Vine::Client::WaitForClientConnection::Fn("cocoon-main", 3000).await;

			crate::Vine::Client::SendRequest::Fn("cocoon-main", command.to_string(), payload, 15_000)
				.await
				.map_err(|e| format!("Cocoon error: {:?}", e))
		},

		"language:prepareTypeHierarchy"
		| "language:provideTypeHierarchySupertypes"
		| "language:provideTypeHierarchySubtypes" => {
			let payload = cocoon_payload(arguments);

			let _ = crate::Vine::Client::WaitForClientConnection::Fn("cocoon-main", 3000).await;

			crate::Vine::Client::SendRequest::Fn("cocoon-main", command.to_string(), payload, 15_000)
				.await
				.map_err(|e| format!("Cocoon error: {:?}", e))
		},

		"language:provideLinkedEditingRanges" => {
			let payload = cocoon_payload(arguments);

			let _ = crate::Vine::Client::WaitForClientConnection::Fn("cocoon-main", 3000).await;

			crate::Vine::Client::SendRequest::Fn("cocoon-main", command.to_string(), payload, 15_000)
				.await
				.map_err(|e| format!("Cocoon error: {:?}", e))
		},

		_ => Err(format!("Unknown language command: {}", command)),
	}
}
