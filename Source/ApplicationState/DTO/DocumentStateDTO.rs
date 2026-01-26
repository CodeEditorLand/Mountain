//! # DocumentStateDTO
//!
//! Defines the Data Transfer Object for storing the state of a single open
//! text document in memory.

#![allow(non_snake_case, non_camel_case_types)]

use Common::Error::CommonError::CommonError;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use url::Url;

use super::RPCModelContentChangeDTO::RPCModelContentChangeDTO;
use crate::ApplicationState::Internal::{AnalyzeTextLinesAndEOL, URLSerializationHelper};

/// Represents the complete in-memory state of a single text document.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct DocumentStateDTO {
	/// The unique resource identifier for this document.
	#[serde(with = "URLSerializationHelper")]
	pub URI:Url,

	/// The VS Code language identifier (e.g., "rust", "typescript").
	pub LanguageIdentifier:String,

	/// The version number, incremented on each change from the client.
	pub Version:i64,

	/// The content of the document, split into lines.
	pub Lines:Vec<String>,

	/// The detected end-of-line sequence (e.g., `\n` or `\r\n`).
	pub EOL:String,

	/// A flag indicating if the in-memory version has unsaved changes.
	pub IsDirty:bool,

	/// The detected file encoding (e.g., "utf8").
	pub Encoding:String,

	/// An internal version number, used for tracking changes within the host.
	pub VersionIdentifier:i64,
}

impl DocumentStateDTO {
	/// Creates a new `DocumentStateDTO` from its initial content.
	pub fn Create(URI:Url, LanguageIdentifier:Option<String>, Content:String) -> Self {
		let (Lines, EOL) = AnalyzeTextLinesAndEOL(&Content);

		let LanguageID = LanguageIdentifier.unwrap_or_else(|| "plaintext".to_string());

		let Encoding = "utf8".to_string();

		Self {
			URI,

			LanguageIdentifier:LanguageID,

			Version:1,

			Lines,

			EOL,

			IsDirty:false,

			Encoding,

			VersionIdentifier:1,
		}
	}

	/// Reconstructs the full text content of the document from its lines.
	pub fn GetText(&self) -> String { self.Lines.join(&self.EOL) }

	/// Converts the struct to a `serde_json::Value`, useful for notifications.
	pub fn ToDTO(&self) -> Result<Value, CommonError> {
		serde_json::to_value(self).map_err(|Error| CommonError::SerializationError { Description:Error.to_string() })
	}

	/// Applies a set of changes to the document. This can be a full text
	/// replacement or a collection of delta changes.
	pub fn ApplyChanges(&mut self, NewVersion:i64, ChangesValue:&Value) -> Result<(), CommonError> {
		// Ignore stale changes.
		if NewVersion <= self.Version {
			return Ok(());
		}

		// Attempt to deserialize as an array of delta changes first.
		if let Ok(RPCChange) = serde_json::from_value::<Vec<RPCModelContentChangeDTO>>(ChangesValue.clone()) {
			// A full implementation would apply each delta change to the `Lines` vector.
			// This is a complex operation involving coordinate transformations.
			// For now, we will log that this is a stub and only update the version.
			log::warn!(
				"Applying changes to {} by version bump only (delta application is a stub).",
				self.URI
			);

			// In a real implementation:
			self.Lines = ApplyDeltaChanges(&self.Lines, &self.EOL, &RPCChange);
		} else if let Some(FullText) = ChangesValue.as_str() {
			// If it's not deltas, check if it's a full text replacement.
			let (NewLines, NewEOL) = AnalyzeTextLinesAndEOL(FullText);

			self.Lines = NewLines;

			self.EOL = NewEOL;
		} else {
			return Err(CommonError::InvalidArgument {
				ArgumentName:"ChangesValue".into(),

				Reason:format!(
					"Invalid change format for {}: expected string or RPCModelContentChangeDTO array.",
					self.URI
				),
			});
		}

		// Update metadata after changes have been applied.
		self.Version = NewVersion;

		self.VersionIdentifier += 1;

		self.IsDirty = true;

		Ok(())
	}
}

fn ApplyDeltaChanges(_Line:&[String], _EOL:&str, _RPCChange:&[RPCModelContentChangeDTO]) -> Vec<String> { todo!() }
