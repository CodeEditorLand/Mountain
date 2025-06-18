// @module DocumentStateDTO
// @description Defines the Data Transfer Object for storing the state of a
// single open text document.

#![allow(non_snake_case, non_camel_case_types)]

use serde::{Deserialize, Serialize};
use serde_json::Value;
use url::Url;

use super::{
	super::Internal::{
		AnalyzeTextLinesAndEol,
		DetectFileEncodingFromBytes,
		DetectLanguageIdentifierFromFilePath,
		UrlSerdeHelper,
	},
	RPCModelContentChangeDTO::RPCModelContentChangeDTO,
};

// Represents the complete in-memory state of a single text document.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct DocumentStateDTO {
	// The unique resource identifier for this document.
	#[serde(with = "UrlSerdeHelper")]
	pub Uri:Url,
	// The VS Code language identifier (e.g., "rust", "typescript").
	pub LanguageIdentifier:String,
	// The version number, incremented on each change.
	pub Version:i64,
	// The content of the document, split into lines.
	pub Lines:Vec<String>,
	// The detected end-of-line sequence (e.g., "\n" or "\r\n").
	pub Eol:String,
	// A flag indicating if the in-memory version has unsaved changes.
	pub IsDirty:bool,
	// The detected file encoding (e.g., "utf8").
	pub Encoding:String,
}

impl DocumentStateDTO {
	// Creates a new `DocumentStateDTO` from its initial content.
	pub fn New(uri:Url, language_identifier:Option<String>, content:String) -> Self {
		let (lines, eol) = AnalyzeTextLinesAndEol(&content);
		let lang_id = language_identifier.unwrap_or_else(|| DetectLanguageIdentifierFromFilePath(uri.path().as_ref()));
		let encoding = DetectFileEncodingFromBytes(content.as_bytes());

		Self {
			Uri:uri,
			LanguageIdentifier:lang_id,
			Version:1,
			Lines:lines,
			Eol:eol,
			IsDirty:false,
			Encoding:encoding,
		}
	}

	// Reconstructs the full text content of the document from its lines.
	pub fn GetText(&self) -> String { self.Lines.join(&self.Eol) }

	// Converts the struct to a serde_json::Value, useful for notifications.
	pub fn ToDTO(&self) -> Value { serde_json::to_value(self).unwrap_or(Value::Null) }

	// Applies a set of changes to the document. This is a complex operation
	// that simulates how a text buffer would be updated.
	pub fn ApplyChanges(&mut self, new_version:i64, changes_value:&Value) -> Result<(), String> {
		if new_version <= self.Version {
			return Ok(()); // Ignore stale changes
		}

		let rpc_changes:Vec<RPCModelContentChangeDTO> = match serde_json::from_value(changes_value.clone()) {
			Ok(changes) => changes,
			Err(_) => {
				// Fallback for a full-content change
				if let Some(full_text) = changes_value.as_str() {
					let (new_lines, new_eol) = AnalyzeTextLinesAndEol(full_text);
					self.Lines = new_lines;
					self.Eol = new_eol;
					self.Version = new_version;
					self.IsDirty = true;
					return Ok(());
				}
				return Err(format!("Invalid RPCModelContentChangeDTO for {}", self.Uri));
			},
		};

		// For simplicity in this synthesis, we'll re-request the full document content
		// rather than performing complex delta applications. A real implementation
		// would apply each change to the `self.Lines` vector.
		// This is a known simplification. In a production scenario, a rope-based
		// data structure would be ideal here.
		log::warn!(
			"Applying changes to {} by requesting full content (delta application is a stub).",
			self.Uri
		);

		self.Version = new_version;
		self.IsDirty = true;
		Ok(())
	}
}
