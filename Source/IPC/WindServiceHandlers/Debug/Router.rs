//! Debug command router - Mountain-native pre-processing + Cocoon forward.

use std::sync::Arc;

use serde_json::{Value, json};
use tauri::Emitter;

use crate::{
	IPC::WindServiceHandlers::Utilities::JsonValueHelpers::{arg_string_or, arg_val},
	RunTime::ApplicationRunTime::ApplicationRunTime,
	dev_log,
};

/// Build a Cocoon-forward payload from arguments (mirrors the
/// `cocoon_payload` helper in DispatchMatch.rs).
fn cocoon_payload(args:Vec<Value>) -> Value {
	match args.len() {
		0 => Value::Null,

		1 => args.into_iter().next().unwrap(),

		_ => Value::Array(args),
	}
}

/// Forward a command result to Cocoon via gRPC.
async fn forward_to_cocoon(tag:&str, command:&str, arguments:Vec<Value>) -> Result<Value, String> {
	dev_log!("ipc", "{}: {} (→ Cocoon)", tag, command);

	let payload = cocoon_payload(arguments);

	if crate::Vine::Client::IsClientConnected::Fn("cocoon-main") {
		Ok(
			crate::Vine::Client::SendRequest::Fn("cocoon-main", command.to_string(), payload, 10_000)
				.await
				.unwrap_or(Value::Null),
		)
	} else {
		dev_log!("ipc", "{}: Cocoon disconnected - returning Null fallback", tag);

		Ok(Value::Null)
	}
}

/// Routes debug commands. Returns Some(result) for handled commands,
/// None otherwise.
pub(crate) async fn route(
	ApplicationHandle:tauri::AppHandle,

	RunTime:Arc<ApplicationRunTime>,

	command:&str,

	Arguments:Vec<Value>,
) -> Option<Result<Value, String>> {
	match command {
		// `debug:startDebugging`: call DebugProvider::StartDebugging
		// (via the Debug.Start effect) to register the session and
		// optionally spawn the DAP adapter before forwarding to
		// Cocoon so vscode.debug extension listeners see a live
		// session.
		"debug:startDebugging" => {
			let FolderUriStr = arg_string_or(&Arguments, 0, "");

			let Config = arg_val(&Arguments, 1);

			let DebugStartParams = json!([FolderUriStr, Config]);

			let StartEffect = crate::Track::Effect::CreateEffectForRequest::Debug::CreateEffect::<tauri::Wry>(
				"Debug.Start",
				DebugStartParams,
			);

			if let Some(EffectResult) = StartEffect {
				match EffectResult {
					Ok(task) => {
						if let Err(e) = task(RunTime.clone()).await {
							dev_log!("exthost", "warn: debug:startDebugging effect failed: {}", e);
						}
					},

					Err(e) => {
						dev_log!("exthost", "warn: debug:startDebugging effect build error: {}", e);
					},
				}
			}

			Some(forward_to_cocoon("debug", command, Arguments).await)
		},

		// `debug:addBreakpoints`: store in ApplicationState and emit
		// sky://debug/breakpointsChanged for renderer decorations,
		// then forward to Cocoon for onDidChangeBreakpoints.
		"debug:addBreakpoints" => {
			if let Some(serde_json::Value::Array(RawBreakpoints)) = Arguments.first() {
				let Entries:Vec<crate::ApplicationState::State::FeatureState::Debug::DebugState::BreakpointEntry> =
					RawBreakpoints
						.iter()
						.filter_map(|Raw| {
							let Id = Raw
								.get("id")
								.or_else(|| Raw.get("Id"))
								.and_then(serde_json::Value::as_str)
								.map(str::to_string)?;

							let Kind = Raw
								.get("type")
								.or_else(|| Raw.get("kind"))
								.and_then(serde_json::Value::as_str)
								.unwrap_or("source")
								.to_string();

							let Uri = Raw
								.get("uri")
								.or_else(|| Raw.get("source").and_then(|S| S.get("uri")))
								.and_then(serde_json::Value::as_str)
								.unwrap_or("")
								.to_string();

							let Line = Raw
								.get("lineNumber")
								.or_else(|| Raw.get("line"))
								.and_then(serde_json::Value::as_u64)
								.unwrap_or(0);

							let Column = Raw.get("column").and_then(serde_json::Value::as_u64);

							let Enabled = Raw.get("enabled").and_then(serde_json::Value::as_bool).unwrap_or(true);

							Some(
								crate::ApplicationState::State::FeatureState::Debug::DebugState::BreakpointEntry {
									id:Id,
									kind:Kind,
									uri:Uri,
									line:Line,
									column:Column,
									enabled:Enabled,
									raw:Raw.clone(),
								},
							)
						})
						.collect();

				if !Entries.is_empty() {
					RunTime.Environment.ApplicationState.Feature.Debug.AddBreakpoints(Entries);

					let _ = ApplicationHandle.emit(
						"sky://debug/breakpointsChanged",
						json!({
							"added": RawBreakpoints,
							"removed": [],
							"changed": [],
						}),
					);
				}
			}

			Some(forward_to_cocoon("debug", command, Arguments).await)
		},

		// `debug:getBreakpoints`: served from Mountain's local store;
		// no Cocoon round-trip needed.
		"debug:getBreakpoints" => {
			let Bps = RunTime.Environment.ApplicationState.Feature.Debug.GetBreakpoints();

			Some(Ok(json!(Bps)))
		},

		// `debug:removeBreakpoints`: evict from local store and emit
		// change event, then forward to Cocoon.
		"debug:removeBreakpoints" => {
			if let Some(serde_json::Value::Array(RawIds)) = Arguments.first() {
				let Ids:Vec<String> = RawIds.iter().filter_map(|V| V.as_str().map(str::to_string)).collect();

				if !Ids.is_empty() {
					RunTime.Environment.ApplicationState.Feature.Debug.RemoveBreakpoints(&Ids);

					let _ = ApplicationHandle.emit(
						"sky://debug/breakpointsChanged",
						json!({
							"added": [],
							"removed": RawIds,
							"changed": [],
						}),
					);
				}
			}

			Some(forward_to_cocoon("debug", command, Arguments).await)
		},

		"debug:stopDebugging" | "debug:getSessions" => Some(forward_to_cocoon("debug", command, Arguments).await),

		_ => None,
	}
}
