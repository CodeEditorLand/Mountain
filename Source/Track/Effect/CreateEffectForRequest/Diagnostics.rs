use std::sync::Arc;

use CommonLibrary::{Diagnostic::DiagnosticManager::DiagnosticManager, Environment::Requires::Requires};
use serde_json::{Value, json};
use tauri::Runtime;

use crate::Track::Effect::{
	CreateEffectForRequest::Utilities::Params::{string_at, val_at},
	MappedEffectType::MappedEffect,
};

pub fn CreateEffect<R:Runtime>(MethodName:&str, Parameters:Value) -> Option<Result<MappedEffect, String>> {
	match MethodName {
		"Diagnostic.Set" => {
			crate::effect!(run_time, {
				let provider:Arc<dyn DiagnosticManager> = run_time.Environment.Require();
				let owner = string_at(&Parameters, 0);
				let entries = val_at(&Parameters, 1);
				provider
					.SetDiagnostics(owner, entries)
					.await
					.map(|_| json!(null))
					.map_err(|e| e.to_string())
			})
		},

		"Diagnostic.Clear" => {
			crate::effect!(run_time, {
				let provider:Arc<dyn DiagnosticManager> = run_time.Environment.Require();
				let owner = string_at(&Parameters, 0);
				provider
					.ClearDiagnostics(owner)
					.await
					.map(|_| json!(null))
					.map_err(|e| e.to_string())
			})
		},

		_ => None,
	}
}
