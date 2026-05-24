use std::sync::Arc;

use CommonLibrary::{Diagnostic::DiagnosticManager::DiagnosticManager, Environment::Requires::Requires};
use serde_json::{Value, json};
use tauri::Runtime;

use crate::Track::Effect::{
	CreateEffectForRequest::Utilities::Params::{StringAt, ValAt},
	MappedEffectType::MappedEffect,
};

pub fn Fn<R:Runtime>(MethodName:&str, Parameters:Value) -> Option<Result<MappedEffect, String>> {
	match MethodName {
		"Diagnostic.Set" => {
			crate::effect!(RunTime, {
				let Provider:Arc<dyn DiagnosticManager> = RunTime.Environment.Require();
				let owner = StringAt(&Parameters, 0);
				let entries = ValAt(&Parameters, 1);
				provider
					.SetDiagnostics(owner, entries)
					.await
					.map(|_| json!(null))
					.map_err(|E| e.to_string())
			})
		},

		"Diagnostic.Clear" => {
			crate::effect!(RunTime, {
				let Provider:Arc<dyn DiagnosticManager> = RunTime.Environment.Require();
				let owner = StringAt(&Parameters, 0);
				provider
					.ClearDiagnostics(owner)
					.await
					.map(|_| json!(null))
					.map_err(|E| e.to_string())
			})
		},

		_ => None,
	}
}
