pub fn Matches(MethodName:&str) -> bool {
	match MethodName {
		Diagnostic.Set, Diagnostic.Clear => true,
		_ => false,
	}
}

use std::sync::Arc;

use CommonLibrary::{Diagnostic::DiagnosticManager::DiagnosticManager, Environment::Requires::Requires};
use serde_json::{Value, json};
use tauri::Runtime;

use crate::Track::Effect::{
	CreateEffectForRequest::Utilities::Params::{string_at, val_at},
	MappedEffectType::MappedEffect,
};
pub fn CreateEffect<R:Runtime>(MethodName:&str, Parameters:&Value) -> Option<Result<MappedEffect, String>> {
	match MethodName {
		"Diagnostic.Set" => {
			crate::effect!(run_time, {
				let provider:Arc<dyn DiagnosticManager> = run_time.Environment.Require();

				let owner = string_at(&Parameters, 0);

				let entries = val_at(&Parameters, 1);

				let Result = provider
					.SetDiagnostics(owner.clone(), entries.clone())
					.await
					.map(|_| json!(null))
					.map_err(|e| e.to_string());

				// Fan back to Cocoon so peer extensions hooking
				// `vscode.languages.onDidChangeDiagnostics` observe the
				// change. The matching subscriber lives at
				// `Languages/Namespace.ts:1140` on the
				// `diagnostics.didChange` Emitter channel. Extract the
				// list of changed URIs from the entries payload
				// (`entries` is `[[uriString, diagnostics[]], ...]`).
				let Uris:Vec<Value> = entries
					.as_array()
					.map(|Arr| {
						Arr.iter()
							.filter_map(|Pair| Pair.as_array().and_then(|P| P.first().cloned()))
							.collect()
					})
					.unwrap_or_default();

				let _ = ::Vine::Client::SendNotification::Fn(
					"cocoon-main".to_string(),
					"$acceptDiagnosticsChanged".to_string(),
					json!({ "owner": owner, "uris": Uris }),
				)
				.await;

				Result
			})
		},

		"Diagnostic.Clear" => {
			crate::effect!(run_time, {
				let provider:Arc<dyn DiagnosticManager> = run_time.Environment.Require();

				let owner = string_at(&Parameters, 0);

				let Result = provider
					.ClearDiagnostics(owner.clone())
					.await
					.map(|_| json!(null))
					.map_err(|e| e.to_string());

				// Clear translates to "every URI previously held by this
				// owner now has zero diagnostics". Without knowing the
				// prior URI set we send an empty `uris` list - the
				// `onDidChangeDiagnostics` subscriber should re-query
				// `getDiagnostics(uri)` if it needs the new state.
				let _ = ::Vine::Client::SendNotification::Fn(
					"cocoon-main".to_string(),
					"$acceptDiagnosticsChanged".to_string(),
					json!({ "owner": owner, "uris": [] }),
				)
				.await;

				Result
			})
		},

		_ => None,
	}
}
