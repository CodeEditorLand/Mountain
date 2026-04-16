#![allow(non_snake_case)]

//! Model (Text Model Registry) domain handlers for Wind IPC.

use std::sync::Arc;

use serde_json::{Value, json};

use crate::RunTime::ApplicationRunTime::ApplicationRunTime;

/// Open a text model: read content from disk and register in DocumentState.
/// Returns { uri, content, version, languageId }.
pub async fn handle_model_open(Runtime:Arc<ApplicationRunTime>, Args:Vec<Value>) -> Result<Value, String> {
	let Uri = Args
		.first()
		.and_then(|V| V.as_str())
		.ok_or("model:open requires uri".to_string())?
		.to_owned();

	let FilePath = if Uri.starts_with("file://") {
		Uri.trim_start_matches("file://").to_owned()
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

	let Version = Runtime
		.Environment
		.ApplicationState
		.Feature
		.Documents
		.Get(&Uri)
		.map(|D| D.Version + 1)
		.unwrap_or(1);

	{
		use crate::ApplicationState::DTO::DocumentStateDTO::DocumentStateDTO;

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

			Runtime
				.Environment
				.ApplicationState
				.Feature
				.Documents
				.AddOrUpdate(Uri.clone(), Document);
		}
	}

	Ok(json!({
		"uri": Uri,
		"content": Content,
		"version": Version,
		"languageId": LanguageId,
	}))
}

/// Close a text model and remove it from DocumentState.
pub async fn handle_model_close(Runtime:Arc<ApplicationRunTime>, Args:Vec<Value>) -> Result<Value, String> {
	let Uri = Args
		.first()
		.and_then(|V| V.as_str())
		.ok_or("model:close requires uri".to_string())?;

	Runtime.Environment.ApplicationState.Feature.Documents.Remove(Uri);
	Ok(Value::Null)
}

/// Get the current snapshot of an open text model, or null if not open.
pub async fn handle_model_get(Runtime:Arc<ApplicationRunTime>, Args:Vec<Value>) -> Result<Value, String> {
	let Uri = Args
		.first()
		.and_then(|V| V.as_str())
		.ok_or("model:get requires uri".to_string())?;

	match Runtime.Environment.ApplicationState.Feature.Documents.Get(Uri) {
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

/// Return all currently open text models.
pub async fn handle_model_get_all(Runtime:Arc<ApplicationRunTime>) -> Result<Value, String> {
	let All = Runtime
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

/// Update the content of an open text model, incrementing its version.
pub async fn handle_model_update_content(Runtime:Arc<ApplicationRunTime>, Args:Vec<Value>) -> Result<Value, String> {
	let Uri = Args
		.first()
		.and_then(|V| V.as_str())
		.ok_or("model:updateContent requires uri".to_string())?
		.to_owned();

	let NewContent = Args
		.get(1)
		.and_then(|V| V.as_str())
		.ok_or("model:updateContent requires content".to_string())?
		.to_owned();

	let (NewVersion, LanguageId) = match Runtime.Environment.ApplicationState.Feature.Documents.Get(&Uri) {
		None => return Err(format!("model:updateContent - model not open: {}", Uri)),
		Some(mut Document) => {
			Document.Version += 1;
			Document.Lines = NewContent.lines().map(|L| L.to_owned()).collect();
			Document.IsDirty = true;
			let Version = Document.Version;
			let LangId = Document.LanguageIdentifier.clone();
			Runtime
				.Environment
				.ApplicationState
				.Feature
				.Documents
				.AddOrUpdate(Uri.clone(), Document);
			(Version, LangId)
		},
	};

	Ok(json!({
		"uri": Uri,
		"content": NewContent,
		"version": NewVersion,
		"languageId": LanguageId,
	}))
}
