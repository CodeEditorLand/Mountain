#![allow(non_snake_case)]

//! Snapshot a single open text model. Returns
//! `{ uri, content, version, languageId }` or `null` when the
//! URI isn't currently open. `content` is rejoined from
//! `Lines` using the document's EOL so the wire shape matches
//! VS Code's `TextDocument.getText()`.

use std::sync::Arc;

use serde_json::{Value, json};

use crate::RunTime::ApplicationRunTime::ApplicationRunTime;

pub async fn ModelGet(RunTime:Arc<ApplicationRunTime>, Arguments:Vec<Value>) -> Result<Value, String> {
	let Uri = Arguments
		.first()
		.and_then(|V| V.as_str())
		.ok_or("model:get requires uri".to_string())?;

	match RunTime.Environment.ApplicationState.Feature.Documents.Get(Uri) {
		None => Ok(Value::Null),

		Some(Document) => {
			Ok(json!({
				"uri": Uri,
				"content": Document.Lines.join(&Document.EOL),
				"version": Document.Version,
				"languageId": Document.LanguageIdentifier,
			}))
		},
	}
}
