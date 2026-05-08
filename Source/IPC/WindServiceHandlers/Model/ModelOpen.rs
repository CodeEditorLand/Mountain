#![allow(non_snake_case)]

//! Open a text model: read content from disk, derive language
//! ID from the extension, register the resulting
//! `DocumentStateDTO` in `ApplicationState.Feature.Documents`,
//! and return `{ uri, content, version, languageId }` to Wind.
//!
//! Version starts at 1 for fresh opens; an existing entry
//! increments instead of resetting so concurrent re-opens
//! don't desync VS Code's TextDocument observers.

use std::sync::Arc;

use serde_json::{Value, json};

use crate::{
	ApplicationState::DTO::DocumentStateDTO::DocumentStateDTO,
	RunTime::ApplicationRunTime::ApplicationRunTime,
};

pub async fn ModelOpen(RunTime:Arc<ApplicationRunTime>, Arguments:Vec<Value>) -> Result<Value, String> {
	let Uri = Arguments
		.first()
		.and_then(|V| V.as_str())
		.ok_or("model:open requires uri".to_string())?
		.to_owned();

	let FilePath = if let Some(stripped) = Uri.strip_prefix("file://") {
		stripped.to_owned()
	} else {
		Uri.clone()
	};

	let Content = tokio::fs::read_to_string(&FilePath).await.unwrap_or_default();

	let LanguageId = std::path::Path::new(&FilePath)
		.extension()
		.and_then(|E| E.to_str())
		.map(|Ext| {
			match Ext {
				"rs" => "rust",
				"ts" | "tsx" => "typescript",
				"js" | "jsx" | "mjs" | "cjs" => "javascript",
				"json" | "jsonc" => "json",
				"toml" => "toml",
				"yaml" | "yml" => "yaml",
				"md" => "markdown",
				"html" | "htm" => "html",
				"css" | "scss" | "less" => "css",
				"sh" | "bash" | "zsh" => "shellscript",
				"py" => "python",
				"go" => "go",
				"c" | "h" => "c",
				"cpp" | "cc" | "cxx" | "hpp" => "cpp",
				_ => "plaintext",
			}
		})
		.unwrap_or("plaintext")
		.to_owned();

	let Version = RunTime
		.Environment
		.ApplicationState
		.Feature
		.Documents
		.Get(&Uri)
		.map(|D| D.Version + 1)
		.unwrap_or(1);

	if let Ok(ParsedUri) = url::Url::parse(&Uri) {
		let Lines:Vec<String> = Content.lines().map(|L| L.to_owned()).collect();

		let Eol = if Content.contains("\r\n") { "\r\n" } else { "\n" }.to_owned();

		let Document = DocumentStateDTO {
			URI:ParsedUri,

			LanguageIdentifier:LanguageId.clone(),

			Version,

			Lines,

			EOL:Eol,

			IsDirty:false,

			Encoding:"utf-8".to_owned(),

			VersionIdentifier:Version,
		};

		RunTime
			.Environment
			.ApplicationState
			.Feature
			.Documents
			.AddOrUpdate(Uri.clone(), Document);
	}

	Ok(json!({
		"uri": Uri,
		"content": Content,
		"version": Version,
		"languageId": LanguageId,
	}))
}
