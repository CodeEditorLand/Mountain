#![allow(non_snake_case)]

//! Replace an open model's content. Increments `Version`,
//! recomputes `Lines`, marks `IsDirty=true`. Mirrors VS Code's
//! `TextDocument.update(...)` semantics - the Monaco model
//! observers see a single coherent edit, not partial state.
//!
//! Errors when the URI isn't open; callers must `ModelOpen`
//! first.

use std::sync::Arc;

use serde_json::{Value, json};

use crate::RunTime::ApplicationRunTime::ApplicationRunTime;

pub async fn Fn(RunTime:Arc<ApplicationRunTime>, Arguments:Vec<Value>) -> Result<Value, String> {
	let Uri = Arguments
		.first()
		.and_then(|V| V.as_str())
		.ok_or("model:updateContent requires uri".to_string())?
		.to_owned();

	let NewContent = Arguments
		.get(1)
		.and_then(|V| V.as_str())
		.ok_or("model:updateContent requires content".to_string())?
		.to_owned();

	let (NewVersion, LanguageId) = match RunTime.Environment.ApplicationState.Feature.Documents.Get(&Uri) {
		None => return Err(format!("model:updateContent - model not open: {}", Uri)),

		Some(mut Document) => {
			Document.Version += 1;

			Document.Lines = NewContent.lines().map(|L| L.to_owned()).collect();

			Document.IsDirty = true;

			let Version = Document.Version;

			let LangId = Document.LanguageIdentifier.clone();

			RunTime
				.Environment
				.ApplicationState
				.Feature
				.Documents
				.AddOrUpdate(Uri.clone(), Document);

			(Version, LangId)
		},
	};

	// Notify Cocoon so `onDidChangeTextDocument` fires with the new content.
	let UriForCocoon = Uri.clone();
	let ContentForCocoon = NewContent.clone();
	let VersionForCocoon = NewVersion;
	tokio::spawn(async move {
		let _ = crate::Vine::Client::SendNotification::Fn(
			"cocoon-main".to_string(),
			"$acceptModelChanged".to_string(),
			serde_json::json!([
				{ "external": UriForCocoon, "$mid": 1 },
				{ "content": ContentForCocoon, "versionId": VersionForCocoon, "isDirty": true, "changes": [] }
			]),
		)
		.await;
	});

	Ok(json!({
		"uri": Uri,
		"content": NewContent,
		"version": NewVersion,
		"languageId": LanguageId,
	}))
}
