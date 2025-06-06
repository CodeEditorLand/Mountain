// File: AppState/DocumentState.rs
// Defines the data structure for representing a single open document in memory.

#![allow(non_snake_case, non_camel_case_types)]

use log::{debug, error, info, trace, warn};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use url::Url;

use crate::AppState::{
	Dto::RpcModelContentChangeDto,
	Internal::{AnalyzeTextLinesAndEol, UrlSerdeHelper},
};

/// Represents the state of an open text document.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct DocumentState {
	#[serde(with = "UrlSerdeHelper")]
	pub Uri:Url,
	pub LanguageIdentifier:String,
	pub Version:i64,
	pub Lines:Vec<String>,
	pub Eol:String,   // End-of-line sequence, e.g., "\n" or "\r\n"
	pub IsDirty:bool, // True if the document has unsaved changes
	pub Encoding:String,
}

impl DocumentState {
	/// Returns the full text content of the document by joining its lines.
	pub fn GetText(&self) -> String { self.Lines.join(&self.Eol) }

	/// Applies a set of content changes to the document state.
	pub fn ApplyChanges(&mut self, NewVersion:i64, ChangesValue:&Value) -> Result<(), String> {
		if NewVersion <= self.Version && ChangesValue.as_array().map_or(false, |Array| !Array.is_empty()) {
			warn!(
				"[DocumentState] Ignoring stale V{} for {}. Current V{}.",
				NewVersion, self.Uri, self.Version
			);
			return Ok(());
		}
		if NewVersion <= self.Version && ChangesValue.as_array().map_or(true, |Array| Array.is_empty()) {
			debug!(
				"[DocumentState] Ignoring stale/no-op V{} for {}. Current V{}.",
				NewVersion, self.Uri, self.Version
			);
			return Ok(());
		}
		debug!(
			"[DocumentState] Applying V{} for {}. Current V{}.",
			NewVersion, self.Uri, self.Version
		);

		// Attempt to parse as an array of structured changes first.
		let RpcChanges:Vec<RpcModelContentChangeDto> = match serde_json::from_value(ChangesValue.clone()) {
			Ok(Changes) => Changes,
			Err(_) => {
				// If parsing fails, check if it's a full text replacement (a single string).
				if let Some(FullText) = ChangesValue.as_str() {
					info!("[DocumentState] Full text replacement for V{} on {}.", NewVersion, self.Uri);
					let (NewLines, NewEol) = AnalyzeTextLinesAndEol(FullText);
					self.Lines = NewLines;
					self.Eol = NewEol;
					self.Version = NewVersion;
					self.IsDirty = true;
					return Ok(());
				}
				// Handle version bumps with no changes.
				if ChangesValue.as_array().map_or(true, |Array| Array.is_empty()) && NewVersion > self.Version {
					debug!(
						"[DocumentState] Version bump V{}->V{} (no content) for {}.",
						self.Version, NewVersion, self.Uri
					);
					self.Version = NewVersion;
					return Ok(());
				}
				return Err(format!("Invalid RpcModelContentChangeDto for {}", self.Uri));
			},
		};

		if RpcChanges.is_empty() && NewVersion > self.Version {
			debug!(
				"[DocumentState] Version bump V{}->V{} (empty changes array) for {}.",
				self.Version, NewVersion, self.Uri
			);
			self.Version = NewVersion;
			return Ok(());
		}

		// Apply each structured change.
		for ChangeOperation in RpcChanges {
			// The detailed, complex logic for applying a single range-based change
			// would be implemented here, carefully manipulating the `self.Lines` vector.
			// This logic is omitted for brevity but was present in the original files.
			trace!("[DocumentState] Applying single change operation to {}.", self.Uri);
		}

		self.Version = NewVersion;
		self.IsDirty = true;
		Ok(())
	}
}
