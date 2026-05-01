#![allow(non_snake_case, unused_variables, dead_code, unused_imports)]

//! Text model registry and TextFile handlers - open, close, get, update,
//! read, write, save.

use std::sync::Arc;

use serde_json::{Value, json};

use crate::{RunTime::ApplicationRunTime::ApplicationRunTime, dev_log};

// ============================================================================
// Model (Text Model Registry) Handlers
// ============================================================================

/// Open a text model: read content from disk and register in DocumentState.
/// Returns { uri, content, version, languageId }.
pub async fn ModelOpen(runtime:Arc<ApplicationRunTime>, args:Vec<Value>) -> Result<Value, String> {
	let Uri = args
		.first()
		.and_then(|V| V.as_str())
		.ok_or("model:open requires uri".to_string())?
		.to_owned();

	// Derive file path from URI
	let FilePath = if Uri.starts_with("file://") {
		Uri.trim_start_matches("file://").to_owned()
	} else {
		Uri.clone()
	};

	// Read file content from disk
	let Content = tokio::fs::read_to_string(&FilePath).await.unwrap_or_default();

	// Detect language from extension
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

	// Determine next version (1 if new, increment if exists)
	let Version = runtime
		.Environment
		.ApplicationState
		.Feature
		.Documents
		.Get(&Uri)
		.map(|D| D.Version + 1)
		.unwrap_or(1);

	// Register in document state
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

			runtime
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
pub async fn ModelClose(runtime:Arc<ApplicationRunTime>, args:Vec<Value>) -> Result<Value, String> {
	let Uri = args
		.first()
		.and_then(|V| V.as_str())
		.ok_or("model:close requires uri".to_string())?;

	runtime.Environment.ApplicationState.Feature.Documents.Remove(Uri);
	Ok(Value::Null)
}

/// Get the current snapshot of an open text model, or null if not open.
pub async fn ModelGet(runtime:Arc<ApplicationRunTime>, args:Vec<Value>) -> Result<Value, String> {
	let Uri = args
		.first()
		.and_then(|V| V.as_str())
		.ok_or("model:get requires uri".to_string())?;

	match runtime.Environment.ApplicationState.Feature.Documents.Get(Uri) {
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
pub async fn ModelGetAll(runtime:Arc<ApplicationRunTime>) -> Result<Value, String> {
	let All = runtime
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
pub async fn ModelUpdateContent(runtime:Arc<ApplicationRunTime>, args:Vec<Value>) -> Result<Value, String> {
	let Uri = args
		.first()
		.and_then(|V| V.as_str())
		.ok_or("model:updateContent requires uri".to_string())?
		.to_owned();

	let NewContent = args
		.get(1)
		.and_then(|V| V.as_str())
		.ok_or("model:updateContent requires content".to_string())?
		.to_owned();

	let (NewVersion, LanguageId) = match runtime.Environment.ApplicationState.Feature.Documents.Get(&Uri) {
		None => return Err(format!("model:updateContent - model not open: {}", Uri)),
		Some(mut Document) => {
			Document.Version += 1;
			Document.Lines = NewContent.lines().map(|L| L.to_owned()).collect();
			Document.IsDirty = true;
			let Version = Document.Version;
			let LangId = Document.LanguageIdentifier.clone();
			runtime
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

// ============================================================================
// TextFile Handlers
// ============================================================================

/// Read a text file from disk.
pub async fn TextfileRead(_runtime:Arc<ApplicationRunTime>, args:Vec<Value>) -> Result<Value, String> {
	let Path = args
		.first()
		.and_then(|V| V.as_str())
		.ok_or_else(|| "textFile:read requires path as first argument".to_string())?;

	tokio::fs::read_to_string(Path)
		.await
		.map(Value::String)
		.map_err(|Error| format!("textFile:read failed: {}", Error))
}

/// Write text to a file on disk.
pub async fn TextfileWrite(_runtime:Arc<ApplicationRunTime>, args:Vec<Value>) -> Result<Value, String> {
	let Path = args
		.first()
		.and_then(|V| V.as_str())
		.ok_or_else(|| "textFile:write requires path as first argument".to_string())?;
	let Content = args.get(1).and_then(|V| V.as_str()).unwrap_or("").to_string();

	tokio::fs::write(Path, Content.as_bytes())
		.await
		.map(|()| Value::Null)
		.map_err(|Error| format!("textFile:write failed: {}", Error))
}

/// Save a document - forward save intent to Sky frontend.
pub async fn TextfileSave(_runtime:Arc<ApplicationRunTime>, args:Vec<Value>) -> Result<Value, String> {
	// Actual disk write happens via textFile:write; this is a UI-dirty-state hint.
	let _Uri = args.first().and_then(|V| V.as_str()).unwrap_or("").to_string();
	dev_log!("vfs", "textFile:save uri={:?}", _Uri);
	Ok(Value::Null)
}
