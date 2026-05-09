#![allow(non_snake_case)]

//! Bulk snapshot of every open text model. Used by Wind on
//! workbench restore to repopulate the Monaco model registry
//! without per-tab `ModelOpen` round-trips.

use std::sync::Arc;

use serde_json::{Value, json};

use crate::RunTime::ApplicationRunTime::ApplicationRunTime;

pub async fn ModelGetAll(RunTime:Arc<ApplicationRunTime>) -> Result<Value, String> {

	let All = RunTime
		.Environment
		.ApplicationState
		.Feature
		.Documents
		.GetAll()
		.into_iter()
		.map(|(Uri, Document)| {
			json!({
				"uri": Uri,
				"content": Document.Lines.join(&Document.EOL),
				"version": Document.Version,
				"languageId": Document.LanguageIdentifier,
			})
		})
		.collect::<Vec<_>>();

	Ok(Value::Array(All))
}
