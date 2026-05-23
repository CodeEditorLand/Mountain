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

pub async fn Fn(RunTime:Arc<ApplicationRunTime>, Arguments:Vec<Value>) -> Result<Value, String> {
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

	let Content = tokio::fs::read_to_string(&FilePath)
		.await
		.map_err(|Error| format!("model:open read failed for {}: {}", FilePath, Error))?;

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

	// Track the active document so `nativeHost:getWindows` and the Cocoon
	// `vscode.window.activeTextEditor` shim stay in sync.
	RunTime
		.Environment
		.ApplicationState
		.Workspace
		.SetActiveDocumentURI(Some(Uri.clone()));

	// Fire-and-forget: notify Cocoon so its `window.activeTextEditor` shim
	// updates. The notification is cheap (~gRPC framing + 1 JSON field); if
	// Cocoon isn't connected yet the `SendNotification` error is swallowed.
	let NotifyUri = Uri.clone();

	let NotifyLang = LanguageId.clone();

	let NotifyVer = Version;

	tokio::spawn(async move {
		let _ = crate::Vine::Client::SendNotification::Fn(
			"cocoon-main".to_string(),
			"window.didChangeActiveTextEditor".to_string(),
			serde_json::json!({ "uri": NotifyUri, "languageId": NotifyLang, "version": NotifyVer }),
		)
		.await;
	});

	Ok(json!({
		"uri": Uri,
		"content": Content,
		"version": Version,
		"languageId": LanguageId,
	}))
}
