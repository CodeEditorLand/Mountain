#![allow(non_snake_case, unused_variables, dead_code, unused_imports)]

use std::{future::Future, pin::Pin, sync::Arc};

use CommonLibrary::{Diagnostic::DiagnosticManager::DiagnosticManager, Environment::Requires::Requires};

use serde_json::{Value, json};

use tauri::Runtime;

use crate::{RunTime::ApplicationRunTime::ApplicationRunTime, Track::Effect::MappedEffectType::MappedEffect};

pub fn CreateEffect<R:Runtime>(MethodName:&str, Parameters:Value) -> Option<Result<MappedEffect, String>> {

	match MethodName {

		"Diagnostic.Set" => {

			let effect =
				move |run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {

					Box::pin(async move {
						let provider:Arc<dyn DiagnosticManager> = run_time.Environment.Require();
						let owner = Parameters.get(0).and_then(Value::as_str).unwrap_or("").to_string();
						let entries = Parameters.get(1).cloned().unwrap_or_default();
						provider
							.SetDiagnostics(owner, entries)
							.await
							.map(|_| json!(null))
							.map_err(|e| e.to_string())
					})
				};

			Some(Ok(Box::new(effect)))
		},

		"Diagnostic.Clear" => {

			let effect =
				move |run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {

					Box::pin(async move {
						let provider:Arc<dyn DiagnosticManager> = run_time.Environment.Require();
						let owner = Parameters.get(0).and_then(Value::as_str).unwrap_or("").to_string();
						provider
							.ClearDiagnostics(owner)
							.await
							.map(|_| json!(null))
							.map_err(|e| e.to_string())
					})
				};

			Some(Ok(Box::new(effect)))
		},

		_ => None,
	}
}
