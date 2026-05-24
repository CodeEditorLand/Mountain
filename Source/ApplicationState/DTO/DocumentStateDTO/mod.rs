pub mod Create;
pub mod CreateUnsafe;
pub mod GetText;
pub mod ToDTO;
pub mod ApplyChanges;

use CommonLibrary::{Error::CommonError::CommonError, Utility::Serialization::URLSerializationHelper};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use url::Url;
use crate::{
	ApplicationState::Internal::TextProcessing::AnalyzeTextLinesAndEOL::Fn as AnalyzeTextLinesAndEOL,
	dev_log,
};
use super::{RPCModelContentChangeDTO::RPCModelContentChangeDTO, RPCRangeDTO::RPCRangeDTO};

/// Represents the complete in-memory state of a single text document.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Struct {
	/// The unique resource identifier for this document.
	#[serde(rename = "uri", with = "URLSerializationHelper")]
	pub URI:Url,

	/// The VS Code language identifier (e.g., "rust", "typescript").
	#[serde(skip_serializing_if = "String::is_empty")]
	pub LanguageIdentifier:String,

	/// The version number, incremented on each change from the client.
	pub Version:i64,

	/// The content of the document, split into lines.
	pub Lines:Vec<String>,

	/// The detected end-of-line sequence (e.g., `\n` or `\r\n`).
	#[serde(rename = "eol")]
	pub EOL:String,

	/// A flag indicating if the in-memory version has unsaved changes.
	pub IsDirty:bool,

	/// The detected file encoding (e.g., "utf8").
	pub Encoding:String,

	/// An internal version number, used for tracking changes within the host.
	pub VersionIdentifier:i64,
}
