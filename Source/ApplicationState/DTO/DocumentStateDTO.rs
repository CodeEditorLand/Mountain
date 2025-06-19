//! # DocumentStateDTO
//!
//! Defines the Data Transfer Object for storing the state of a single open
//! text document in memory.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use url::Url;

use super::RPCModelContentChangeDTO::RPCModelContentChangeDTO;
use crate::ApplicationState::Internal::{
	AnalyzeTextLinesAndEOL,
	URLSerializationHelper, /* DetectFileEncodingFromBytes,
	                         * DetectLanguageIdentifierFromFilePath, */
};

/// Represents the complete in-memory state of a single text document.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct DocumentStateDTO {
	/// The unique resource identifier for this document.
	#[serde(with = "URLSerializationHelper")]
	pub URI:Url,
	/// The VS Code language identifier (e.g., "rust", "typescript").
	pub LanguageIdentifier:String,
	/// The version number, incremented on each change.
	pub Version:i64,
	/// The content of the document, split into lines.
	pub Lines:Vec<String>,
	/// The detected end-of-line sequence (e.g., `\n` or `\r\n`).
	pub EOL:String,
	/// A flag indicating if the in-memory version has unsaved changes.
	pub IsDirty:bool,
	/// The detected file encoding (e.g., "utf8").
	pub Encoding:String,
}

impl DocumentStateDTO {
	/// Creates a new `DocumentStateDTO` from its initial content.
	pub fn Create(URI:Url, LanguageIdentifier:Option<String>, Content:String) -> Self {
		let (Lines, EOL) = AnalyzeTextLinesAndEOL(&Content);
		// A real implementation would have more robust language/encoding detection.
		let LanguageID = LanguageIdentifier.unwrap_or_else(|| "plaintext".to_string());
		let Encoding = "utf8".to_string(); // Stub for encoding detection

		Self {
			URI,
			LanguageIdentifier:LanguageID,
			Version:1,
			Lines,
			EOL,
			IsDirty:false,
			Encoding,
		}
	}

	/// Reconstructs the full text content of the document from its lines.
	pub fn GetText(&self) -> String { self.Lines.join(&self.EOL) }

	/// Converts the struct to a `serde_json::Value`, useful for notifications.
	pub fn ToDTO(&self) -> Value { serde_json::to_value(self).unwrap_or(Value::Null) }

	/// Applies a set of changes to the document.
	///
	/// This is a complex operation that simulates how a text buffer would be
	/// updated. For this implementation, it is simplified.
	pub fn ApplyChanges(&mut self, NewVersion:i64, ChangesValue:&Value) -> Result<(), String> {
		if NewVersion <= self.Version {
			return Ok(()); // Ignore stale changes
		}

		let RPCChanges:Vec<RPCModelContentChangeDTO> = match serde_json::from_value(ChangesValue.clone()) {
			Ok(changes) => changes,
			Err(_) => {
				// Fallback for a full-content change, which is a common scenario.
				if let Some(FullText) = ChangesValue.as_str() {
					let (NewLines, NewEOL) = AnalyzeTextLinesAndEOL(FullText);
					self.Lines = NewLines;
					self.EOL = NewEOL;
					self.Version = NewVersion;
					self.IsDirty = true;
					return Ok(());
				}
				return Err(format!("Invalid RPCModelContentChangeDTO for {}", self.URI));
			},
		};

		// A full implementation would require a rope data structure or complex logic
		// to apply deltas efficiently. We will log a warning and accept the new
		// version number, but the content will be out of sync until the next full
		// update.
		log::warn!(
			"Applying changes to {} by version bump only (delta application is a stub).",
			self.URI
		);

		self.Version = NewVersion;
		self.IsDirty = true;
		Ok(())
	}
}
